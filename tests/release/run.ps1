# Package tests of a release on Windows (run.sh is the macOS and Linux one): every package for this
# processor (the installer and the .zip) is checked against SHA256SUMS.txt, installed silently or
# unpacked into a temporary folder, its version checked, started once as a user would (graphics
# chosen as on a normal start, a screenshot taken) and driven through the UI scripts of
# scripts.txt; the installer is then uninstalled. Writes test-results\<time>-windows-<cpu>\
# report.md and report.html beside the packages and ends with PASS or FAIL (exit code 0 or 1).
#
# usage: run.cmd [options] [packages folder]   (default: the folder this kit is in, or the current one)
#   -Full        the full script set on every package (default: only on the installer)
#   -Quick       the quick set only
#   -Only NAME   only packages whose file name contains NAME (e.g. -Only setup)
#   -AllCpu      also the packages for the other processor (x64 under emulation on Arm)
#   -Keep        keep the temporary folders
# The installer is skipped when Varstotas is installed on this computer already (it would update
# and then remove that one): test it in Windows Sandbox (sandbox.cmd) instead.
param(
  [string]$Packages = "",
  [switch]$Full,
  [switch]$Quick,
  [string[]]$Only = @(),
  [switch]$AllCpu,
  [switch]$Keep
)
$ErrorActionPreference = "Continue"
$ProgressPreference = "SilentlyContinue"

