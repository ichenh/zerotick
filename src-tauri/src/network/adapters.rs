//! Current interface configuration from IP Helper. These records are not a
//! connectivity test, and each interface keeps its own IPv4/IPv6 gateways.

use serde::Serialize;
use std::collections::HashSet;
use std::mem::{align_of, size_of};
use std::net::{Ipv4Addr, Ipv6Addr};
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_NO_DATA, ERROR_SUCCESS};
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_INCLUDE_ALL_INTERFACES, GAA_FLAG_INCLUDE_GATEWAYS,
    GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_MULTICAST, IF_TYPE_ETHERNET_CSMACD, IF_TYPE_IEEE80211,
    IF_TYPE_SOFTWARE_LOOPBACK, IF_TYPE_TUNNEL, IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_DHCP_ENABLED,
};
use windows::Win32::NetworkManagement::Ndis::{
    IfOperStatusDormant as IF_OPER_STATUS_DORMANT, IfOperStatusDown as IF_OPER_STATUS_DOWN,
    IfOperStatusLowerLayerDown as IF_OPER_STATUS_LOWER_LAYER_DOWN,
    IfOperStatusNotPresent as IF_OPER_STATUS_NOT_PRESENT,
    IfOperStatusTesting as IF_OPER_STATUS_TESTING, IfOperStatusUp as IF_OPER_STATUS_UP,
    IF_OPER_STATUS,
};
use windows::Win32::Networking::WinSock::{
    IpDadStateDeprecated as IP_DAD_STATE_DEPRECATED, IpDadStatePreferred as IP_DAD_STATE_PREFERRED,
    ADDRESS_FAMILY, AF_INET, AF_INET6, AF_UNSPEC, NL_DAD_STATE, SOCKADDR_IN, SOCKADDR_IN6,
    SOCKET_ADDRESS,
};

const INITIAL_BUFFER_BYTES: usize = 15_000;
const MAX_BUFFER_BYTES: usize = 4 * 1024 * 1024;
const MAX_BUFFER_ATTEMPTS: usize = 3;

#[derive(Debug, Clone, Serialize)]
pub struct NetworkAdapter {
    /// Windows' internal adapter name, not the renameable friendly name or index.
    pub id: String,
    pub name: String,
    pub description: String,
    pub if_index: u32,
    pub ipv6_if_index: u32,
    pub kind: String,
    pub oper_status: String,
    /// DHCPv4 configuration only; this says nothing about DHCPv6 or lease health.
    pub dhcp_enabled: bool,
    /// Assigned, nonexpired addresses with preferred/deprecated DAD state.
    /// Interface operational state and actual reachability remain separate.
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub gateways: Vec<String>,
    pub dns_servers: Vec<String>,
    pub has_apipa: bool,
    pub ipv4_metric: u32,
    pub ipv6_metric: u32,
}

pub fn snapshot() -> Result<Vec<NetworkAdapter>, String> {
    // GetAdaptersAddresses is synchronous and has no cancellation parameter.
    // Its caller must keep this work off the UI thread and share the snapshot.
    let mut requested = INITIAL_BUFFER_BYTES;
    for _ in 0..MAX_BUFFER_ATTEMPTS {
        if requested > MAX_BUFFER_BYTES {
            return Err("network_adapters:buffer_limit_exceeded".into());
        }
        // The API's linked structures contain u64 alignment unions. Vec<u8>
        // would not guarantee their required alignment, including on 32-bit hosts.
        let mut storage = vec![0u64; requested.div_ceil(size_of::<u64>())];
        let allocated = storage.len() * size_of::<u64>();
        let mut bytes = allocated as u32;
        let first = storage.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
        let result = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_INCLUDE_ALL_INTERFACES
                    | GAA_FLAG_INCLUDE_GATEWAYS
                    | GAA_FLAG_SKIP_ANYCAST
                    | GAA_FLAG_SKIP_MULTICAST,
                None,
                Some(first),
                &mut bytes,
            )
        };
        if result == ERROR_NO_DATA.0 {
            return Ok(Vec::new());
        }
        if result == ERROR_BUFFER_OVERFLOW.0 {
            requested = bytes as usize;
            continue;
        }
        if result != ERROR_SUCCESS.0 {
            return Err(format!(
                "network_adapters:query_failed:{result}:{}",
                windows::core::Error::from_hresult(windows::core::HRESULT::from_win32(result))
            ));
        }
        let returned = bytes as usize;
        if returned == 0 || returned > allocated {
            return Err(invalid_buffer());
        }
        let raw = unsafe { std::slice::from_raw_parts(storage.as_ptr().cast::<u8>(), returned) };
        return parse_snapshot(&NativeBuffer { bytes: raw }, first);
    }
    Err("network_adapters:configuration_changed_during_query".into())
}

