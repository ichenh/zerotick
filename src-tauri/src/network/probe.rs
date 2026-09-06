//! A user-requested probe of one server. HTTP proxies are bypassed, but Windows
//! routing (including a VPN) still applies. This is not a browser or an all-network test.

use reqwest::Url;
use serde::Serialize;
use std::error::Error;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

static DNS_WORKER_BUSY: AtomicBool = AtomicBool::new(false);
const DNS_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Serialize)]
pub struct ConnectionProbe {
    pub target: String,
    pub dns_status: String,
    pub addresses: Vec<String>,
    pub http_status: Option<u16>,
    pub http_error: Option<String>,
    pub elapsed_ms: u64,
}

pub fn validate_target(target: &str) -> Result<Url, String> {
    let invalid = || "network_probe:invalid_target".to_string();
    if target.len() > 2048 || target.chars().any(char::is_control) {
        return Err(invalid());
    }
    let target = target.trim();
    if target.is_empty() {
        return Err(invalid());
    }
    let candidate = if let Some((scheme, _)) = target.split_once("://") {
        if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
            return Err(invalid());
        }
        target.to_string()
    } else if let Ok(ip) = target.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(_) => format!("https://{ip}/"),
            IpAddr::V6(_) => format!("https://[{ip}]/"),
        }
    } else {
        format!("https://{target}")
    };
    let url = Url::parse(&candidate).map_err(|_| invalid())?;
    // Reject even an empty user-info prefix (`https://@host`), which the URL
    // parser otherwise normalizes away. No error includes the user's URL.
    let authority = candidate
        .split_once("://")
        .map(|(_, rest)| rest.split(['/', '?', '#', '\\']).next().unwrap_or_default())
        .unwrap_or_default();
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none_or(str::is_empty)
        || !url.username().is_empty()
        || url.password().is_some()
        || authority.is_empty()
        || authority.contains('@')
        || url.as_str().len() > 2048
    {
        return Err(invalid());
    }
    Ok(url)
}

struct DnsWorkerSlot<'a>(&'a AtomicBool);

impl<'a> DnsWorkerSlot<'a> {
    fn acquire(busy: &'a AtomicBool) -> Option<Self> {
        busy.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| Self(busy))
    }
}

impl Drop for DnsWorkerSlot<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

enum DnsResolution {
    Resolved(Vec<SocketAddr>),
    Failed,
    Timeout,
    Busy,
}

fn resolve(host: String, port: u16) -> DnsResolution {
    let Some(slot) = DnsWorkerSlot::acquire(&DNS_WORKER_BUSY) else {
        return DnsResolution::Busy;
    };
    let (sender, receiver) = mpsc::channel();
    // Windows' native resolver cannot be cancelled by recv_timeout. Ownership
    // stays with the actual worker until it exits, so repeated clicks cannot
    // accumulate resolver threads. A spawn failure also drops the captured slot.
    let worker = std::thread::Builder::new()
        .name("network-target-dns".into())
        .spawn(move || {
            let _slot = slot;
            let addresses = (host.as_str(), port)
                .to_socket_addrs()
                .map(|addresses| addresses.collect::<Vec<_>>());
            let _ = sender.send(addresses);
        });
    if worker.is_err() {
        return DnsResolution::Busy;
    }
    match receiver.recv_timeout(DNS_TIMEOUT) {
        Ok(Ok(addresses)) if !addresses.is_empty() => DnsResolution::Resolved(addresses),
        Ok(_) => DnsResolution::Failed,
        Err(RecvTimeoutError::Disconnected) => DnsResolution::Busy,
        Err(RecvTimeoutError::Timeout) => DnsResolution::Timeout,
    }
}

fn classify_http_failure(
    timed_out: bool,
    connecting: bool,
    error: &(dyn Error + 'static),
) -> &'static str {
    if timed_out {
        return "timeout";
    }
    // reqwest's outer Display includes the URL. Inspect only underlying causes
    // to avoid treating a hostname/path containing "tls" as a TLS failure.
    let mut cause = error.source();
    while let Some(error) = cause {
        let detail = error.to_string().to_ascii_lowercase();
        if ["tls", "ssl", "certificate", "invalid peer"]
            .iter()
            .any(|marker| detail.contains(marker))
        {
            return "tls";
        }
        cause = error.source();
    }
    if connecting {
        "connect"
    } else {
        "response"
    }
}

fn http_error_id(error: &reqwest::Error) -> &'static str {
    classify_http_failure(error.is_timeout(), error.is_connect(), error)
}

/// An HTTP response is evidence only for this target and this request. Error
/// responses such as 403 or 500 still prove an HTTP response was received.
pub fn response_observed(probe: &ConnectionProbe) -> bool {
    probe
        .http_status
        .is_some_and(|status| (100..600).contains(&status))
}

