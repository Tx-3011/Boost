# Packaging and sharing Boost

Boost builds a per-user Windows installer with NSIS. The installer contains the
Boost desktop app and a pinned `dispatch.exe`; the routing engine (ProxiFyre) and
Windows Packet Filter are fetched during first-run setup instead of being
redistributed in the installer.

## Build the installer

On the build machine:

1. Install Rust (MSVC), Node.js, Tauri's Windows prerequisites, and
   `dispatch` (`cargo install dispatch-proxy` or a release build on `PATH`).
2. In the project folder:

   ```powershell
   npm install
   npm run stage:dispatch   # copies dispatch.exe into src-tauri/binaries
   npm run package          # release build + NSIS installer
   ```

`npm run stage:dispatch` is also run automatically by `npm run package`'s
`beforeBuildCommand`. Use `BOOST_DISPATCH_PATH` to bundle a specific
`dispatch.exe` instead of the one on `PATH`.

Output:

```
src-tauri\target\release\bundle\nsis\Boost_0.1.0_x64-setup.exe
```

The installer is per-user (`installMode: currentUser`), so installing Boost does
not itself require administrator rights. Only the one-time routing component
setup asks for elevation.

## What the friend receives

Send `Boost_0.1.0_x64-setup.exe` plus the short "Friend checklist" below. A
single file is enough; no Rust, Node.js, or `dispatch` installation is needed.

SmartScreen will warn because the build is unsigned. That is expected for the
alpha. Options, in order of effort:

- **Unsigned (today):** tell the friend to choose **More info → Run anyway**.
  Some antivirus products also flag the driver installer.
- **Sign it:** an Authenticode code-signing certificate (OV/EV) removes most
  SmartScreen warnings over time. This is the main blocker for a smooth
  consumer release and costs money plus a signing step in the build.
- **Avoid the warning entirely:** not possible for an unsigned binary that
  installs a network driver.

## Friend checklist

1. Windows 10/11 **x64**, with both phones able to provide USB tethering (or one
   phone plus Wi-Fi).
2. Install `Boost_0.1.0_x64-setup.exe`. If SmartScreen appears: **More info →
   Run anyway**.
3. Make sure these Windows prerequisites are present:
   - **WebView2 runtime** — included with Windows 11 and most updated Windows 10;
     the installer offers to download it if missing.
   - **.NET Framework 4.7.2 or newer** — built into current Windows 10/11. Check
     with `reg query "HKLM\SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full" /v Release`
     (a value of 461808 or higher means 4.7.2+).
   - **Microsoft Visual C++ 2015–2022 x64 runtime** — install
     `vc_redist.x64.exe` from Microsoft if the routing engine fails to start.
4. Open Boost, connect and unlock the phones, enable USB tethering, then click
   **Refresh** and select **two different** adapters (e.g. `Ethernet 3` and
   `Wi-Fi`). Do not select VPN or virtual adapters.
5. Expand **Routing component setup** and click **Install routing component**.
   Approve the UAC prompt. This downloads and verifies ProxiFyre 2.6.4, installs
   Windows Packet Filter, and adds a program-scoped inbound TCP firewall rule.
   Networking may pause briefly; a reboot may be requested.
6. Click **Choose .exe** and pick the program that actually makes the download
   connections (for example a browser, a game launcher, or a download manager).
7. Click **Boost application**, then start a **fresh** download (restart the app
   if it was already running).
8. Click **Stop** when finished. Closing Boost also stops Dispatch and the
   routing engine.

## Updating and uninstalling

- Installing a newer build over an older one keeps `%LOCALAPPDATA%\Boost\runtime`,
  so the routing component does not need to be downloaded again.
- Uninstalling Boost from **Settings → Apps** removes `%LOCALAPPDATA%\Boost`,
  including the staged routing engine and its lock/log files. Windows Packet
  Filter and the firewall rule are left in place because the driver may be
  shared with other software. A reinstall therefore needs one click of
  **Install routing component** again.
- To remove the driver deliberately, uninstall **Windows Packet Filter** from
  **Settings → Apps** only if no other product uses it. Remove the
  `Boost selected-application TCP routing` firewall rule from Windows Defender
  Firewall if you want to undo the setup completely.

## Honest expectations

- This balances separate TCP connections across links. It is **not** bonding: a
  single connection still uses one link, so a two-phone setup does not
  guarantee twice the speed of one stream.
- Only **IPv4 TCP** is routed. UDP/QUIC (many browsers and games) is not
  accelerated, and matched IPv6 connections are blocked so apps can fall back.
- Rules match the **executable filename**, so every running instance with that
  name is included. Helper processes with other names are not.
- The engine runs in standard-user mode. Elevated or protected apps may stay
  direct.
- Steam's real downloader process has not been verified, so do not promise a
  working Steam preset yet.

## Licences and obligations

Boost itself is this repository. The installer bundles `dispatch-proxy` (MIT or
Apache-2.0, depending on the build), so send `THIRD-PARTY-NOTICES.md` along with
the installer. First-run setup downloads ProxiFyre (AGPL-3.0) and Windows Packet
Filter from their upstream releases; because the friend's machine fetches them
rather than receiving them in the installer, the installer does not repackage
them. Keep the credit and licence links in the setup UI and docs. If you later
bundle them inside the installer, include the AGPL source offer and the driver's
distribution terms.

## Later work for a friendlier release

- Code signing.
- Bundle the .NET/VC++ prerequisites and check them before routing setup.
- Verify the Steam downloader process and add a tested preset.
- Add auto-update and an uninstall path that offers to remove the staged
  runtime.
- Clean-machine test on a PC without Rust, Node.js, or any of the prerequisites.