fn parse_snapshot(
    buffer: &NativeBuffer<'_>,
    first: *mut IP_ADAPTER_ADDRESSES_LH,
) -> Result<Vec<NetworkAdapter>, String> {
    let mut adapters = Vec::new();
    for native in buffer.nodes(first, |node| node.Next)? {
        if native.IfType == IF_TYPE_SOFTWARE_LOOPBACK {
            continue;
        }
        let id = buffer.ansi(native.AdapterName.0)?;
        if id.is_empty() {
            return Err("network_adapters:missing_adapter_identity".into());
        }
        let mut adapter = NetworkAdapter {
            id,
            name: buffer.wide(native.FriendlyName.0)?,
            description: buffer.wide(native.Description.0)?,
            if_index: unsafe { native.Anonymous1.Anonymous.IfIndex },
            ipv6_if_index: native.Ipv6IfIndex,
            kind: adapter_kind(native.IfType).into(),
            oper_status: operational_state(native.OperStatus).into(),
            dhcp_enabled: unsafe { native.Anonymous2.Flags } & IP_ADAPTER_DHCP_ENABLED != 0,
            ipv4_addresses: Vec::new(),
            ipv6_addresses: Vec::new(),
            gateways: Vec::new(),
            dns_servers: Vec::new(),
            has_apipa: false,
            ipv4_metric: native.Ipv4Metric,
            ipv6_metric: native.Ipv6Metric,
        };
        for address in buffer.nodes(native.FirstUnicastAddress, |node| node.Next)? {
            if !usable_unicast(address.DadState, address.ValidLifetime) {
                continue;
            }
            match buffer.socket_ip(&address.Address)? {
                Some(SocketIp::V4(ip)) if !ip.is_unspecified() => {
                    adapter.has_apipa |= is_apipa(ip);
                    push_unique(&mut adapter.ipv4_addresses, ip.to_string());
                }
                Some(SocketIp::V6(ip, scope)) if !ip.is_unspecified() => {
                    push_unique(&mut adapter.ipv6_addresses, ipv6_text(ip, scope));
                }
                _ => {}
            }
        }
        for gateway in buffer.nodes(native.FirstGatewayAddress, |node| node.Next)? {
            if let Some(ip) = buffer.socket_ip(&gateway.Address)? {
                if !ip.is_unspecified() {
                    push_unique(&mut adapter.gateways, ip.text());
                }
            }
        }
        for server in buffer.nodes(native.FirstDnsServerAddress, |node| node.Next)? {
            if let Some(ip) = buffer.socket_ip(&server.Address)? {
                if !ip.is_unspecified() {
                    push_unique(&mut adapter.dns_servers, ip.text());
                }
            }
        }
        adapters.push(adapter);
    }
    adapters.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
    Ok(adapters)
}

struct NativeBuffer<'a> {
    bytes: &'a [u8],
}

