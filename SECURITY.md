# Security

Varstotas is a desktop application. It runs as the current user and is not an OS sandbox.

## What is trusted

- **Trusted code** — scripts run by hand (Script Console, File → Run Script) and installed
  plugins (Rhai and Python). They get host file and network access. Run only code you trust.
  Python scripts and plugins are ordinary processes of the current user.
- **Untrusted code** — generators and component scripts inside documents (including automatic
  previews) and every bridge / MCP `eval`. They get the geometry API only: no file reads or
  writes, no directory listing, no downloads. Opening a `.vrst` file never gains host IO.
- **The local bridge** listens only on `127.0.0.1` on a random port and needs the random token
  from `~/.varstotas/bridge.json` (permissions 0600). Browser requests (with an `Origin` header)
  are rejected. `VARSTOTAS_NO_BRIDGE=1` turns it off.

Rhai runs have a deadline and memory / operation limits; downloads accept HTTP(S) only, with size
and time caps. Details: [docs/SCRIPTING.md](docs/SCRIPTING.md#trust-and-execution-limits).

## Known limits

- Native geometry calls cannot be interrupted mid-call, so a hostile document can still make
  the application slow.
- Undo and script-error rollback restore the model, not files written or requests sent by
  trusted code.
- Release builds are not code-signed (macOS Gatekeeper and Windows SmartScreen warn on first
  start); check downloads against `SHA256SUMS.txt` from the release.

## Reporting a problem

Please report vulnerabilities privately through GitHub:
**Security → Report a vulnerability** on the repository page. Include the version
(`varstotas --version`), the platform and, if possible, a file or script that reproduces it.
