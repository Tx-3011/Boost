#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod proxy;

use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use proxy::{ProxyState, ProxyStatus};
use serde::Serialize;
use std::net::IpAddr;
use tauri::State;

#[derive(Serialize)]
struct Adapter {
    name: String,
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
            adapters.push(Adapter { name: interface.name, ipv4 });
        }
    }
    Ok(adapters)
}

#[tauri::command]
fn list_adapters() -> Result<Vec<Adapter>, String> {
    local_adapters()
}

#[tauri::command]
fn start_proxy(addresses: Vec<String>, state: State<'_, ProxyState>) -> Result<(), String> {
    let available = local_adapters()?;
    state.start(addresses, &available.iter().flat_map(|a| a.ipv4.iter().cloned()).collect::<Vec<_>>())
}

#[tauri::command]
fn stop_proxy(state: State<'_, ProxyState>) -> Result<(), String> {
    state.stop()
}

#[tauri::command]
fn proxy_status(state: State<'_, ProxyState>) -> Result<ProxyStatus, String> {
    state.status()
}

fn main() {
    tauri::Builder::default()
        .manage(ProxyState::default())
        .invoke_handler(tauri::generate_handler![list_adapters, start_proxy, stop_proxy, proxy_status])
        .run(tauri::generate_context!())
        .expect("failed to run Boost");
}