impl NativeBuffer<'_> {
    fn offset(&self, pointer: *const u8, length: usize) -> Result<usize, String> {
        let offset = (pointer as usize)
            .checked_sub(self.bytes.as_ptr() as usize)
            .ok_or_else(invalid_buffer)?;
        if pointer.is_null()
            || offset
                .checked_add(length)
                .is_none_or(|end| end > self.bytes.len())
        {
            return Err(invalid_buffer());
        }
        Ok(offset)
    }

    // Used only for the Win32 plain-data records below. All reads come from the
    // owned initialized buffer, after checking bounds and native alignment.
    fn read<T: Copy>(&self, pointer: *const T) -> Result<T, String> {
        let offset = self.offset(pointer.cast(), size_of::<T>())?;
        if (pointer as usize) % align_of::<T>() != 0 {
            return Err(invalid_buffer());
        }
        Ok(unsafe { self.bytes.as_ptr().add(offset).cast::<T>().read_unaligned() })
    }

    fn nodes<T: Copy>(
        &self,
        mut pointer: *mut T,
        next: impl Fn(&T) -> *mut T,
    ) -> Result<Vec<T>, String> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        while !pointer.is_null() {
            if !seen.insert(pointer as usize) || result.len() >= self.bytes.len() / size_of::<T>() {
                return Err(invalid_buffer());
            }
            // Every IP Helper list record starts with its own versioned Length.
            let length = self.read(pointer.cast::<u32>())? as usize;
            if length < size_of::<T>() {
                return Err("network_adapters:unsupported_record_length".into());
            }
            self.offset(pointer.cast(), length)?;
            let node = self.read(pointer)?;
            pointer = next(&node);
            result.push(node);
        }
        Ok(result)
    }

    fn ansi(&self, pointer: *const u8) -> Result<String, String> {
        let offset = self.offset(pointer, 1)?;
        let suffix = &self.bytes[offset..];
        let length = suffix
            .iter()
            .position(|value| *value == 0)
            .ok_or_else(invalid_buffer)?;
        // The internal adapter identity is ASCII. Never replace invalid bytes
        // and risk conflating two different identifiers.
        String::from_utf8(suffix[..length].to_vec()).map_err(|_| invalid_buffer())
    }

    fn wide(&self, pointer: *const u16) -> Result<String, String> {
        if pointer.is_null() {
            return Ok(String::new());
        }
        let offset = self.offset(pointer.cast(), size_of::<u16>())?;
        if (pointer as usize) % align_of::<u16>() != 0 {
            return Err(invalid_buffer());
        }
        let mut units = Vec::new();
        for pair in self.bytes[offset..].chunks_exact(2) {
            let unit = u16::from_ne_bytes([pair[0], pair[1]]);
            if unit == 0 {
                return Ok(String::from_utf16_lossy(&units));
            }
            units.push(unit);
        }
        Err(invalid_buffer())
    }

    fn socket_ip(&self, socket: &SOCKET_ADDRESS) -> Result<Option<SocketIp>, String> {
        let length = usize::try_from(socket.iSockaddrLength).map_err(|_| invalid_buffer())?;
        self.offset(socket.lpSockaddr.cast(), length)?;
        if length < size_of::<ADDRESS_FAMILY>() {
            return Err(invalid_buffer());
        }
        let family = self.read(socket.lpSockaddr.cast::<ADDRESS_FAMILY>())?;
        if family == AF_INET {
            if length < size_of::<SOCKADDR_IN>() {
                return Err(invalid_buffer());
            }
            let address = self.read(socket.lpSockaddr.cast::<SOCKADDR_IN>())?;
            Ok(Some(SocketIp::V4(ipv4_from_network_u32(unsafe {
                address.sin_addr.S_un.S_addr
            }))))
        } else if family == AF_INET6 {
            if length < size_of::<SOCKADDR_IN6>() {
                return Err(invalid_buffer());
            }
            let address = self.read(socket.lpSockaddr.cast::<SOCKADDR_IN6>())?;
            Ok(Some(SocketIp::V6(
                Ipv6Addr::from(unsafe { address.sin6_addr.u.Byte }),
                unsafe { address.Anonymous.sin6_scope_id },
            )))
        } else {
            Ok(None)
        }
    }
}

enum SocketIp {
    V4(Ipv4Addr),
    V6(Ipv6Addr, u32),
}

impl SocketIp {
    fn is_unspecified(&self) -> bool {
        match self {
            Self::V4(ip) => ip.is_unspecified(),
            Self::V6(ip, _) => ip.is_unspecified(),
        }
    }

    fn text(&self) -> String {
        match self {
            Self::V4(ip) => ip.to_string(),
            Self::V6(ip, scope) => ipv6_text(*ip, *scope),
        }
    }
}

fn invalid_buffer() -> String {
    "network_adapters:invalid_native_buffer".into()
}

fn ipv4_from_network_u32(address: u32) -> Ipv4Addr {
    // S_addr's in-memory bytes are already in network order; to_be_bytes would
    // reverse the address on Windows' little-endian architectures.
    Ipv4Addr::from(address.to_ne_bytes())
}

fn ipv6_text(address: Ipv6Addr, scope: u32) -> String {
    if scope == 0 {
        address.to_string()
    } else {
        format!("{address}%{scope}")
    }
}

fn is_apipa(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    octets[0] == 169 && octets[1] == 254
}

