//! Native ICMP evidence. A timeout means no Echo Reply was observed, not that
//! the gateway is broken. Local API failures remain unknown.

use std::mem::{align_of, size_of, size_of_val};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use windows::Win32::Foundation::{GetLastError, ERROR_TIMEOUT, HANDLE};
use windows::Win32::NetworkManagement::IpHelper::{
    Icmp6CreateFile, Icmp6SendEcho2, IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
    ICMPV6_ECHO_REPLY_LH, ICMP_ECHO_REPLY, IP_DEST_HOST_UNREACHABLE, IP_DEST_NET_UNREACHABLE,
    IP_DEST_PORT_UNREACHABLE, IP_DEST_PROT_UNREACHABLE, IP_DEST_UNREACHABLE, IP_PACKET_TOO_BIG,
    IP_PARAM_PROBLEM, IP_REQ_TIMED_OUT, IP_SUCCESS, IP_TIME_EXCEEDED, IP_TTL_EXPIRED_REASSEM,
    IP_TTL_EXPIRED_TRANSIT,
};
use windows::Win32::Networking::WinSock::{
    AF_INET6, IN6_ADDR, IN6_ADDR_0, SOCKADDR_IN6, SOCKADDR_IN6_0,
};
use windows::Win32::System::IO::IO_STATUS_BLOCK;

const TIMEOUT_MS: u32 = 1500;
const PAYLOAD: &[u8; 8] = b"ZeroTick";
// Both APIs write native structs. u64 storage supplies their required alignment
// and space for the payload, ICMP error data, and IPv6 IO_STATUS_BLOCK.
type ReplyBuffer = [u64; 32];
const _: () = assert!(align_of::<ReplyBuffer>() >= align_of::<ICMP_ECHO_REPLY>());
const _: () = assert!(align_of::<ReplyBuffer>() >= align_of::<ICMPV6_ECHO_REPLY_LH>());
const _: () = assert!(size_of::<ReplyBuffer>() >= size_of::<ICMP_ECHO_REPLY>() + PAYLOAD.len() + 8);
const _: () = assert!(
    size_of::<ReplyBuffer>()
        >= size_of::<ICMPV6_ECHO_REPLY_LH>() + PAYLOAD.len() + 8 + size_of::<IO_STATUS_BLOCK>()
);

struct IcmpHandle(HANDLE);

impl Drop for IcmpHandle {
    fn drop(&mut self) {
        // This wrapper is constructed only from a successful CreateFile result.
        let _ = unsafe { IcmpCloseHandle(self.0) };
    }
}

fn parse_target(host: &str) -> Option<(IpAddr, u32)> {
    let host = if host.starts_with('[') {
        host.strip_prefix('[')?.strip_suffix(']')?
    } else {
        host
    };
    let (address, scope) = match host.split_once('%') {
        Some((address, scope))
            if !scope.is_empty() && scope.bytes().all(|byte| byte.is_ascii_digit()) =>
        {
            (address.parse::<IpAddr>().ok()?, scope.parse::<u32>().ok()?)
        }
        Some(_) => return None,
        None => (host.parse::<IpAddr>().ok()?, 0),
    };
    match address {
        IpAddr::V4(ip)
            if host.contains('%')
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_broadcast() =>
        {
            None
        }
        IpAddr::V6(ip)
            if ip.is_unspecified()
                || ip.is_multicast()
                || (ip.is_unicast_link_local() && scope == 0) =>
        {
            None
        }
        _ => Some((address, scope)),
    }
}

fn no_reply_status(status: u32) -> Option<bool> {
    match status {
        IP_REQ_TIMED_OUT
        | IP_DEST_NET_UNREACHABLE
        | IP_DEST_HOST_UNREACHABLE
        | IP_DEST_PROT_UNREACHABLE
        | IP_DEST_PORT_UNREACHABLE
        | IP_DEST_UNREACHABLE
        | IP_PACKET_TOO_BIG
        | IP_TTL_EXPIRED_TRANSIT
        | IP_TTL_EXPIRED_REASSEM
        | IP_TIME_EXCEEDED
        | IP_PARAM_PROBLEM => Some(false),
        // Includes invalid arguments, unsupported stacks/APIs, permission and
        // resource failures, bad scopes, and IP_GENERAL_FAILURE.
        _ => None,
    }
}

fn call_failure(error: u32) -> Option<bool> {
    if error == ERROR_TIMEOUT.0 {
        Some(false)
    } else {
        no_reply_status(error)
    }
}

fn reply_outcome(status: u32, target_matches: bool) -> Option<bool> {
    if status == IP_SUCCESS {
        Some(target_matches)
    } else {
        no_reply_status(status)
    }
}

fn ipv4_native(ip: Ipv4Addr) -> u32 {
    // IPAddr has network-order bytes in memory, as returned by inet_addr.
    u32::from_ne_bytes(ip.octets())
}

fn ipv6_socket(ip: Ipv6Addr, scope: u32) -> SOCKADDR_IN6 {
    SOCKADDR_IN6 {
        sin6_family: AF_INET6,
        sin6_addr: IN6_ADDR {
            u: IN6_ADDR_0 { Byte: ip.octets() },
        },
        Anonymous: SOCKADDR_IN6_0 {
            sin6_scope_id: scope,
        },
        ..Default::default()
    }
}

fn ipv6_native_words(ip: Ipv6Addr) -> [u16; 8] {
    let bytes = ip.octets();
    std::array::from_fn(|index| u16::from_ne_bytes([bytes[index * 2], bytes[index * 2 + 1]]))
}

