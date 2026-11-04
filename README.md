# Boost

Boost is a Windows-first desktop prototype for selecting multiple USB-tethered network adapters and starting the **modern Rust `dispatch` SOCKS proxy** on `127.0.0.1:1080`. It is intended to grow into a simple Steam-download workflow.

> **Current scope:** Adapter discovery, manual selection, launching/stopping a local Dispatch process, and basic process/listener status. Boost does **not** yet route Steam or other applications, guarantee an aggregate speed increase, monitor per-adapter throughput, or provide true single-flow bonding. There is no packaged installer yet.

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

`npm run build` builds only the web frontend; `npm run tauri dev` compiles and runs the desktop application. Run `cargo test --manifest-path src-tauri/Cargo.toml` for the Rust tests. To compile without launching the UI or producing an installer, run `npm run tauri -- build --debug --no-bundle`; this places the executable at `src-tauri/target/debug/boost.exe`.

**Build verification (2026-11-04):** Rust 1.99.1, Cargo 1.99.1, Visual Studio Build Tools 2022 with MSVC/Windows SDK, and WebView2 are installed. `cargo test --manifest-path src-tauri/Cargo.toml` passes (1 test) and `npm run tauri -- build --debug --no-bundle` succeeds. Steam traffic routing and actual desktop interaction remain untested.

## Documentation

- [Goals, scope and success criteria](docs/01-goals.md)
- [Research and primary sources](docs/02-research.md)
- [Architecture and implementation boundaries](docs/03-architecture.md)
- [Experimental test plan](docs/04-validation.md)
- [Roadmap, risks and open decisions](docs/05-roadmap.md)

## Project layout

`src-tauri/src/` owns adapter discovery and the Dispatch child process. `src/` is a minimal vanilla JS interface; it calls only the exposed Tauri commands. The browser UI is not allowed to run arbitrary shell commands. `docs/` records verified facts separately from assumptions and proposals.
