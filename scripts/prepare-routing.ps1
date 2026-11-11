param([string]$Destination = "$env:LOCALAPPDATA\Boost\runtime\proxifyre-2.6.4", [switch]$InstallNetwork)
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
if (-not [Environment]::Is64BitOperatingSystem -or $env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    throw 'This development setup currently supports x64 Windows only.'
}
$url = 'https://github.com/wiresock/proxifyre/releases/download/v2.6.4/ProxiFyre-v2.6.4-x64.zip'
$expected = 'c7da1dc5fdf9aaa7828fbf5c2076c4f612cba7889dcac2e3d06dc2e1dee4c3df'
$staging = Join-Path ([IO.Path]::GetTempPath()) ("boost-routing-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $staging | Out-Null
try {
    $zip = Join-Path $staging 'engine.zip'
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
    if ((Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) { throw 'ProxiFyre package checksum mismatch.' }
    $payload = Join-Path $staging 'payload'
    Expand-Archive $zip -DestinationPath $payload
    $engine = @(Get-ChildItem $payload -Filter ProxiFyre.exe -Recurse)
    if ($engine.Count -ne 1) { throw 'Expected exactly one ProxiFyre executable in the pinned package.' }
    if (Test-Path $Destination) {
        # Existing runtime must match the pinned package; never overwrite a live or unknown engine.
        foreach ($file in Get-ChildItem $engine[0].Directory.FullName -File) {
            if ($file.Extension -notin @('.exe','.dll') -and $file.Name -ne 'ProxiFyre.exe.config') { continue }
            $existing = Join-Path $Destination $file.Name
            if (-not (Test-Path $existing) -or (Get-FileHash $existing).Hash -ne (Get-FileHash $file.FullName).Hash) {
                throw "Existing runtime differs from pinned release: $existing. Close Boost and repair this runtime before retrying."
            }
        }
    } else {
        New-Item -ItemType Directory -Path (Split-Path $Destination) -Force | Out-Null
        Move-Item $engine[0].Directory.FullName $Destination
    }
    Set-Content (Join-Path $Destination '.boost-managed') 'proxifyre-2.6.4-x64' -Encoding Ascii
    Write-Output "Routing engine staged at $Destination"
    if ($InstallNetwork) {
        $driver = Join-Path $staging 'Windows.Packet.Filter.3.6.2.1.x64.msi'
        $driverPath = "$env:windir\System32\drivers\ndisrd.sys"
        $driverVersion = if (Test-Path $driverPath) { [Version](Get-Item $driverPath).VersionInfo.FileVersion } else { [Version]'0.0' }
        $installDriver = -not (Get-Service ndisrd -ErrorAction SilentlyContinue) -or $driverVersion -lt [Version]'3.6.1.0'
        if ($driverVersion -ge [Version]'4.0') { throw 'Installed NDISRD is newer than the supported 3.x API. No downgrade attempted.' }
        if ($installDriver) {
            Invoke-WebRequest 'https://github.com/wiresock/ndisapi/releases/download/v3.6.2/Windows.Packet.Filter.3.6.2.1.x64.msi' -OutFile $driver -UseBasicParsing
            if ((Get-FileHash $driver -Algorithm SHA256).Hash.ToLowerInvariant() -ne '9c388c0b7f189f7fa98720bae2caecf7d64f30910838b80b438ecf8956b8502c') { throw 'Driver checksum mismatch.' }
            if ((Get-AuthenticodeSignature $driver).Status -ne 'Valid') { throw 'Driver signature is not valid.' }
        }
        # Elevate only fixed driver/firewall operations, not the user-writable routing executable.
        $quotedDriver = $driver.Replace("'", "''")
        $quotedEngine = (Join-Path $Destination 'ProxiFyre.exe').Replace("'", "''")
        $ruleName = 'Boost-SelectedApp-TCP-' + [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        $install = if ($installDriver) { @"
`$installer = Start-Process msiexec.exe -ArgumentList '/i `"$quotedDriver`" /passive /norestart' -PassThru -Wait
if (`$installer.ExitCode -ne 0) { throw "Driver installation returned code `$(`$installer.ExitCode). A reboot may be required; rerun setup after reboot." }
"@ } else { '' }
        $admin = @"
`$ErrorActionPreference = 'Stop'
try {
$install
Start-Service ndisrd
Get-NetFirewallRule -Name '$ruleName' -ErrorAction SilentlyContinue | Remove-NetFirewallRule
New-NetFirewallRule -Name '$ruleName' -DisplayName 'Boost selected-application TCP routing' -Direction Inbound -Program '$quotedEngine' -Protocol TCP -Action Allow -Profile Any | Out-Null
exit 0
} catch { Write-Error `$_ -ErrorAction Continue; exit 1 }
"@
        $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($admin))
        $elevated = Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -EncodedCommand $encoded" -PassThru -Wait
        if ($elevated.ExitCode -ne 0) { throw "Network setup failed or was cancelled (exit $($elevated.ExitCode))." }
        Set-Content (Join-Path $Destination '.boost-network-ready') '2.6.4' -Encoding Ascii
        Write-Output 'Network setup complete. No application routing starts until you click Boost application.'
    } else { Write-Output 'Run again with -InstallNetwork to install the driver and register the TCP firewall rule.' }
} finally { Remove-Item $staging -Recurse -Force }
