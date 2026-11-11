use crate::{application::Application, process_job::ProcessJob};
use serde::Serialize;
use std::{fs, io::{Read, Seek, SeekFrom}, path::{Path, PathBuf}, process::{Child, Command, Stdio}, time::{Duration, Instant}};

#[derive(Serialize)]
pub struct Prerequisites {
    pub engine_available: bool,
    pub driver_available: bool,
    pub setup_complete: bool,
    pub message: String,
}

pub fn runtime_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("BOOST_PROXIFYRE_DIR") { return Ok(PathBuf::from(path)); }
    std::env::var_os("LOCALAPPDATA").map(|path| PathBuf::from(path).join("Boost/runtime/proxifyre-2.6.4"))
        .ok_or("Cannot locate the routing runtime. Set BOOST_PROXIFYRE_DIR.".into())
}

pub fn prerequisites() -> Prerequisites {
    let engine_available = runtime_dir().is_ok_and(|path| path.join("ProxiFyre.exe").is_file());
    let driver_available = driver_available();
    let setup_complete = runtime_dir().is_ok_and(|path| path.join(".boost-network-ready").is_file());
    let message = match (engine_available, driver_available, setup_complete) {
        (false, _, _) => "Install the routing component to enable selected-app boosting. Setup downloads the pinned engine and requests administrator approval for its network driver and firewall rule.",
        (true, false, _) => "Routing engine found. Install the routing component to add Windows Packet Filter (NDISRD).",
        (true, true, false) => "Engine and driver found. Complete routing component setup to register the engine's firewall rule.",
        (true, true, true) => "Routing components found. Start will verify engine activation. Standard-user mode cannot reliably route elevated or protected apps.",
    };
    Prerequisites { engine_available, driver_available, setup_complete, message: message.into() }
}

#[cfg(windows)]
fn driver_available() -> bool {
    let mut command = Command::new("sc.exe");
    hide_window(&mut command);
    command.args(["query", "ndisrd"]).output().is_ok_and(|out| out.status.success())
}
#[cfg(not(windows))]
fn driver_available() -> bool { false }

pub fn hide_window(command: &mut Command) {
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    #[cfg(not(windows))] let _ = command;
}

pub fn install() -> Result<Prerequisites, String> {
    #[cfg(not(windows))] return Err("Application routing setup requires Windows.".into());
    #[cfg(windows)] {
        let runtime = runtime_dir()?;
        let parent = runtime.parent().ok_or("Invalid runtime directory")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let script = parent.join("prepare-routing.ps1");
        fs::write(&script, include_str!("../../scripts/prepare-routing.ps1")).map_err(|e| e.to_string())?;
        let mut command = Command::new("powershell.exe");
        hide_window(&mut command);
        let output = command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(&script).arg("-Destination").arg(runtime).arg("-InstallNetwork")
            .output().map_err(|e| format!("Could not start component setup: {e}"))?;
        if !output.status.success() {
            return Err(format!("Routing setup failed: {}", String::from_utf8_lossy(&output.stderr).chars().take(1800).collect::<String>()));
        }
        Ok(prerequisites())
    }
}

pub fn configuration(application: &Application, dispatch_name: &str) -> serde_json::Value {
    // Filename matching is exact in this version of ProxiFyre. Full paths use
    // substring matching, so explicitly document that all same-name instances match.
    serde_json::json!({
        "logLevel": "Info", "bypassLan": true,
        "proxies": [{
            "appNames": [application.name], "socks5ProxyEndpoint": "127.0.0.1:1080",
            "socks5Transport": "TCP", "supportedProtocols": ["TCP"],
            "supportedAddressFamilies": ["IPv4"]
        }],
        "excludes": ["boost.exe", "dispatch.exe", dispatch_name, "ProxiFyre.exe", "Proxifier.exe"]
    })
}

pub struct Router {
    pub child: Child,
    directory: PathBuf,
    _lock: fs::File,
}