pub fn ping(host: &str) -> Option<bool> {
    let (address, scope) = parse_target(host)?;
    let mut buffer: ReplyBuffer = [0; 32];
    let buffer_size = size_of_val(&buffer) as u32;
    match address {
        IpAddr::V4(ip) => {
            let handle = IcmpHandle(unsafe { IcmpCreateFile() }.ok()?);
            let target = ipv4_native(ip);
            // All pointers refer to live, aligned storage until this synchronous
            // call returns; no event/APC or detached work is involved.
            let count = unsafe {
                IcmpSendEcho(
                    handle.0,
                    target,
                    PAYLOAD.as_ptr().cast(),
                    PAYLOAD.len() as u16,
                    None,
                    buffer.as_mut_ptr().cast(),
                    buffer_size,
                    TIMEOUT_MS,
                )
            };
            if count == 0 {
                return call_failure(unsafe { GetLastError() }.0);
            }
            if count as usize > size_of_val(&buffer) / size_of::<ICMP_ECHO_REPLY>() {
                return None;
            }
            // The documented reply layout begins with count consecutive headers.
            let replies = unsafe {
                std::slice::from_raw_parts(
                    buffer.as_ptr().cast::<ICMP_ECHO_REPLY>(),
                    count as usize,
                )
            };
            let mut outcome = None;
            for reply in replies {
                match reply_outcome(reply.Status, reply.Address == target) {
                    Some(true) => return Some(true),
                    Some(false) => outcome = Some(false),
                    None => {}
                }
            }
            outcome
        }
        IpAddr::V6(ip) => {
            let handle = IcmpHandle(unsafe { Icmp6CreateFile() }.ok()?);
            let source = ipv6_socket(Ipv6Addr::UNSPECIFIED, 0);
            let destination = ipv6_socket(ip, scope);
            let count = unsafe {
                Icmp6SendEcho2(
                    handle.0,
                    None,
                    None,
                    None,
                    &source,
                    &destination,
                    PAYLOAD.as_ptr().cast(),
                    PAYLOAD.len() as u16,
                    None,
                    buffer.as_mut_ptr().cast(),
                    buffer_size,
                    TIMEOUT_MS,
                )
            };
            if count == 0 {
                return call_failure(unsafe { GetLastError() }.0);
            }
            // Synchronous Icmp6SendEcho2 already populated the reply header.
            // Icmp6ParseReplies is for asynchronous completion; do not parse twice.
            let reply = unsafe { &*buffer.as_ptr().cast::<ICMPV6_ECHO_REPLY_LH>() };
            // IPV6_ADDRESS_EX is packed: copy fields before comparing to avoid
            // creating references to potentially unaligned members.
            let reply_address = reply.Address.sin6_addr;
            let reply_scope = reply.Address.sin6_scope_id;
            reply_outcome(
                reply.Status,
                reply_address == ipv6_native_words(ip) && (scope == 0 || reply_scope == scope),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::{
        ERROR_ACCESS_DENIED, ERROR_CALL_NOT_IMPLEMENTED, ERROR_INVALID_PARAMETER,
        ERROR_NOT_SUPPORTED,
    };
    use windows::Win32::NetworkManagement::IpHelper::IP_GENERAL_FAILURE;

    #[test]
    fn unreachable_and_expired_replies_are_never_success() {
        for status in [
            IP_DEST_HOST_UNREACHABLE,
            IP_TTL_EXPIRED_TRANSIT,
            IP_REQ_TIMED_OUT,
        ] {
            assert_eq!(reply_outcome(status, true), Some(false));
            assert_eq!(call_failure(status), Some(false));
        }
        assert_eq!(reply_outcome(IP_SUCCESS, true), Some(true));
        assert_eq!(reply_outcome(IP_SUCCESS, false), Some(false));
        assert_eq!(call_failure(IP_SUCCESS), None);
    }

    #[test]
    fn api_failures_remain_unknown() {
        for code in [
            ERROR_ACCESS_DENIED.0,
            ERROR_CALL_NOT_IMPLEMENTED.0,
            ERROR_INVALID_PARAMETER.0,
            ERROR_NOT_SUPPORTED.0,
            IP_GENERAL_FAILURE,
        ] {
            assert_eq!(call_failure(code), None);
        }
        assert_eq!(call_failure(ERROR_TIMEOUT.0), Some(false));
    }

    #[test]
    fn parses_only_unicast_ip_addresses_and_numeric_ipv6_scopes() {
        assert_eq!(
            parse_target("192.0.2.1"),
            Some(("192.0.2.1".parse().unwrap(), 0))
        );
        assert_eq!(
            parse_target("fe80::1%12"),
            Some(("fe80::1".parse().unwrap(), 12))
        );
        assert_eq!(
            parse_target("[2001:db8::1]"),
            Some(("2001:db8::1".parse().unwrap(), 0))
        );
        for host in [
            "example.com",
            "192.0.2.1%1",
            "fe80::1",
            "fe80::1%WiFi",
            "fe80::1%+1",
            "::",
            "ff02::1",
            "255.255.255.255",
        ] {
            assert_eq!(parse_target(host), None);
        }
    }

    #[test]
    fn native_address_fields_preserve_network_order_bytes() {
        let ipv4: Ipv4Addr = "192.0.2.1".parse().unwrap();
        assert_eq!(ipv4_native(ipv4).to_ne_bytes(), ipv4.octets());
        let ipv6: Ipv6Addr = "fe80::1234".parse().unwrap();
        let socket = ipv6_socket(ipv6, 12);
        assert_eq!(unsafe { socket.sin6_addr.u.Byte }, ipv6.octets());
        assert_eq!(unsafe { socket.Anonymous.sin6_scope_id }, 12);
        let native_bytes = ipv6_native_words(ipv6)
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect::<Vec<_>>();
        assert_eq!(native_bytes, ipv6.octets());
    }
}