pub fn run(target: &str) -> Result<ConnectionProbe, String> {
    let started = Instant::now();
    let url = validate_target(target)?;
    let host = url
        .host_str()
        .ok_or_else(|| "network_probe:invalid_target".to_string())?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| "network_probe:invalid_target".to_string())?;
    let literal_ip = host.trim_matches(['[', ']']).parse::<IpAddr>().ok();
    let mut report = ConnectionProbe {
        target: url.as_str().to_string(),
        dns_status: "not_required".into(),
        addresses: Vec::new(),
        http_status: None,
        http_error: None,
        elapsed_ms: 0,
    };
    let addresses = if let Some(ip) = literal_ip {
        vec![SocketAddr::new(ip, port)]
    } else {
        match resolve(host.to_string(), port) {
            DnsResolution::Resolved(addresses) => {
                report.dns_status = "resolved".into();
                addresses
            }
            resolution => {
                report.dns_status = match resolution {
                    DnsResolution::Failed => "failed",
                    DnsResolution::Timeout => "timeout",
                    DnsResolution::Busy => "busy",
                    DnsResolution::Resolved(_) => unreachable!(),
                }
                .into();
                report.http_error = Some("dns_unavailable".into());
                report.elapsed_ms = started.elapsed().as_millis() as u64;
                return Ok(report);
            }
        }
    };
    for address in &addresses {
        let ip = address.ip().to_string();
        if !report.addresses.contains(&ip) {
            report.addresses.push(ip);
        }
    }
    let mut builder = reqwest::blocking::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(6))
        .user_agent("ZeroTick connection probe");
    if literal_ip.is_none() {
        // Reuse this probe's actual resolution instead of asking DNS again.
        builder = builder.resolve_to_addrs(host, &addresses);
    }
    let response = builder.build().and_then(|client| {
        client
            .get(url)
            .header(reqwest::header::RANGE, "bytes=0-0")
            .send()
    });
    match response {
        Ok(response) => {
            report.http_status = Some(response.status().as_u16());
            // Never collect the body, including when the server ignores Range.
            drop(response);
        }
        Err(error) => report.http_error = Some(http_error_id(&error).into()),
    }
    report.elapsed_ms = started.elapsed().as_millis() as u64;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[test]
    fn target_defaults_to_https_and_preserves_explicit_http_and_port() {
        assert_eq!(
            validate_target(" Example.COM/path?q=1 ").unwrap().as_str(),
            "https://example.com/path?q=1"
        );
        assert_eq!(
            validate_target("http://example.com:8080/")
                .unwrap()
                .as_str(),
            "http://example.com:8080/"
        );
        assert_eq!(
            validate_target("example.com:8443").unwrap().as_str(),
            "https://example.com:8443/"
        );
        assert_eq!(
            validate_target("2001:db8::1").unwrap().as_str(),
            "https://[2001:db8::1]/"
        );
    }

    #[test]
    fn target_rejects_credentials_non_http_and_malformed_input() {
        for target in [
            "",
            "https://",
            "file:///C:/test",
            "ftp://example.com",
            "javascript:alert(1)",
            "https://user:secret@example.com",
            "https://user@example.com",
            "https://@example.com",
            "https://example.com\n",
            "https://example.com\u{007f}",
            "example.com:wrong",
        ] {
            assert_eq!(
                validate_target(target).unwrap_err(),
                "network_probe:invalid_target"
            );
        }
        assert!(validate_target(&format!("https://example.com/{}", "a".repeat(2048))).is_err());
    }

    #[test]
    fn dns_timeout_keeps_slot_owned_until_the_worker_exits() {
        let busy = AtomicBool::new(false);
        let worker_slot = DnsWorkerSlot::acquire(&busy).unwrap();
        let (_sender, receiver) = mpsc::channel::<()>();
        assert!(matches!(
            receiver.recv_timeout(Duration::ZERO),
            Err(RecvTimeoutError::Timeout)
        ));
        assert!(DnsWorkerSlot::acquire(&busy).is_none());
        drop(worker_slot);
        assert!(DnsWorkerSlot::acquire(&busy).is_some());
    }

    #[derive(Debug)]
    struct TestError {
        message: &'static str,
        cause: Option<Box<TestError>>,
    }

    impl fmt::Display for TestError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.message)
        }
    }

    impl Error for TestError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.cause.as_deref().map(|error| error as &dyn Error)
        }
    }

    #[test]
    fn tls_cause_is_classified_before_generic_connect_error() {
        let error = TestError {
            message: "request failed",
            cause: Some(Box::new(TestError {
                message: "connection failed",
                cause: Some(Box::new(TestError {
                    message: "invalid peer certificate: UnknownIssuer",
                    cause: None,
                })),
            })),
        };
        assert_eq!(classify_http_failure(false, true, &error), "tls");
        assert_eq!(classify_http_failure(true, true, &error), "timeout");
    }

    #[test]
    fn url_text_does_not_turn_connection_failures_into_tls_errors() {
        let error = TestError {
            message: "request failed for https://tls.example.com/certificate",
            cause: Some(Box::new(TestError {
                message: "connection refused",
                cause: None,
            })),
        };
        assert_eq!(classify_http_failure(false, true, &error), "connect");
        assert_eq!(classify_http_failure(false, false, &error), "response");
    }

    #[test]
    fn http_error_responses_are_evidence_but_dns_alone_is_not() {
        let mut probe = ConnectionProbe {
            target: "https://example.com/".into(),
            dns_status: "resolved".into(),
            addresses: vec!["192.0.2.1".into()],
            http_status: None,
            http_error: Some("timeout".into()),
            elapsed_ms: 1,
        };
        assert!(!response_observed(&probe));
        for status in [200, 302, 403, 404, 500, 503] {
            probe.http_status = Some(status);
            assert!(response_observed(&probe));
        }
        for status in [0, 99, 600] {
            probe.http_status = Some(status);
            assert!(!response_observed(&probe));
        }
    }
}
