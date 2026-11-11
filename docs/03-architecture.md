# Architecture and implementation boundary

## What is built today

```text
Tauri window (vanilla JS)
  ├─ list_adapters() ──> Rust network-interface enumeration ──> opt-in IPv4 addresses
  ├─ start_proxy(addresses) ──> validation ──> child: dispatch start --ip 127.0.0.1 --port 1080 <addresses>
  ├─ proxy_status() ──> child liveness + loopback TCP port probe
  ├─ stop_proxy() ──> terminate child
  ├─ adapter_traffic(addresses) ──> Windows GetIfEntry2 interface byte counters
  └─ check_connections(addresses) ──> on-demand TCP probe bound to each local IPv4

Steam ──> Windows default networking (unchanged by Boost)
Optional external Proxifier rule ──> 127.0.0.1:1080 ──> Dispatch ──> selected local source addresses
```

- The Rust backend owns the child process, and stops it when Boost exits normally. `BOOST_DISPATCH_PATH` is a developer override; otherwise the executable is resolved from `PATH`. The Tauri frontend has no general shell permission.
- User-selected source IPv4 addresses must still belong to a currently enumerated local adapter at start time. The backend refuses empty, duplicate, invalid, and foreign selections. This prevents command-line options from being injected via the frontend; addresses are passed as discrete arguments without a shell.
- A free-port check precedes launch. The listener status is a TCP connect probe, **not** a SOCKS handshake, an Internet health check, or confirmation of successful routing. There remains a race between port check and child binding; child exit is reported on the next status refresh.
- `stdout`/`stderr` from the child are discarded to avoid pipe blocking. This is only appropriate for a prototype; diagnostics need a privacy-conscious logging design later.
- Traffic counters use a Windows-only IP Helper API, keyed by the interface index from adapter enumeration. UI rates are deltas of cumulative interface receive/send octets over elapsed time, reset when counters go backwards or a tracked adapter disappears. All apps and protocols on the interface contribute; the values cannot attribute traffic to Steam, Dispatch or an individual IPv4 address. No traffic history is persisted.
- The on-demand connection check binds a TCP socket to each selected IPv4 and attempts `1.1.1.1:443` with a two-second timeout. This does not automatically probe in the background, fetch a URL, or prove Steam routing; it may fail when that endpoint is filtered. No global proxy, route or firewall settings are changed.
- The app does not persist selections, automatically detect phone-vs-Wi-Fi, intercept processes or modify global proxies/firewall/routes.

## Preferred evolution (decision not locked)

**Core:** Keep adapter inventory, selection, health and subprocess control separate from traffic capture. Continue to use the upstream Rust Dispatch engine while validating throughput and compatibility. Investigate upstream contribution/fork only if measurements or missing APIs justify it; do not copy code without preserving licenses.

**Steam routing options to spike:** (a) WFP-based per-process connection redirection to a local service; (b) opt-in virtual adapter with packet-to-proxy translation and careful bypass of Boost/Dispatch traffic. Compare scope, DNS/IPv6/UDP support, admin rights, signing/redistribution, cleanup on crash, other VPN interactions, and testing complexity. A system proxy setting alone must not be assumed to cover all Steam content connections.

**Diagnostics:** The local interface counters and bound-source TCP check now separate observed adapter activity from basic Internet connectivity. Process attribution and evidence of Steam reaching Dispatch are still required before claiming routing success. A real SOCKS handshake can further distinguish a live listener from a working proxy path.

## Failure and safety policy

- No silent fallback to Wi-Fi for an advertised “Steam on phones only” mode. Either fail closed for selected traffic or clearly disclose a fallback policy; decide through user testing.
- Keep the proxy on loopback by default so this tool is not an open proxy on LAN/mobile networks.
- Never overwrite Windows network settings without recording and restoring prior state; cleanup must also handle app crashes/reboots if a future mode changes system routing.
- Consider IPv6, DNS resolution path, changing local IPs on reconnect, apps opening UDP, multiple NICs with the same subnet, and phone data caps as explicit test cases.
- No third-party networking binaries or kernel drivers are bundled in this prototype. Distribution/updates require a separate security and license review.
