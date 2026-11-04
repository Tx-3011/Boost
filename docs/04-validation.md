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

## Acceptance checklist for the current scaffold

- [ ] `npm run tauri dev` opens the desktop window on a Windows dev machine (the debug executable builds; the UI has not been launched).
- [x] `cargo test --manifest-path src-tauri/Cargo.toml` passes (1 test on 2026-11-04).
- [ ] Connected adapters appear by name and IPv4 address; USB phones can be distinguished manually.
- [ ] Empty, stale, and duplicate selected addresses cannot launch an unintended proxy.
- [ ] Only a loopback SOCKS listener appears on port 1080; a second start is rejected.
- [ ] Stop and normal app exit terminate the Dispatch child; a busy port gives a useful error.
- [ ] With an external Proxifier rule, Steam network traffic uses the proxy and selected adapters (independently verify; do not infer from the UI).

## Follow-up investigation for Steam-specific routing

Identify processes opening Steam content connections, DNS lookup behavior, protocol mix, and number of concurrent connections during downloads. Prototype at least one no-Proxifier path with measurable on/off state, clear rollback, and an unplugged-phone scenario. Do not ship “one click Steam” before the routing is observable and reliable.
