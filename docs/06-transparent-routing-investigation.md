# Transparent routing investigation

Investigated 2026-11-12. Documentation/source review plus read-only local inspection; no routing changes or driver installation performed.

## Findings

1. The installed Dispatch process currently has both USB-phone and Wi-Fi source addresses in its command line. Proxifier.exe exists in its standard installation directory, but no Proxifier process was observed. This supports the missing-redirection diagnosis, not proof of the browser's actual path.
2. Proxifier Standard v4 explicitly uses a Windows Filtering Platform callout driver. Its documented scope is system-wide or selected TCP connections. This is the missing layer in the previous working setup, not bandwidth bonding or a remote relay.
3. Dispatch's current upstream `src/socks.rs` accepts SOCKS5 CONNECT and rejects other commands, including UDP ASSOCIATE. DNS for domain requests uses system lookup. IPv6 targets require suitable selected IPv6 source addresses; Boost currently selects IPv4 only. These source observations should be confirmed against the exact installed binary before release.
4. Proxifier supports loading XML profiles through `Proxifier.exe profile.ppx silent-load`. This offers a short path to reproduce the known workflow under Boost control, but retains the commercial Proxifier installation/license dependency. Loading a profile changes an existing instance too; preserve and restore the user's configuration rather than overwriting it.
5. ProxiFyre is an existing transparent Windows SOCKS5 redirection candidate. Current upstream documentation describes process rules, a catch-all rule, exclusions, service control, TCP/UDP and IPv6. It uses Windows Packet Filter and is AGPL-3.0; the ndisapi repository is MIT, while exact distributed driver/runtime artifacts need separate review. Its UDP capability cannot make Dispatch support UDP. Do not enable its default TCP+UDP configuration against Dispatch.
6. A custom WFP redirector needs a driver and a user-space bridge that recovers the original destination and performs SOCKS negotiation. Simply redirecting raw application bytes to Dispatch's SOCKS port will not work. Exclude Dispatch's outbound connections to prevent recursive interception.

## Recommendation

Do not assume a TUN architecture is necessary. First reproduce transparent TCP balancing with the already-installed Proxifier and existing Dispatch. Test a browser and Steam separately, inspect proxifier connection records and Dispatch outbound source addresses, and correlate with both adapter counters. Existing connections may need reconnecting before newly enabled interception affects them.

For a standalone Boost implementation, evaluate ProxiFyre as the first reusable interception candidate, subject to its license and exact release prerequisites. A custom WFP driver is a substantially larger fallback, not the first implementation step. TUN remains an alternative if packet-level coverage is required.

The first honest milestone is automatic **TCP** redirection and local multi-connection balancing. The user's ultimate all-PC goal additionally needs explicit UDP/QUIC, DNS and IPv6 behavior; those protocols must not silently be advertised as accelerated by Dispatch. A browser test must record which protocol it actually used rather than assume Fast.com uses TCP or QUIC. No fixed speed multiplier is established.

## Proposed validation experiment

- Preserve current Proxifier profile and rule state before making changes.
- Configure SOCKS5 at 127.0.0.1:1080; place Direct exclusions for Dispatch and the routing engine ahead of any catch-all rule, and preserve appropriate local-network bypasses.
- Initially route only the test browser and Steam for controlled attribution, then evaluate catch-all TCP rules toward the all-PC goal.
- Compare each uplink alone and both together using fresh parallel TCP transfers. Observe intercepted application connections, Dispatch outbound local addresses, and sustained per-adapter rates. Interface totals alone are insufficient attribution.
- Verify Stop, app exit, proxy failure and unplug/reconnect behavior; restore the original profile after the experiment. Do not stop the user's currently running Dispatch as incidental cleanup.

## Primary sources

- Proxifier scope: https://www.proxifier.com/docs/win-v4/
- WFP driver: https://www.proxifier.com/docs/win-v4/install.html
- Profile automation: https://www.proxifier.com/docs/win-v4/profiles.html
- Loop prevention: https://www.proxifier.com/docs/win-v4/loop.html
- Dispatch protocol handling: https://github.com/alexkirsz/dispatch/blob/main/src/socks.rs
- Microsoft WFP redirection contract: https://learn.microsoft.com/en-us/windows-hardware/drivers/network/using-bind-or-connect-redirection
- ProxiFyre: https://github.com/wiresock/proxifyre
- ProxiFyre configuration: https://github.com/wiresock/proxifyre/blob/main/docs/configuration.md
- ndisapi license: https://github.com/wiresock/ndisapi/blob/master/LICENSE

Upstream main-branch documents are mutable. Pin versions/commits and verify binaries before implementing or distributing an integration.
