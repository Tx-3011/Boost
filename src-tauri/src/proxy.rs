use crate::{application::Application, process_job::ProcessJob, routing::{self, Router}};
use serde::Serialize;
use std::{collections::HashSet, io::{Read, Write}, net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream}, path::Path, process::{Child, Command, Stdio}, sync::{Arc, Mutex}, time::{Duration, Instant}};

const PROXY_PORT: u16 = 1080;

#[derive(Default, Clone)]
pub struct ProxyState {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    session: Option<Session>,
    message: Option<String>,
}

struct Session {
    child: Child,
    router: Option<Router>,
    addresses: Vec<String>,
    application: Option<Application>,
    _job: ProcessJob,
}

impl Session {
    fn stop(&mut self) -> Result<(), String> {
        // Remove interception before removing the proxy it depends on.
        if let Some(router) = &mut self.router { router.stop()?; }
        self.router = None;
        if self.child.try_wait().map_err(|e| e.to_string())?.is_none() {
            self.child.kill().map_err(|e| format!("Could not stop Dispatch: {e}"))?;
        }
        self.child.wait().map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Drop for Session {
    fn drop(&mut self) { let _ = self.stop(); }
}

#[derive(Serialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub ready: bool,
    pub routing: bool,
    pub addresses: Vec<String>,
    pub application: Option<Application>,
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

fn socks_ready() -> bool {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, PROXY_PORT));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(200)) else { return false };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(200)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(200)));
    let mut answer = [0; 2];
    stream.write_all(&[5, 1, 0]).is_ok() && stream.read_exact(&mut answer).is_ok() && answer == [5, 0]
}

impl ProxyState {
    pub fn start(&self, addresses: Vec<String>, available: &[String]) -> Result<(), String> {
        self.start_session(addresses, available, None)
    }

    pub fn start_application(&self, addresses: Vec<String>, available: &[String], application: Application) -> Result<(), String> {
        let prereqs = routing::prerequisites();
        if !prereqs.engine_available || !prereqs.driver_available || !prereqs.setup_complete { return Err(prereqs.message); }
        self.start_session(addresses, available, Some(application))
    }

    fn start_session(&self, addresses: Vec<String>, available: &[String], application: Option<Application>) -> Result<(), String> {
        validate_addresses(&addresses, available)?;
        let mut inner = self.inner.lock().map_err(|_| "Proxy state unavailable")?;
        if inner.session.is_some() { return Err("Boost is already running. Stop it before changing the application or connections.".into()); }
        TcpListener::bind((Ipv4Addr::LOCALHOST, PROXY_PORT))
            .map_err(|e| format!("Local port {PROXY_PORT} is unavailable. Stop the other proxy first: {e}"))?;
        let binary = std::env::var_os("BOOST_DISPATCH_PATH").unwrap_or_else(|| "dispatch".into());
        let dispatch_name = Path::new(&binary).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "dispatch.exe".into());
        let job = ProcessJob::new()?;
        let mut command = Command::new(binary);
        routing::hide_window(&mut command);
        let mut child = command.arg("start").args(["--ip", "127.0.0.1", "--port", "1080"])
            .args(&addresses).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().map_err(|e| format!("Could not launch Dispatch; install it or set BOOST_DISPATCH_PATH: {e}"))?;
        if let Err(error) = job.assign(&child) { let _ = child.kill(); let _ = child.wait(); return Err(error); }
        let mut session = Session { child, router: None, addresses, application, _job: job };
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(exit) = session.child.try_wait().map_err(|e| e.to_string())? { return Err(format!("Dispatch exited during startup: {exit}")); }
            if socks_ready() { break; }
            if Instant::now() >= deadline { return Err("Dispatch did not provide a SOCKS5 listener within five seconds. Startup was rolled back.".into()); }
            std::thread::sleep(Duration::from_millis(100));
        }
        if let Some(application) = &session.application {
            session.router = Some(Router::start(application, &dispatch_name, &session._job)?);
        }
        inner.session = Some(session);
        inner.message = None;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().map_err(|_| "Proxy state unavailable")?;
        if let Some(session) = &mut inner.session { session.stop()?; }
        inner.session = None;
        inner.message = None;
        Ok(())
    }

    pub fn status(&self) -> Result<ProxyStatus, String> {
        let mut inner = self.inner.lock().map_err(|_| "Proxy state unavailable")?;
        let mut failure = None;
        if let Some(session) = &mut inner.session {
            if let Some(exit) = session.child.try_wait().map_err(|e| e.to_string())? {
                failure = Some(format!("Dispatch exited ({exit}). Application routing has been stopped."));
            }
            if let Some(router) = &mut session.router {
                if let Some(exit) = router.child.try_wait().map_err(|e| e.to_string())? {
                    failure = Some(format!("Application routing engine exited ({exit}). Dispatch has been stopped."));
                }
            }
        }
        if let Some(message) = failure {
            // Dropping the session closes the job as a last resort if normal cleanup fails.
            inner.session = None;
            inner.message = Some(message);
        }
        Ok(match &inner.session {
            Some(session) => ProxyStatus {
                running: true, ready: true, routing: session.router.is_some(),
                addresses: session.addresses.clone(), application: session.application.clone(), message: None,
            },
            None => ProxyStatus { running: false, ready: false, routing: false, addresses: vec![], application: None, message: inner.message.clone() },
        })
    }

    pub fn supervise(&self) {
        let weak = Arc::downgrade(&self.inner);
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            let Some(inner) = weak.upgrade() else { break };
            let _ = Self { inner }.status();
        });
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
