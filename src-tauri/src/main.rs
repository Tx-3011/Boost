#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod diagnostics;
mod application;
mod process_job;
mod proxy;
mod routing;

use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use proxy::{ProxyState, ProxyStatus};
use serde::Serialize;
use std::net::IpAddr;
use tauri::{Manager, State};

#[derive(Serialize)]
struct Adapter {
    name: String,
    index: u32,
    ipv4: Vec<String>,
}

fn local_adapters() -> Result<Vec<Adapter>, String> {
    let interfaces = NetworkInterface::show().map_err(|error| error.to_string())?;
    let mut adapters = Vec::new();
    for interface in interfaces {
        let ipv4: Vec<String> = interface.addr.iter().filter_map(|address| match address.ip() {
            IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_link_local() => Some(ip.to_string()),
            _ => None,
        }).collect();
        if !ipv4.is_empty() {
            adapters.push(Adapter { name: interface.name, index: interface.index, ipv4 });
        }
    }
    Ok(adapters)
}

#[tauri::command]
fn list_adapters() -> Result<Vec<Adapter>, String> {
    local_adapters()
}

#[tauri::command]
async fn start_proxy(addresses: Vec<String>, state: State<'_, ProxyState>) -> Result<(), String> {
    let available = local_adapters()?;
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.start(addresses, &available.iter().flat_map(|a| a.ipv4.iter().cloned()).collect::<Vec<_>>()))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn choose_application() -> Result<Option<application::Application>, String> {
    tauri::async_runtime::spawn_blocking(application::choose).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn routing_prerequisites() -> Result<routing::Prerequisites, String> {
    tauri::async_runtime::spawn_blocking(routing::prerequisites).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_routing(state: State<'_, ProxyState>) -> Result<routing::Prerequisites, String> {
    if state.status()?.running { return Err("Stop Boost before installing routing components.".into()); }
    tauri::async_runtime::spawn_blocking(routing::install).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn start_application(addresses: Vec<String>, path: String, state: State<'_, ProxyState>) -> Result<(), String> {
    let application = application::validate(std::path::Path::new(&path))?;
    let available = local_adapters()?;
    let selected: std::collections::HashSet<_> = available.iter()
        .filter(|adapter| adapter.ipv4.iter().any(|ip| addresses.contains(ip))).map(|adapter| adapter.index).collect();
    if selected.len() < 2 { return Err("Select at least two different network adapters to boost an application.".into()); }
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.start_application(addresses, &available.iter().flat_map(|a| a.ipv4.iter().cloned()).collect::<Vec<_>>(), application))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn stop_proxy(state: State<'_, ProxyState>) -> Result<(), String> {
    state.stop()
}

#[tauri::command]
fn proxy_status(state: State<'_, ProxyState>) -> Result<ProxyStatus, String> {
    state.status()
}

#[tauri::command]
fn adapter_traffic(addresses: Vec<String>) -> Result<Vec<diagnostics::AdapterTraffic>, String> {
    diagnostics::traffic(&addresses, &local_adapters()?)
}

#[tauri::command]
async fn check_connections(addresses: Vec<String>) -> Result<Vec<diagnostics::Connectivity>, String> {
    let requested = diagnostics::requested_addresses(&addresses)?;
    let available = local_adapters()?;
    let current: std::collections::HashSet<_> = available.iter()
        .flat_map(|a| a.ipv4.iter().filter_map(|ip| ip.parse::<std::net::Ipv4Addr>().ok()))
        .collect();
    let checks: Vec<_> = requested.into_iter().map(|address| {
        if !current.contains(&address) {
            return None;
        }
        Some(tauri::async_runtime::spawn_blocking(move || diagnostics::probe(address)))
    }).collect();
    let mut results = Vec::new();
    for (address, check) in addresses.into_iter().zip(checks) {
        results.push(match check {
            Some(check) => check.await.map_err(|error| error.to_string())?,
            None => diagnostics::Connectivity { address, reachable: false, message: "Adapter disconnected or IPv4 address changed.".into() },
        });
    }
    Ok(results)
}

fn main() {
    let proxy = ProxyState::default();
    proxy.supervise();
    tauri::Builder::default()
        .manage(proxy)
        .invoke_handler(tauri::generate_handler![list_adapters, start_proxy, stop_proxy, proxy_status, adapter_traffic, check_connections, choose_application, routing_prerequisites, install_routing, start_application])
        .build(tauri::generate_context!())
        .expect("failed to build Boost")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Err(error) = app.state::<ProxyState>().stop() {
                    eprintln!("Could not stop Dispatch on exit: {error}");
                }
            }
        });
}
