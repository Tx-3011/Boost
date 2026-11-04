use serde::Serialize;
use std::{collections::HashSet, net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream}, process::{Child, Command, Stdio}, sync::Mutex, time::Duration};

const PROXY_PORT: u16 = 1080;

#[derive(Default)]
pub struct ProxyState {
    child: Mutex<Option<Child>>,
}

#[derive(Serialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub ready: bool,
    pub message: Option<String>,
}

pub fn validate_addresses(addresses: &[String], available: &[String]) -> Result<(), String> {
    if addresses.is_empty() { return Err("Select at least one adapter.".into()); }
    let valid: HashSet<&str> = available.iter().map(String::as_str).collect();
    let mut seen = HashSet::new();
    for address in addresses {
        let parsed: IpAddr = address.parse().map_err(|_| format!("Invalid IP address: {address}"))?;
        if !matches!(parsed, IpAddr::V4(_)) || !valid.contains(address.as_str()) {
            return Err(format!("Address is not on an available IPv4 adapter: {address}"));
        }
        if !seen.insert(address) { return Err(format!("Duplicate address: {address}")); }
    }
    Ok(())
}

impl ProxyState {
    pub fn start(&self, addresses: Vec<String>, available: &[String]) -> Result<(), String> {
        validate_addresses(&addresses, available)?;
        let mut child = self.child.lock().map_err(|_| "Proxy state unavailable")?;
        if let Some(process) = child.as_mut() {
            if process.try_wait().map_err(|e| e.to_string())?.is_none() {
                return Err("Dispatch is already running under Boost.".into());
            }
        }
        *child = None;
        TcpListener::bind((Ipv4Addr::LOCALHOST, PROXY_PORT))
            .map_err(|e| format!("Local port {PROXY_PORT} is unavailable: {e}"))?;
        let binary = std::env::var_os("BOOST_DISPATCH_PATH").unwrap_or_else(|| "dispatch".into());
        let process = Command::new(binary)
            .arg("start").args(["--ip", "127.0.0.1", "--port", "1080"])
            .args(&addresses)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().map_err(|e| format!("Could not launch dispatch; install it or set BOOST_DISPATCH_PATH: {e}"))?;
        *child = Some(process);
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut child = self.child.lock().map_err(|_| "Proxy state unavailable")?;
        if let Some(mut process) = child.take() {
            if process.try_wait().map_err(|e| e.to_string())?.is_none() {
                process.kill().map_err(|e| format!("Could not stop dispatch: {e}"))?;
                process.wait().map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn status(&self) -> Result<ProxyStatus, String> {
        let mut child = self.child.lock().map_err(|_| "Proxy state unavailable")?;
        if let Some(process) = child.as_mut() {
            if let Some(exit) = process.try_wait().map_err(|e| e.to_string())? {
                *child = None;
                return Ok(ProxyStatus { running: false, ready: false, message: Some(format!("Dispatch exited: {exit}. Check your adapter selection and dispatch logs.")) });
            }
            let address = SocketAddr::from((Ipv4Addr::LOCALHOST, PROXY_PORT));
            let ready = TcpStream::connect_timeout(&address, Duration::from_millis(200)).is_ok();
            return Ok(ProxyStatus { running: true, ready, message: None });
        }
        Ok(ProxyStatus { running: false, ready: false, message: None })
    }
}

impl Drop for ProxyState {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            if let Some(mut process) = child.take() {
                if matches!(process.try_wait(), Ok(None)) {
                    let _ = process.kill();
                    let _ = process.wait();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_addresses;

    #[test]
    fn rejects_absent_and_unselected_addresses() {
        let available = vec!["192.168.42.1".to_string()];
        assert!(validate_addresses(&[], &available).is_err());
        assert!(validate_addresses(&["8.8.8.8".into()], &available).is_err());
        assert!(validate_addresses(&["192.168.42.1/3".into()], &available).is_err());
        assert!(validate_addresses(&["192.168.42.1".into(), "192.168.42.1".into()], &available).is_err());
        assert!(validate_addresses(&["192.168.42.1".into()], &available).is_ok());
    }
}
