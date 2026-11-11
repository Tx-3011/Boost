# Boost

Boost is a Windows-first desktop prototype for selecting multiple USB-tethered network adapters and starting the **modern Rust `dispatch` SOCKS proxy** on `127.0.0.1:1080`. It is intended to grow into a simple Steam-download workflow.

> **Current scope:** Experimental selected-application IPv4 TCP routing through Dispatch, adapter discovery/selection, live adapter byte counters, and on-demand connection checks. Boost does **not** guarantee a speed increase, accelerate UDP/QUIC, or provide single-flow bonding. There is no packaged consumer installer yet. See [selected-application setup and limitations](docs/07-selected-app-routing.md).

## Run from source (Windows)

1. Install the [Rust toolchain](https://rustup.rs/) with the MSVC target, [Tauri's Windows prerequisites](https://v2.tauri.app/start/prerequisites/) (including WebView2 and C++ Build Tools), and Node.js/npm.
2. Install the **Rust rewrite** of Dispatch from [alexkirsz/dispatch releases](https://github.com/alexkirsz/dispatch/releases) or `cargo install dispatch-proxy`. Confirm `dispatch --version` works. Boost invokes `dispatch` from `PATH`; alternatively set the `BOOST_DISPATCH_PATH` environment variable to the full path to `dispatch.exe` **before** launching Boost.
3. Connect phones, enable USB tethering, and confirm each phone creates a working network adapter in Windows. In the project folder run:

   ```powershell
   npm install
   npm run tauri dev
   ```

4. Select the desired IPv4 addresses (check Windows adapter details if unsure), then click **Start proxy**. Boost does not change Windows default routes. The port is intentionally bound to localhost; other devices cannot connect to it.
5. For a Steam experiment, configure your own Proxifier rule to send Steam's relevant connections to `127.0.0.1:1080` over SOCKS5, then verify the actual adapter traffic. Proxifier is not shipped, managed, or required to start the Boost prototype.

**Automatic selected-app mode:** Instead of the manual proxy workflow above, install the routing component from Boost's setup panel, select two adapters, choose an `.exe`, and click **Boost application**. Boost manages ProxiFyre redirection and Dispatch together; the target app needs no proxy settings. See [the full setup guide](docs/07-selected-app-routing.md). Use **Start proxy only** only for manual/external-proxifier tests.

### Connection diagnostics

Select adapters to see their Windows interface receive/send rates and bytes since monitoring began. These counters include **all** traffic on each adapter (including other apps and the connection check); multiple IPv4 addresses on one adapter share the same counters. A nonzero rate is not proof of Steam or Dispatch use. No packet contents, URLs, or traffic history are stored by Boost.

**Check connections** is an explicit, on-demand TCP connection attempt to `1.1.1.1:443` (Cloudflare) bound to each selected IPv4 address, with a two-second timeout. It sends no application data and uses a small amount of network traffic; it does not run automatically. A success shows that address can open a TCP connection to this endpoint, not that Steam is routed or that every destination works. A failure can also mean the endpoint is blocked. To prove Steam traffic, use the controlled trials in [the validation plan](docs/04-validation.md) and independent process-aware connection evidence.

`npm run build` builds only the web frontend; `npm run tauri dev` compiles and runs the desktop application. Run `npm test` and `cargo test --manifest-path src-tauri/Cargo.toml` for the frontend and Rust tests. To compile without launching the UI or producing an installer, run `npm run tauri -- build --debug --no-bundle`; this places the executable at `src-tauri/target/debug/boost.exe`.

**Build a shareable installer:** `npm run stage:dispatch` copies `dispatch.exe` into the bundle, and `npm run package` produces a per-user NSIS installer at `src-tauri/target/release/bundle/nsis/Boost_0.1.0_x64-setup.exe`. The recipient needs no Rust or Node.js. See [packaging and sharing](docs/08-packaging-and-sharing.md) for prerequisites, the code-signing caveat, and a friend-ready checklist.

**Build and smoke-test verification (2026-11-04):** Rust 1.99.1, Cargo 1.99.1, Visual Studio Build Tools 2022 with MSVC/Windows SDK, and WebView2 are installed. `cargo test --manifest-path src-tauri/Cargo.toml` passes (1 test) and `npm run tauri -- build --debug --no-bundle` succeeds. The desktop window opens, lists local adapters, and starts/stops Dispatch on loopback; a SOCKS5 handshake succeeds. Duplicate start, stale address, and busy-port errors were checked; normal window exit now stops Dispatch. No USB-tethered phone was connected, so phone identification, Steam routing, and multi-adapter traffic remain untested.

**Diagnostics verification (2026-11-12):** `npm test` and `cargo test --manifest-path src-tauri/Cargo.toml` pass (3 tests each). The running desktop app displayed nonzero Wi-Fi receive/send rates, the on-demand Wi-Fi check reached `1.1.1.1:443`, and an unavailable IPv4 was reported offline. No USB-tethered phone or Steam-routing trial was available for this check.

## Documentation

- [Goals, scope and success criteria](docs/01-goals.md)
- [Research and primary sources](docs/02-research.md)
- [Architecture and implementation boundaries](docs/03-architecture.md)
- [Experimental test plan](docs/04-validation.md)
- [Roadmap, risks and open decisions](docs/05-roadmap.md)
- [Selected-application routing and setup](docs/07-selected-app-routing.md)
- [Packaging and sharing with a friend](docs/08-packaging-and-sharing.md)

## Project layout

`src-tauri/src/` owns adapter discovery and the Dispatch child process. `src/` is a minimal vanilla JS interface; it calls only the exposed Tauri commands. The browser UI is not allowed to run arbitrary shell commands. `docs/` records verified facts separately from assumptions and proposals.