fn usable_unicast(state: NL_DAD_STATE, lifetime: u32) -> bool {
    lifetime != 0 && matches!(state, IP_DAD_STATE_PREFERRED | IP_DAD_STATE_DEPRECATED)
}

fn operational_state(state: IF_OPER_STATUS) -> &'static str {
    match state {
        IF_OPER_STATUS_UP => "up",
        IF_OPER_STATUS_DOWN => "down",
        IF_OPER_STATUS_DORMANT => "dormant",
        IF_OPER_STATUS_NOT_PRESENT => "not_present",
        IF_OPER_STATUS_LOWER_LAYER_DOWN => "lower_layer_down",
        IF_OPER_STATUS_TESTING => "testing",
        _ => "unknown",
    }
}

fn adapter_kind(kind: u32) -> &'static str {
    match kind {
        IF_TYPE_ETHERNET_CSMACD => "ethernet",
        IF_TYPE_IEEE80211 => "wifi",
        IF_TYPE_TUNNEL => "tunnel",
        _ => "other",
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Networking::WinSock::{
        IpDadStateDuplicate as IP_DAD_STATE_DUPLICATE, IpDadStateInvalid as IP_DAD_STATE_INVALID,
        IpDadStateTentative as IP_DAD_STATE_TENTATIVE,
    };

    #[test]
    fn ipv4_native_storage_preserves_network_byte_order() {
        for octets in [[192, 168, 1, 42], [10, 0, 0, 1], [169, 254, 20, 9]] {
            assert_eq!(
                ipv4_from_network_u32(u32::from_ne_bytes(octets)).octets(),
                octets
            );
        }
    }

    #[test]
    fn ipv6_preserves_link_local_scope_without_adding_a_global_scope() {
        assert_eq!(ipv6_text("fe80::1".parse().unwrap(), 12), "fe80::1%12");
        assert_eq!(
            ipv6_text("2001:db8::42".parse().unwrap(), 0),
            "2001:db8::42"
        );
    }

    #[test]
    fn apipa_is_limited_to_ipv4_link_local_prefix() {
        for ip in ["169.254.0.1", "169.254.255.254"] {
            assert!(is_apipa(ip.parse().unwrap()));
        }
        for ip in ["169.253.255.254", "169.255.0.1", "192.168.1.1"] {
            assert!(!is_apipa(ip.parse().unwrap()));
        }
    }

    #[test]
    fn tentative_duplicate_invalid_and_expired_addresses_are_not_usable() {
        for state in [
            IP_DAD_STATE_TENTATIVE,
            IP_DAD_STATE_DUPLICATE,
            IP_DAD_STATE_INVALID,
        ] {
            assert!(!usable_unicast(state, u32::MAX));
        }
        assert!(!usable_unicast(IP_DAD_STATE_PREFERRED, 0));
        assert!(usable_unicast(IP_DAD_STATE_PREFERRED, 120));
        assert!(usable_unicast(IP_DAD_STATE_DEPRECATED, 120));
    }

    #[test]
    fn operational_states_do_not_turn_unavailable_interfaces_into_up() {
        for (state, label) in [
            (IF_OPER_STATUS_UP, "up"),
            (IF_OPER_STATUS_DOWN, "down"),
            (IF_OPER_STATUS_DORMANT, "dormant"),
            (IF_OPER_STATUS_NOT_PRESENT, "not_present"),
            (IF_OPER_STATUS_LOWER_LAYER_DOWN, "lower_layer_down"),
            (IF_OPER_STATUS_TESTING, "testing"),
            (IF_OPER_STATUS(99), "unknown"),
        ] {
            assert_eq!(operational_state(state), label);
        }
    }

    #[test]
    fn native_buffer_rejects_foreign_unaligned_and_overrunning_pointers() {
        let storage = [0u64; 4];
        let bytes = unsafe {
            std::slice::from_raw_parts(
                storage.as_ptr().cast::<u8>(),
                std::mem::size_of_val(&storage),
            )
        };
        let buffer = NativeBuffer { bytes };
        let foreign = 1u64;
        assert!(buffer.read(&foreign).is_err());
        assert!(buffer.read(bytes[1..].as_ptr().cast::<u32>()).is_err());
        assert!(buffer.read(bytes[28..].as_ptr().cast::<u64>()).is_err());
        assert!(buffer.read(storage.as_ptr()).is_ok());
    }
}
