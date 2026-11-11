use crate::Adapter;
use serde::Serialize;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{collections::{HashMap, HashSet}, net::{Ipv4Addr, SocketAddrV4}, time::Duration};

const PROBE_DESTINATION: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(1, 1, 1, 1), 443);
const MAX_ADDRESSES: usize = 16;

#[derive(Serialize)]
pub struct AdapterTraffic {
    pub address: String,
    pub index: Option<u32>,
    pub received_bytes: Option<u64>,
    pub sent_bytes: Option<u64>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct Connectivity {
    pub address: String,
    pub reachable: bool,
    pub message: String,
}

pub fn requested_addresses(addresses: &[String]) -> Result<Vec<Ipv4Addr>, String> {
    if addresses.len() > MAX_ADDRESSES {
        return Err(format!("Select at most {MAX_ADDRESSES} addresses for diagnostics."));
    }
    let mut seen = HashSet::new();
    addresses.iter().map(|address| {
        let ip = address.parse::<Ipv4Addr>().map_err(|_| format!("Invalid IPv4 address: {address}"))?;
        if !seen.insert(ip) { return Err(format!("Duplicate address: {address}")); }
        Ok(ip)
    }).collect()
}

pub fn traffic(addresses: &[String], adapters: &[Adapter]) -> Result<Vec<AdapterTraffic>, String> {
    let requested = requested_addresses(addresses)?;
    let by_address: HashMap<&str, &Adapter> = adapters.iter()
        .flat_map(|adapter| adapter.ipv4.iter().map(move |ip| (ip.as_str(), adapter)))
        .collect();
    let mut counters = HashMap::new();
    Ok(requested.into_iter().map(|ip| {
        let address = ip.to_string();
        match by_address.get(address.as_str()) {
            None => AdapterTraffic { address, index: None, received_bytes: None, sent_bytes: None, error: Some("Adapter disconnected or IPv4 address changed.".into()) },
            Some(adapter) => {
                let result = counters.entry(adapter.index).or_insert_with(|| read_counters(adapter.index));
                match result {
                    Ok((received, sent)) => AdapterTraffic { address, index: Some(adapter.index), received_bytes: Some(*received), sent_bytes: Some(*sent), error: None },
                    Err(error) => AdapterTraffic { address, index: Some(adapter.index), received_bytes: None, sent_bytes: None, error: Some(error.clone()) },
                }
            }
        }
    }).collect())
}

pub fn probe(address: Ipv4Addr) -> Connectivity {
    let result = (|| {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        socket.bind(&SockAddr::from(SocketAddrV4::new(address, 0)))?;
        socket.connect_timeout(&SockAddr::from(PROBE_DESTINATION), Duration::from_secs(2))
    })();
    match result {
        Ok(()) => Connectivity { address: address.to_string(), reachable: true, message: "TCP connection to 1.1.1.1:443 succeeded.".into() },
        Err(error) => Connectivity { address: address.to_string(), reachable: false, message: format!("TCP connection to 1.1.1.1:443 failed: {error}") },
    }
}

#[cfg(windows)]
fn read_counters(index: u32) -> Result<(u64, u64), String> {
    use windows_sys::Win32::NetworkManagement::IpHelper::{GetIfEntry2, MIB_IF_ROW2};
    let mut row = MIB_IF_ROW2::default();
    row.InterfaceIndex = index;
    // GetIfEntry2 initializes the row's counters. Only the documented InterfaceIndex input is set.
    let status = unsafe { GetIfEntry2(&mut row) };
    if status != 0 { return Err(format!("Windows could not read adapter counters (error {status}).")); }
    Ok((row.InOctets, row.OutOctets))
}

#[cfg(not(windows))]
fn read_counters(_index: u32) -> Result<(u64, u64), String> {
    Err("Adapter traffic counters currently require Windows.".into())
}

#[cfg(test)]
mod tests {
    use super::{requested_addresses, traffic};
    use crate::Adapter;

    #[test]
    fn diagnostics_validate_addresses() {
        assert!(requested_addresses(&["1.2.3.4".into(), "1.2.3.4".into()]).is_err());
        assert!(requested_addresses(&["1.2.3.4/1".into()]).is_err());
        assert!(requested_addresses(&vec!["1.2.3.4".into(); 17]).is_err());
        assert!(requested_addresses(&[]).unwrap().is_empty());
    }

    #[test]
    fn disconnected_address_has_no_counters() {
        let adapters = vec![Adapter { name: "Test".into(), index: 1, ipv4: vec!["10.0.0.1".into()] }];
        let snapshot = traffic(&["10.0.0.2".into()], &adapters).unwrap();
        assert_eq!(snapshot[0].address, "10.0.0.2");
        assert!(snapshot[0].index.is_none());
        assert!(snapshot[0].received_bytes.is_none());
        assert!(snapshot[0].error.is_some());
    }
}
