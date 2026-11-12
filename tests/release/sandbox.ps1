# Runs the Windows package tests (run.ps1) in Windows Sandbox: a clean Windows where the
# installer can be installed and removed without touching this computer. The packages folder is
# shared with the sandbox, so the report lands in its test-results\ as with run.ps1. Needs
# Windows 10/11 Pro or Enterprise with the "Windows Sandbox" feature turned on.
#
# usage: sandbox.cmd [-NoGpu] [-Full] [packages folder]
#   -NoGpu   no graphics card in the sandbox: the app has to fall back to software rendering,
#            as in a virtual machine without 3D acceleration
#   -Full    the full script set on every package
param(
  [string]$Packages = "",
  [switch]$NoGpu,
  [switch]$Full
)
$here = $PSScriptRoot
$kit = if (Test-Path "$here\tests\ui\scripts") { $here } else { (Resolve-Path "$here\..\..").Path }
if (-not $Packages) { $Packages = if (Test-Path "$here\..\SHA256SUMS.txt") { "$here\.." } else { "." } }
$pkgdir = (Resolve-Path $Packages).Path
$sandbox = "$env:SystemRoot\System32\WindowsSandbox.exe"
if (-not (Test-Path $sandbox)) {
  Write-Host "Windows Sandbox is not turned on: Settings > System > Optional features > More Windows features > Windows Sandbox (Pro or Enterprise), then restart." -ForegroundColor Red
  exit 1
}
$runner = "C:\kit" + $here.Substring($kit.Length) + "\run.ps1"
$opts = @()
if ($Full) { $opts += "-Full" }
$gpu = if ($NoGpu) { "Disable" } else { "Enable" }
$wsb = @"
<Configuration>
  <vGPU>$gpu</vGPU>
  <Networking>Disable</Networking>
  <MappedFolders>
    <MappedFolder><HostFolder>$pkgdir</HostFolder><SandboxFolder>C:\release</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder>
    <MappedFolder><HostFolder>$kit</HostFolder><SandboxFolder>C:\kit</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
  </MappedFolders>
  <LogonCommand>
    <Command>cmd /c start "Varstotas package tests" powershell -NoProfile -ExecutionPolicy Bypass -NoExit -File $runner $($opts -join " ") C:\release</Command>
  </LogonCommand>
</Configuration>
"@
$file = Join-Path ([IO.Path]::GetTempPath()) "varstotas-tests.wsb"
[IO.File]::WriteAllText($file, $wsb)
Write-Host "Starting Windows Sandbox (graphics: $gpu); the tests open in a PowerShell window inside it,"
Write-Host "the report goes to $pkgdir\test-results. Closing the sandbox discards everything else."
Start-Process $file
