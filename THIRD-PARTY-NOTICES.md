# Third-party notices

Boost is an experimental alpha. It uses or downloads the following components.
Their licences and copyright belong to their authors.

## Dispatch (`dispatch-proxy`)

- Project: https://github.com/alexkirsz/dispatch
- Licence: MIT OR Apache-2.0
- Distribution: a pinned `dispatch.exe` build is included in the Boost installer.
  The full MIT and Apache-2.0 texts are available in that repository.

## ProxiFyre

- Project: https://github.com/wiresock/proxifyre
- Licence: GNU Affero General Public License v3.0 (AGPL-3.0)
- Distribution: **not** included in the installer. Boost's first-run setup
  downloads the official, checksum-verified ProxiFyre 2.6.4 x64 release archive
  to `%LOCALAPPDATA%\Boost\runtime\proxifyre-2.6.4` on the user's machine.
  Source for that release is available from the project above.

## Windows Packet Filter (NDISRD / ndisapi)

- Project: https://github.com/wiresock/ndisapi
- Distribution: **not** included in the installer. Setup downloads the official
  Windows Packet Filter 3.6.2.1 x64 MSI from the project's releases and verifies
  its SHA-256 and Authenticode signature before installing it.
- Licence and redistribution terms: see the project repository and the MSI
  package.

## Microsoft runtime prerequisites

The .NET Framework, Visual C++ runtime, and WebView2 runtime are Microsoft
components installed separately on the user's machine. They are not
redistributed by Boost.

## Tauri

- Project: https://tauri.app
- Licence: MIT OR Apache-2.0
