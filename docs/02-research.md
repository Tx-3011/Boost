# Research notes and evidence

Reviewed 2026-11-04. Sources below are primary documentation or upstream source/README where possible. These are architectural inputs, **not** proof that Boost works with current Steam builds. Distinguish observed user experience, upstream statements, and untested hypotheses.

## Confirmed from upstream

### Rust Dispatch, not Connectify Dispatch

- [`alexkirsz/dispatch`](https://github.com/alexkirsz/dispatch) is the Rust rewrite of the archived CoffeeScript/Node [`dispatch-proxy`](https://github.com/alexkirsz/dispatch-proxy). Its README says it works on Windows, macOS, and Linux and describes a **SOCKS** proxy that chooses an outbound local address per connection using weighted round robin. It documents `dispatch list` and `dispatch start <address>...`, plus `--ip` and `--port` (`127.0.0.1`, `1080` defaults).
- The [upstream CLI source](https://github.com/alexkirsz/dispatch/blob/main/src/main.rs) confirms these argument names. The [upstream address logic](https://github.com/alexkirsz/dispatch/blob/main/src/net.rs) filters loopback/link-local addresses and checks that it can bind a socket to an address. Boost's adapter discovery is intentionally simpler and does **not** claim the address has Internet access.
- Upstream is dual-licensed **MIT OR Apache-2.0** (see [licenses and repository](https://github.com/alexkirsz/dispatch)). Validate exact dependency/release licensing and provide notices before redistributing binaries. Boost currently expects a separately installed Dispatch executable; no binary is bundled.
- The separately named proprietary Connectify Dispatch was declared [end-of-life with Connectify Hotspot 2019](https://connectify.me/new-release/). It is not a Boost dependency.
- The authoring machine has an installed `dispatch.exe` reporting `dispatch-proxy 0.2.4`; this confirms the executable is present there but does **not** establish that Boost's Rust backend builds or that Steam traffic is routed.

### Windows application routing

- Microsoft documents [WFP bind/connect redirection](https://learn.microsoft.com/en-us/windows-hardware/drivers/network/using-bind-or-connect-redirection): a callout driver can intercept application connection requests and redirect them to a proxy service. Microsoft notes TCP/UDP-related cases and requirements for avoiding redirect loops and preserving redirect records. This is **not** a trivial all-Rust user-mode checkbox; plan for driver engineering, signing, permissions, compatibility, and review before choosing it.
- [Wintun](https://github.com/WireGuard/wintun) is a Windows layer-3 TUN adapter intended for userspace tunneling. It is a candidate for system-wide capture with an additional userspace packet-to-proxy bridge, not by itself a Steam-specific router. Source and prebuilt binary licenses differ; read their redistribution terms before adopting. Tunnel routing, DNS, IPv6 and exclusion of the proxy's own outbound connections all require design and testing.
- [Steam's official required-ports documentation](https://help.steampowered.com/en/faqs/view/2EA8-4D75-DA21-31EB) describes TCP HTTP use for login/content and other port/protocol requirements. It does not guarantee all Steam network behavior uses SOCKS, nor tell us which Steam processes open download sockets.

### Limits of single-stream aggregation

- [RFC 8684 (Multipath TCP)](https://www.rfc-editor.org/rfc/rfc8684) explains the multiple subflow model. It is a distinct architecture from Dispatch's per-connection selection. To accelerate arbitrary remote services' single ordinary TCP flows without their direct participation, a cooperating remote endpoint/tunnel service would typically be required. Open-source [OpenMPTCProuter](https://github.com/ysurac/openmptcprouter) uses a router and VPS architecture as an example, not a suggested Boost MVP dependency.

## User-reported observation (not yet reproduced in this repository)

- On the user's earlier setup, Steam did not benefit from Rust Dispatch until Steam was added to Proxifier. Most likely, Steam traffic had not reached the local proxy. This is an inference to validate by inspecting connections, not proof of Steam internals or a universal limitation of Steam.
- The user observed a meaningful speed improvement with multiple tethered phones. No controlled baseline, download-region data, per-adapter traffic trace, or exact speed figures are recorded yet.

## Hypotheses requiring experiments

1. A Steam download creates enough concurrent proxied connections for per-connection balancing to improve throughput on the user's current network.
2. The Proxifier rule catches the actual download process(es), not merely `steam.exe` UI traffic. Identify processes with telemetry rather than hardcoding names.
3. Binding an outbound socket to each chosen local address reliably selects the intended USB tether on the user's Windows routing configuration (particularly with overlapping tether subnets, VPNs, and IPv6).
4. Intercepting Steam without Proxifier can be done safely and maintainably through either a narrow Windows interception layer or an opt-in TUN approach. Neither is implemented or selected.
5. Multiple phones on one cell tower may contend for the same radio capacity; results depend on signal, carriers, data caps, USB bus bandwidth, Steam server/CDN, decompression, and disk writes.

## Research hygiene

When results differ from this document, record OS build, Steam client version, Dispatch version/commit, device models/carriers, download region, and timestamps in a test log. Avoid treating third-party proxy tutorials as authoritative for current Steam behavior.