impl Router {
    pub fn start(application: &Application, dispatch_name: &str, job: &ProcessJob) -> Result<Self, String> {
        let directory = runtime_dir()?;
        if fs::read_to_string(directory.join(".boost-managed")).unwrap_or_default().trim() != "proxifyre-2.6.4-x64" {
            return Err("Routing runtime is not managed by Boost. Complete routing component setup first.".into());
        }
        let mut options = fs::OpenOptions::new();
        options.create(true).truncate(false).write(true);
        #[cfg(windows)] {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0);
        }
        let lock = options.open(directory.join(".boost-session.lock"))
            .map_err(|e| format!("Another Boost session may be using this routing engine: {e}"))?;
        let result = (|| {
            fs::write(directory.join("engine.log"), b"").map_err(|e| e.to_string())?;
            fs::write(directory.join("app-config.json"), serde_json::to_vec_pretty(&configuration(application, dispatch_name)).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            // Fresh, bounded, lifecycle-only log. Never reuse another program's config.
            fs::write(directory.join("NLog.config"), r#"<nlog xmlns="http://www.nlog-project.org/schemas/NLog.xsd" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><targets><target name="file" xsi:type="File" fileName="${basedir}/engine.log" archiveAboveSize="1048576" maxArchiveFiles="1" /></targets><rules><logger name="*" minlevel="Info" writeTo="file" /></rules></nlog>"#).map_err(|e| e.to_string())?;
            let mut command = Command::new(directory.join("ProxiFyre.exe"));
            hide_window(&mut command);
            let mut child = command.arg("--allow-not-admin").current_dir(&directory)
                .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
                .spawn().map_err(|e| format!("Could not launch routing engine: {e}"))?;
            if let Err(error) = job.assign(&child) { let _ = child.kill(); let _ = child.wait(); return Err(error); }
            Ok(child)
        })();
        let child = result?;
        let mut router = Self { child, directory, _lock: lock };
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            if let Some(exit) = router.child.try_wait().map_err(|e| e.to_string())? {
                return Err(format!("Routing engine exited ({exit}). Check Windows Packet Filter and Visual C++ prerequisites; standard-user driver access may be denied."));
            }
            let log = read_log(&router.directory.join("engine.log"));
            if engine_ready(&log, &application.name) { return Ok(router); }
            if Instant::now() >= deadline { return Err("Routing engine did not confirm app-rule activation within 12 seconds. Startup was rolled back. Check Windows Packet Filter access and engine compatibility.".into()); }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.child.try_wait().map_err(|e| e.to_string())?.is_none() {
            self.child.kill().map_err(|e| format!("Cannot stop routing engine: {e}"))?;
        }
        self.child.wait().map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Drop for Router {
    fn drop(&mut self) {
        if self.stop().is_ok() { let _ = fs::remove_file(self.directory.join("app-config.json")); }
    }
}

fn read_log(path: &Path) -> String {
    let Ok(mut file) = fs::File::open(path) else { return String::new() };
    let length = file.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = file.seek(SeekFrom::Start(length.saturating_sub(65536)));
    let mut bytes = Vec::new();
    let _ = file.take(65536).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

fn engine_ready(log: &str, app: &str) -> bool {
    log.contains("ProxiFyre Service is running...") && log.contains(&format!("Successfully associated {app} to 127.0.0.1:1080"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rules_are_app_specific_tcp_only_with_loop_exclusions() {
        let app = Application { path: "C:\\Games\\steam.exe".into(), name: "steam.exe".into() };
        let config = configuration(&app, "custom-dispatch.exe");
        assert_eq!(config["proxies"][0]["appNames"], serde_json::json!(["steam.exe"]));
        assert_eq!(config["proxies"][0]["supportedProtocols"], serde_json::json!(["TCP"]));
        assert_eq!(config["proxies"][0]["supportedAddressFamilies"], serde_json::json!(["IPv4"]));
        assert!(config["excludes"].as_array().unwrap().contains(&serde_json::json!("custom-dispatch.exe")));
    }
    #[test]
    fn readiness_requires_both_rule_and_engine_confirmation() {
        assert!(!engine_ready("ProxiFyre Service is running...", "steam.exe"));
        let log = "Successfully associated steam.exe to 127.0.0.1:1080 SOCKS5 proxy\nProxiFyre Service is running...";
        assert!(engine_ready(log, "steam.exe"));
        assert!(!engine_ready(log, "chrome.exe"));
    }
}
