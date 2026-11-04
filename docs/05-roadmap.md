# Roadmap, risks and decisions

## Stage 0 — foundation (this repository)

Minimal desktop shell; discover IPv4 adapter addresses; choose adapters; control the external Rust Dispatch process; show honest status. No app interception. No installer. No guarantee of working or faster Steam downloads. Confirm build and tests on Windows before calling this stage complete.

## Stage 1 — evidence

Reproduce the user's Steam + Proxifier + Rust Dispatch workflow with Boost launching Dispatch. Add per-interface connectivity checks and reliable throughput measurement. Diagnose whether Steam has sufficient simultaneous downloads to benefit. Gather logs while protecting URLs/credentials and avoiding sensitive packet capture by default.

## Stage 2 — Steam without Proxifier

Time-box two narrow prototypes: application-aware connection redirection vs opt-in system tunnel. Decide using test data, Windows signing/install burden, privacy, safety, IPv6/DNS/UDP coverage and the ability to restore normal networking on failure. Make Steam routing opt-in; clearly mark unsupported operations. UI should distinguish “Dispatch listening,” “Steam routed,” and “two phones carrying data.”

## Stage 3 — usable release

Installer and code-signing plan, tested update path, licensing/notices for bundled dependencies, disconnected-phone handling, app-start/stop/reboot recovery, user-visible data-cap controls, optional interface weights, usability tests with non-experts, and support matrix. Expand to other apps only after process capture is validated.

## Principal risks / mitigations

1. **Wrong promise:** A SOCKS proxy balances connections, not a single connection. Keep terminology accurate; measure Steam behavior.
2. **Routing gap:** Steam may bypass a proxy. Prioritize routing proof and real traffic visibility over adding decorative UI.
3. **Windows network complexity:** WFP may require a signed driver; TUN needs DNS/routing/exclusion logic. Keep either behind an experimental feature until safe recovery is demonstrated.
4. **Connectivity variability:** Mobile carriers, same-tower congestion, CGNAT, tethering subnets, disk/CPU and CDN limits can obscure gains. Run controlled repeat tests.
5. **Process and port ownership:** Crashes, stale processes, port conflicts and environment PATH issues can break setup. Add robust process supervision and actionable diagnostics before release.
6. **Licensing/security:** Dispatch is MIT/Apache-2.0; other drivers/tunnels may differ. Review redistribution terms and executable provenance before bundling anything.

## Decisions open for future work

- Should Boost support Windows 10 and 11, or Windows 11 only?
- Does the desired policy allow Steam traffic to fall back to Wi-Fi on failure, or fail closed with an explicit warning?
- Is Steam the only first-party preset or are other download clients in MVP scope?
- Is a WFP-based targeted solution worth the driver cost, or is opt-in system-wide routing acceptable to users?
- Should Boost later ship a verified Dispatch binary, or require the user to install it separately?

These should be answered using the experiments in [04-validation.md](04-validation.md), not assumed from implementation convenience.
