# Selected-application routing (experimental)

Boost can now start Dispatch and an app-specific ProxiFyre rule together. The target app does not need SOCKS or browser proxy configuration. This is the first working integration, not the completed consumer installer.

## Try it on Windows x64

1. Install Rust Dispatch as described in the README. Run Boost with `npm run tauri dev`.
2. Expand **Routing component setup** and click **Install routing component**. Accept Windows administrator approval for the driver/firewall setup. This stages the pinned ProxiFyre 2.6.4 runtime, installs Windows Packet Filter when missing, and registers an inbound TCP firewall rule for that exact routing executable. It does not enable redirection yet. A driver installation may interrupt networking briefly.
3. Select at least two different usable adapters, such as phone USB tethering and Wi-Fi.
4. Click **Choose .exe** and select the application which actually downloads content. Then click **Boost application**.
5. Start a fresh download or restart the target application to establish new connections. Boost does not launch or close the target app.
6. Click **Stop** to terminate the redirection engine and Dispatch. Existing transfers may need to reconnect. Normal app exit also stops both processes.

The source-build setup can also be run with `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/prepare-routing.ps1 -InstallNetwork`. Without `-InstallNetwork`, it only stages the engine. Setup is x64-only for this first milestone. ProxiFyre also requires .NET Framework 4.7.2+ and the Visual C++ runtime; startup will fail if these are missing. A fresh-machine bundled dependency installer is still required for a consumer release.

## Exact scope

- Rules match the executable **filename**, case-insensitively, including all instances with that name. ProxiFyre full-path rules use substring matching, so this version does not claim exact-path isolation. The picker verifies the selected file exists and is an `.exe`.
- Only IPv4 TCP is proxied to Dispatch. UDP/QUIC and DNS are not accelerated. Matching IPv6 TCP is blocked by the engine's IPv4-only rule; applications may fall back to IPv4. Do not advertise this mode as all-traffic routing.
- Helper processes with different names are not automatically included. Steam's actual downloader process must still be verified; no unverified broad Steam helper preset is shipped.
- Local-network destinations bypass interception. Boost, Dispatch (including a custom executable name), ProxiFyre, and Proxifier are excluded to avoid loops.
- Engine runs with ProxiFyre's explicit standard-user mode. Elevated, protected, or unresolved processes may remain direct. A verified privileged service is a later milestone.
- The UI reports **engine activation**, not proof of traffic capture or improved speed. Adapter byte counters still include all applications. Parallel connections can share links; one TCP stream cannot combine their bandwidth.

## Ownership and lifecycle

- Runtime: `%LOCALAPPDATA%\Boost\runtime\proxifyre-2.6.4`. `BOOST_PROXIFYRE_DIR` can select a development runtime prepared by the same script.
- A Windows exclusive file handle locks the runtime while active. Generated configuration is Boost-owned; no installed Proxifier or ProxiFyre user profile is edited.
- Readiness requires a SOCKS5 handshake from Dispatch plus the fresh engine log's app-association and engine-start confirmations. Failed startup tears down owned processes.
- A backend watchdog checks child liveness independently of UI polling. If either engine exits, the other is stopped. A Windows kill-on-job-close group additionally ties owned children to Boost's process lifetime. Stop removes interception first, then stops Dispatch.
- The bounded engine log is kept in the owned runtime for troubleshooting and reset on each run. Startup logs include executable names. No debug traffic logging is enabled by Boost.
- Driver installation and the program-scoped firewall rule persist after Stop. This is intentional first-run setup, not an active routing rule. Shared Windows Packet Filter must not be uninstalled while another product uses it.

## Dependencies and release work

The setup script pins the ProxiFyre x64 ZIP and Windows Packet Filter MSI SHA-256 values. It verifies the driver's Authenticode signature before invoking Windows Installer. ProxiFyre is AGPL-3.0, Dispatch is MIT/Apache-2.0, and the driver/runtime have their own distribution terms. No third-party binary is checked into this repository. A consumer package still needs licensing/notices, appropriate source availability, complete prerequisite acquisition, signing, uninstall/upgrade behavior and clean-machine testing.

Pinned references:
- https://github.com/wiresock/proxifyre/releases/tag/v2.6.4
- https://github.com/wiresock/proxifyre/blob/v2.6.4/docs/configuration.md
- https://github.com/wiresock/proxifyre/blob/v2.6.4/docs/installer.md

## Validation so far

2026-11-12, this Windows machine: driver 3.6.2.1 installed successfully with user approval. A `curl.exe` request without proxy configuration initially timed out until the routing engine's program-specific TCP firewall rule was added. After that, two parallel HTTPS transfers each returned HTTP 200 and 262,144 bytes. During transfer, Windows showed Dispatch-owned established sockets from both the USB phone and Wi-Fi addresses to the remote HTTPS endpoint, plus ProxiFyre-to-Dispatch loopback connections.

The full UI path was then exercised: select phone + Wi‑Fi, open the native `.exe` picker, choose the target, click **Boost application**, transfer, then **Stop**. Boost reported engine activation, both transfers returned HTTP 200 (131,072 bytes each), and connections appeared on **both** adapter addresses with no proxy settings in the client. After Stop, Dispatch and the routing engine were gone, port 1080 was free, and the connection count returned to zero. Closing the app also left no child processes.

This verifies automatic TCP redirection and two-link use, **not** a speed increase; the small transfers were deliberately rate-limited. Steam, browser QUIC, unplug/reconnect, elevated-app behavior and quantitative throughput remain untested.