$here = $PSScriptRoot
$kit = if (Test-Path "$here\tests\ui\scripts") { $here } else { (Resolve-Path "$here\..\..").Path }
if (-not $Packages) { $Packages = if (Test-Path "$here\..\SHA256SUMS.txt") { "$here\.." } else { "." } }
$pkgdir = (Resolve-Path $Packages).Path

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
$cpu = if ($arch -eq "Arm64") { "aarch64" } else { "x86_64" }
$out = Join-Path $pkgdir ("test-results\" + (Get-Date -Format "yyyyMMdd-HHmmss") + "-windows-$cpu")
New-Item -ItemType Directory -Force "$out\logs", "$out\shots" | Out-Null
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("varstotas-release-" + [guid]::NewGuid().ToString("N").Substring(0, 8))
New-Item -ItemType Directory -Force $tmp | Out-Null
$rows = New-Object System.Collections.ArrayList
$started = Get-Date
$appId = "{2BD864DB-C4B3-457C-9A3F-9CBD48469539}_is1"

function Row($variant, $check, $result, $detail = "") {
  [void]$rows.Add([pscustomobject]@{ Variant = $variant; Check = $check; Result = $result; Detail = "$detail" })
  $color = switch ($result) { "ok" { "Green" } "FAIL" { "Red" } default { "DarkGray" } }
  Write-Host ("  {0,-4} " -f $result) -ForegroundColor $color -NoNewline
  $d = "$detail"; if ($d.Length -gt 150) { $d = $d.Substring(0, 150) }
  Write-Host ("{0,-22} {1}" -f $check, $d)
}

function Head($text) { Write-Host ""; Write-Host $text -ForegroundColor White }

# Start the app with extra environment variables, wait (killed after $Seconds); the exit code,
# stdout and stderr go to $Log. 124 when it had to be killed.
function Run-App($Exe, $Arguments, $EnvVars, $Log, $Seconds, $WorkDir) {
  $saved = @{}
  foreach ($k in $EnvVars.Keys) { $saved[$k] = [Environment]::GetEnvironmentVariable($k); [Environment]::SetEnvironmentVariable($k, $EnvVars[$k]) }
  try {
    $a = @{ FilePath = $Exe; PassThru = $true; NoNewWindow = $true; RedirectStandardOutput = "$Log.out"; RedirectStandardError = "$Log.err" }
    if ($Arguments) { $a.ArgumentList = $Arguments }
    if ($WorkDir) { $a.WorkingDirectory = $WorkDir }
    $p = Start-Process @a
    $code = 124
    if ($p.WaitForExit($Seconds * 1000)) { $p.WaitForExit(); $code = $p.ExitCode } else { $p.Kill() }
  } finally {
    foreach ($k in $saved.Keys) { [Environment]::SetEnvironmentVariable($k, $saved[$k]) }
  }
  $text = ""
  foreach ($f in "$Log.out", "$Log.err") { if (Test-Path $f) { $text += [IO.File]::ReadAllText($f); Remove-Item $f } }
  [IO.File]::WriteAllText($Log, $text)
  return $code
}

function Fresh-Home($name) {
  $h = Join-Path $tmp "home-$name"
  if (Test-Path $h) { Remove-Item -Recurse -Force $h }
  New-Item -ItemType Directory -Force "$h\.varstotas" | Out-Null
  return $h
}

function App-Env($h) {
  return @{ HOME = $h; VARSTOTAS_DATA_DIR = "$h\.varstotas"; VARSTOTAS_LANG = "en"; RUST_LOG = "info" }
}

# --- the release --------------------------------------------------------------------------------
$version = $null
foreach ($f in Get-ChildItem $pkgdir -Filter "Varstotas-*") {
  if ($f.Name -match '^Varstotas-(.*)-(macos|linux|windows)-') { $version = $Matches[1]; break }
}
if (-not $version) { Write-Host "no Varstotas-* packages in $pkgdir"; exit 1 }

Write-Host "Varstotas $version - package tests on windows $cpu" -ForegroundColor White
Write-Host "packages: $pkgdir"
Write-Host "results:  $out"

$os = Get-CimInstance Win32_OperatingSystem
$sys = @("system: $($os.Caption) $($os.Version) ($arch)")
if ($env:USERNAME -eq "WDAGUtilityAccount") { $sys += "Windows Sandbox" }
foreach ($g in Get-CimInstance Win32_VideoController) { $sys += "gpu: $($g.Name) (driver $($g.DriverVersion))" }
[IO.File]::WriteAllText("$out\system.txt", ($sys -join "`n") + "`n")
$sys | ForEach-Object { Write-Host "  $_" }

Head "checksums"
$sums = Join-Path $pkgdir "SHA256SUMS.txt"
if (-not (Test-Path $sums)) {
  Row release SHA256SUMS FAIL "no SHA256SUMS.txt"
} else {
  $n = 0; $bad = 0; $missing = 0
  foreach ($line in Get-Content $sums) {
    if ($line -notmatch '^([0-9a-f]{64})\s+\*?(.+)$') { continue }
    $hash = $Matches[1]; $name = $Matches[2].Trim()
    $p = Join-Path $pkgdir $name
    if (-not (Test-Path $p)) { $missing++; continue }
    $n++
    if ((Get-FileHash -Algorithm SHA256 $p).Hash.ToLower() -ne $hash) { $bad++; Row release checksum FAIL $name }
  }
  if ($bad -eq 0) { Row release SHA256SUMS ok ("$n files match" + $(if ($missing) { ", $missing listed but not here" } else { "" })) }
}

# --- one app ------------------------------------------------------------------------------------
function Script-List($set) {
  foreach ($line in Get-Content "$here\scripts.txt") {
    if ($line -match '^\s*#' -or $line -notmatch '^(quick|full)\s+(\S+)') { continue }
    if ($Matches[1] -eq "quick" -or $set -eq "full") { $Matches[2] }
  }
}

function Test-App($v, $exe, $set) {
  $log = "$out\logs\$v-version.log"
  $code = Run-App $exe "--version" @{} $log 30
  $vout = (Get-Content $log -ErrorAction SilentlyContinue | Select-Object -Last 1)
  if ("$vout" -like "Varstotas $version (*") { Row $v "--version" ok $vout } else { Row $v "--version" FAIL "exit $code, '$vout'" }

  # a normal start (graphics probed and chosen as for a user), a screenshot after ~40 frames
  $h = Fresh-Home "$v-start"
  $log = "$out\logs\$v-start.log"
  $shot = "$out\shots\$v-start.bmp"
  $envs = App-Env $h; $envs.VARSTOTAS_SCREENSHOT = $shot
  $code = Run-App $exe $null $envs $log 120
  $text = [IO.File]::ReadAllText($log)
  $gfx = ([regex]::Matches($text, 'graphics in use: (.*)') | Select-Object -Last 1)
  $gfx = if ($gfx) { $gfx.Groups[1].Value.Trim() } else { "graphics not logged" }
  if ($code -eq 0 -and (Test-Path $shot) -and $text -notmatch 'panicked') { Row $v start ok $gfx }
  else {
    $err = ($text -split "`n" | Where-Object { $_ -match 'ERROR|panicked' } | Select-Object -First 1)
    Row $v start FAIL ("exit $code" + $(if (-not (Test-Path $shot)) { ", no window image" } else { "" }) + "; $err")
  }

  # the UI scripts, each in its own folder with a fresh home
  foreach ($n in Script-List $set) {
    $s = "$kit\tests\ui\scripts\$n.txt"
    if (-not (Test-Path $s)) { Row $v "script $n" FAIL "missing $s"; continue }
    $h = Fresh-Home "$v-$n"
    if (Test-Path "$kit\tests\ui\fixtures\$n\components") { Copy-Item -Recurse "$kit\tests\ui\fixtures\$n\components" "$h\.varstotas\" }
    $work = Join-Path $tmp "work-$v-$n"
    New-Item -ItemType Directory -Force "$work\models", "$out\shots\$v" | Out-Null
    # shots\ and home\ in the script's folder (a junction: no admin rights needed)
    cmd /c mklink /J "$work\shots" "$out\shots\$v" | Out-Null
    cmd /c mklink /J "$work\home" "$h" | Out-Null
    $log = "$out\logs\$v-$n.log"
    $envs = App-Env $h; $envs.VARSTOTAS_FOREGROUND = "1"; $envs.VARSTOTAS_SCRIPT = $s
    $t0 = Get-Date
    $code = Run-App $exe $null $envs $log 400 $work
    $lines = Get-Content $log
    $bad = @($lines | Where-Object { $_ -match 'expect FAIL|panicked' })
    $ok = @($lines | Where-Object { $_ -match 'expect ok' }).Count
    if ($code -eq 0 -and $bad.Count -eq 0) { Row $v "script $n" ok ("$ok checks, {0:N0}s" -f ((Get-Date) - $t0).TotalSeconds) }
    else {
      $first = if ($bad.Count) { ($bad[0] -replace '.*(expect FAIL|panicked)', '$1') } else { "" }
      Row $v "script $n" FAIL "exit $code, $($bad.Count) failed, $ok ok: $first"
    }
    cmd /c rmdir "$work\shots" | Out-Null
    cmd /c rmdir "$work\home" | Out-Null
  }
}

function Set-For($file) {
  if ($Quick) { "quick" } elseif ($Full -or $file -like "*-setup.exe") { "full" } else { "quick" }
}

# --- the packages -------------------------------------------------------------------------------
# the installer's entry in Apps (per user or for everyone)
function Uninstall-Key {
  foreach ($k in "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$appId",
                 "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$appId",
                 "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\$appId") {
    if (Test-Path $k) { return $k }
  }
  return $null
}

function Test-Zip($f, $v) {
  Head $f.Name
  $dir = Join-Path $tmp $v
  try { Expand-Archive -LiteralPath $f.FullName -DestinationPath $dir -Force } catch { Row $v unpack FAIL $_; return }
  $exe = "$dir\varstotas.exe"
  if (-not (Test-Path $exe)) { Row $v unpack FAIL "no varstotas.exe"; return }
  Row $v unpack ok ((Get-ChildItem $dir).Name -join ", ")
  Test-App $v $exe (Set-For $f.Name)
}

function Test-Setup($f, $v) {
  Head $f.Name
  $existing = Uninstall-Key
  if ($existing) {
    Row $v install skip "Varstotas is installed on this computer already ($existing): left as it is; test the installer in Windows Sandbox (sandbox.cmd)"
    return
  }
  $dir = Join-Path $tmp "$v-app"
  $log = "$out\logs\$v-install.log"
  $p = Start-Process $f.FullName -ArgumentList "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/CURRENTUSER", "/TASKS=assoc", "/DIR=`"$dir`"", "/LOG=`"$log`"" -PassThru -Wait
  $exe = "$dir\varstotas.exe"
  $key = Uninstall-Key
  if ($p.ExitCode -ne 0 -or -not (Test-Path $exe) -or -not $key) {
    Row $v install FAIL "exit $($p.ExitCode); exe: $(Test-Path $exe); uninstall entry: $([bool]$key) (log: logs\$v-install.log)"
    return
  }
  $entry = Get-ItemProperty $key
  Row $v install ok "$($entry.DisplayName) $($entry.DisplayVersion) in $dir"
  $lnk = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Varstotas.lnk"
  if (Test-Path $lnk) { Row $v "Start menu" ok "Varstotas.lnk" } else { Row $v "Start menu" FAIL "no $lnk" }
  $assoc = (Get-ItemProperty "HKCU:\Software\Classes\.vrst" -ErrorAction SilentlyContinue).'(default)'
  $open = (Get-ItemProperty "HKCU:\Software\Classes\Varstotas.Document\shell\open\command" -ErrorAction SilentlyContinue).'(default)'
  if ($assoc -eq "Varstotas.Document" -and "$open" -like "*$exe*") { Row $v ".vrst files" ok $open } else { Row $v ".vrst files" FAIL "'.vrst' = '$assoc', open = '$open'" }

  Test-App $v $exe (Set-For $f.Name)

  # uninstall: the uninstaller copies itself to the temporary folder and returns at once
  $u = (Get-ChildItem $dir -Filter "unins*.exe" | Select-Object -First 1)
  Start-Process $u.FullName -ArgumentList "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART" -Wait
  $deadline = (Get-Date).AddSeconds(90)
  while (((Uninstall-Key) -or (Test-Path $exe)) -and (Get-Date) -lt $deadline) { Start-Sleep 1 }
  $left = @()
  if (Uninstall-Key) { $left += "uninstall entry" }
  if (Test-Path $exe) { $left += "varstotas.exe" }
  if (Test-Path $lnk) { $left += "Start menu entry" }
  if (Test-Path "HKCU:\Software\Classes\Varstotas.Document") { $left += ".vrst file type" }
  if ($left.Count -eq 0) { Row $v uninstall ok "nothing left" } else { Row $v uninstall FAIL ("left: " + ($left -join ", ")) }
}

$cpus = if ($AllCpu) { @("x86_64", "aarch64") } else { @($cpu) }
$pkgs = Get-ChildItem $pkgdir -Filter "Varstotas-$version-windows-*" | Where-Object {
  $name = $_.Name
  ($cpus | Where-Object { $name -like "*-$_*" }) -and ($Only.Count -eq 0 -or ($Only | Where-Object { $name -like "*$_*" }))
} | Sort-Object { if ($_.Name -like "*-setup.exe") { 0 } else { 1 } }
if (-not $pkgs) { Row release packages FAIL "no packages for windows $cpu" }
foreach ($f in $pkgs) {
  $c = if ($f.Name -like "*-aarch64*") { "arm64" } else { "x64" }
  $tag = if ($AllCpu) { "-$c" } else { "" }
  if ($f.Name -like "*-setup.exe") { Test-Setup $f "setup$tag" } elseif ($f.Name -like "*.zip") { Test-Zip $f "zip$tag" }
}

# --- report -------------------------------------------------------------------------------------
$fails = @($rows | Where-Object { $_.Result -eq "FAIL" }).Count
$oks = @($rows | Where-Object { $_.Result -eq "ok" }).Count
$mins = [math]::Ceiling(((Get-Date) - $started).TotalMinutes)
$verdict = if ($fails) { "FAIL" } else { "PASS" }
$when = Get-Date -Format "yyyy-MM-dd HH:mm"
$md = @("# Varstotas $version - windows ${cpu}: $verdict", "", "$when, $mins min, $oks passed, $fails failed.", "", '```')
$md += Get-Content "$out\system.txt"
$md += @('```', "", "| Package | Check | Result | Detail |", "|---|---|---|---|")
foreach ($r in $rows) { $res = if ($r.Result -eq "FAIL") { "**FAIL**" } else { $r.Result }; $md += "| $($r.Variant) | $($r.Check) | $res | $($r.Detail -replace '\|', '\|') |" }
$md += @("", "Logs: logs\, window images: shots\.")
[IO.File]::WriteAllText("$out\report.md", ($md -join "`n") + "`n")
function Esc($s) { [System.Net.WebUtility]::HtmlEncode("$s") }
$html = @("<!doctype html><meta charset=utf-8><title>Varstotas $version windows ${cpu}: $verdict</title>",
  "<style>body{font:14px system-ui,sans-serif;margin:16px;color:#222;background:#fff}td,th{padding:3px 8px;border-bottom:1px solid #ddd;text-align:left;vertical-align:top}",
  ".ok{color:#18794e}.FAIL{color:#c00;font-weight:bold}.skip,.info{color:#888}h1 .PASS{color:#18794e}img{max-width:320px;border:1px solid #ccc;margin:4px}pre{background:#f4f4f4;padding:8px}</style>",
  "<h1>Varstotas $version - windows ${cpu}: <span class=$verdict>$verdict</span></h1>",
  "<p>$when, $mins min, $oks passed, $fails failed.</p><pre>$(Esc ((Get-Content "$out\system.txt") -join "`n"))</pre>",
  "<table><tr><th>Package<th>Check<th>Result<th>Detail")
foreach ($r in $rows) { $html += "<tr><td>$(Esc $r.Variant)<td>$(Esc $r.Check)<td class=$($r.Result)>$($r.Result)<td>$(Esc $r.Detail)" }
$html += "</table><h2>Window images</h2>"
foreach ($s in Get-ChildItem "$out\shots" -Recurse -Filter "*.bmp" | Sort-Object FullName) {
  $rel = $s.FullName.Substring($out.Length + 1) -replace '\\', '/'
  $html += "<a href=`"$rel`"><img src=`"$rel`" title=`"$rel`" loading=lazy></a>"
}
[IO.File]::WriteAllText("$out\report.html", ($html -join "`n") + "`n")
if ($Keep) { Write-Host "temporary files: $tmp" } else { Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue }

Write-Host ""
if ($verdict -eq "PASS") { Write-Host "PASS - $oks checks, $mins min" -ForegroundColor Green } else { Write-Host "FAIL - $fails failed, $oks passed, $mins min" -ForegroundColor Red }
Write-Host "report: $out\report.html"
if ($verdict -eq "PASS") { exit 0 } else { exit 1 }
