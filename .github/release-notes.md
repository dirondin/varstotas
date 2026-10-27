## Downloads

| System | File | |
|---|---|---|
| **macOS** 11+ (Apple Silicon and Intel) | `Varstotas-{version}-macos-universal.dmg` | open, drag Varstotas to Applications |
| | `Varstotas-{version}-macos-universal.zip` | the same app, zipped |
| **Windows** 10+ (x64) | `Varstotas-{version}-windows-x86_64-setup.exe` | installer: Start menu, `.vrst` files, uninstall |
| | `Varstotas-{version}-windows-x86_64.zip` | portable: unzip and run `varstotas.exe` |
| **Windows** 11 on Arm | `Varstotas-{version}-windows-aarch64-setup.exe` | the same, native Arm64 |
| | `Varstotas-{version}-windows-aarch64.zip` | |
| **Linux** (x86_64, glibc 2.35+) | `Varstotas-{version}-linux-x86_64.AppImage` | `chmod +x` and run |
| | `Varstotas-{version}-linux-x86_64.tar.gz` | unpack, run `./varstotas` or `./install.sh` |
| **Linux** (aarch64, glibc 2.35+) | `Varstotas-{version}-linux-aarch64.AppImage` | the same, for Arm64 |
| | `Varstotas-{version}-linux-aarch64.tar.gz` | |

`SHA256SUMS.txt` lists the files' checksums (`shasum -a 256 -c SHA256SUMS.txt`).

The builds are not signed with a paid certificate:

- **macOS** — the first launch is blocked: open Varstotas once, then **System Settings → Privacy & Security** →
  **Open Anyway** (macOS 14 and earlier: right-click Varstotas → **Open** → **Open**).
  Or in Terminal: `xattr -dr com.apple.quarantine /Applications/Varstotas.app`.
- **Windows** — SmartScreen may warn: **More info** → **Run anyway**.
