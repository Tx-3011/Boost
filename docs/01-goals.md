# Goals and product definition

## Context

The original workflow uses 2–3 phones connected to a Windows laptop through USB tethering. The user tested the **Rust** `alexkirsz/dispatch` proxy and found Steam did not benefit until Steam traffic was sent through that proxy using Proxifier. Boost aims to remove setup friction without implying that selecting adapters alone will affect every application.

## User-centered problem statement

A Windows user with multiple USB-tethered connections needs an understandable way to route Steam downloads over those connections, because the balancing proxy only helps an application when that application's network connections actually reach it.

## Intended experience (target, not current functionality)

1. Connect phones and turn on tethering in each phone's OS.
2. Boost identifies available adapters, shows enough detail to distinguish phones from Wi-Fi/VPNs, and lets the user opt in to connections.
3. Enable a Steam-specific routing mode without separately configuring Proxifier.
4. Show whether traffic is reaching the local proxy and which connection is being used; warn about failures, caps, and fallback behavior.
5. Disable Boost and restore the previous network state cleanly.

## Non-goals for initial releases

- No claim that *one TCP connection* uses the sum of all phones' bandwidth. Dispatch assigns a connection to one outbound address; its benefit comes from concurrent connections.
- No remote VPN/server or subscription infrastructure.
- No automatic USB tethering activation on the phones; users enable it themselves.
- No silent global routing changes, stealth VPN, or blanket guarantee for games and UDP traffic.
- No requirement to replace Dispatch's Rust implementation before measuring it.

## MVP outcome and measurable gates

**Prototype gate:** With two independent, usable connections, Boost can start/stop the local Dispatch proxy, reject unavailable selections, and remain clear that Steam is not yet routed. Verify no unrelated Windows routes or proxy settings are changed.

**Steam MVP gate:** On a documented Windows test setup, show concurrent Steam download traffic over at least two selected mobile adapters when routing is enabled, and little/no Steam download traffic on an excluded Wi-Fi connection. Compare with a one-phone baseline on the same download/region/time window; report observed speeds rather than promising a fixed uplift.

**Reliability gate:** Removing one tether, restarting Steam, exiting Boost, and encountering a busy proxy port must each produce an understandable result without a persistent broken proxy or unintended route change. Record DNS and IPv6 behavior; failing closed vs falling back must be a user-visible choice, not an accidental side effect.

## Vocabulary for honest copy

**Connection balancing:** separate application connections can use separate adapters. **True bonding:** data for one logical stream is split and reassembled across paths, generally with a cooperating remote endpoint. **Local SOCKS proxy:** an endpoint applications can use if configured or intercepted. **Adapter selected:** permission to use that source address, not evidence that application traffic actually used it.
