# Validation plan

## Reproducible baseline

Record the Windows version, Steam/Dispatch/Proxifier versions, CPU/disk, device and carrier details, phone data usage and download region. Turn off unrelated VPNs and note Wi-Fi state. Choose a large game/update; repeat trials at comparable times and avoid using only Steam's displayed instantaneous peak as the metric (decompression/disk can dominate).

## Experiments

| Trial | Configuration | What to verify |
| --- | --- | --- |
| A | One phone, Steam direct | Baseline throughput and adapter-byte counters |
| B | Two phones selected in Boost, no Proxifier | Proxy starts, but Steam is **not presumed** to use it; measure traffic to prove this |
| C | Two phones selected; Steam routed through proxy with Proxifier | Both mobile NICs carry sustained Steam download traffic; no unintended Wi-Fi download fallback |
| D | Same as C, one phone unplugged mid-download | Error/recovery behavior and effect on active connections |
| E | Same as C, IPv6 enabled, VPNs/overlapping phone subnets if available | DNS/route leaks, failures, and whether interfaces are actually selected |

At minimum collect Windows Performance Monitor or adapter byte counters per interface and process-aware connection evidence where feasible; correlate with Steam's displayed download rate and phone data counters. Use several trials, median sustained rates, and notes on jitter. Estimate uplift as `(two-phone median / one-phone median) - 1`, with test limitations stated. Compare the Steam results with a known multiconnection downloader through the SOCKS proxy to isolate Dispatch from Steam routing.

Boost now displays selected-adapter receive/send rates and bytes since selection. Click **Check connections** to test each selected source IPv4 against `1.1.1.1:443` on demand. Record each result, then repeat with the proxy on and off, with one phone unplugged, and with unrelated traffic stopped. These counters include other apps and cannot distinguish Steam traffic; collect process-aware connection evidence separately. A failed Cloudflare probe does not, by itself, prove an adapter lacks Internet access.

## Acceptance checklist for the current scaffold

- [x] `npm run tauri dev` opens the desktop window on a Windows dev machine (2026-11-04).
- [x] `cargo test --manifest-path src-tauri/Cargo.toml` passes (3 tests on 2026-11-12); `npm test` passes (3 tests).
- [ ] Connected adapters appear by name and IPv4 address; USB phones can be distinguished manually. Wi-Fi and virtual adapters appeared correctly, but no USB phone was connected for this test.
- [x] Empty, stale, and duplicate selected addresses cannot launch an unintended proxy (UI disabled when empty; stale address rejected in-app; duplicate address rejected by Rust test).
- [x] Only a loopback SOCKS listener appears on port 1080; a second start is rejected (SOCKS5 greeting also succeeded).
- [x] Stop and normal app exit terminate the Dispatch child; a busy port gives a useful error. The exit test found an orphaned child, which was fixed with explicit cleanup and retested.
- [x] Selected Wi-Fi showed live Windows receive/send rates, and its on-demand bound-source TCP check succeeded (2026-11-12). An unavailable IPv4 returned an offline result. USB-phone counters and unplug/reconnect recovery still need physical testing.
- [ ] With an external Proxifier rule, Steam network traffic uses the proxy and selected adapters (independently verify; do not infer from the UI).
- [x] Selected-app mode redirects an unconfigured IPv4 TCP app automatically: two parallel transfers produced outbound sockets on both phone and Wi-Fi, and Stop/exit removed both processes (2026-11-12; small rate-limited transfers, not a throughput gain).
- [ ] Selected-app mode on two phones with Steam: confirm the actual downloader process is matched, measure sustained throughput against a one-phone baseline, and test unplug/reconnect plus elevated processes.

## Follow-up investigation for Steam-specific routing

Identify processes opening Steam content connections, DNS lookup behavior, protocol mix, and number of concurrent connections during downloads. Prototype at least one no-Proxifier path with measurable on/off state, clear rollback, and an unplugged-phone scenario. Do not ship “one click Steam” before the routing is observable and reliable.
