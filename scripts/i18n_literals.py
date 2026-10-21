#!/usr/bin/env python3
"""Lists string literals in Rust sources that look like user-visible text (not yet moved to
locales/*.ftl). usage: scripts/i18n_literals.py [files or folders…]  (default: crates/*/src)

A literal counts when it has a word of 2+ letters and either starts with a capital letter or
holds a space between words. Skipped: comments, tr!/log/panic/assert/expect/env/include
arguments, spot / id names, attributes, `#[cfg(test)]` modules (to the end of the file), and
lines marked `// i18n-ignore` (or the line after a `// i18n-ignore-next` comment).
Exit status 1 when anything is found.
"""
import re, sys, pathlib

LIT = re.compile(r'(?<![A-Za-z0-9_#])(?:r#*)?"((?:[^"\\]|\\.)*)"')
WORD = re.compile(r'[A-Za-z]{2,}')
SKIP_CALLS = re.compile(
    r'(tr!\(|tr_args\(|tr\(|log::\w+!\(|\b(?:debug|info|warn|error|trace)!\(|panic!\(|unreachable!\(|todo!\(|assert\w*!\(|'
    r'\.expect\(|env::var\w*\(|env!\(|option_env!\(|include_\w+!\(|concat!\(|spot\(|Id::new\(|id_salt\(|from_id_salt\(|'
    r'\.with\(\(|#\[|register_fn\(|on_hover_text_at_pointer\(\s*$|eprintln!\(|println!\(|writeln!\(|write!\(|'
    r'\.join\(|\.ends_with\(|\.starts_with\(|\.contains\(|\.replace\(|\.split\w*\(|\.strip_\w+\(|\.trim\w*\(|'
    r'serde\(|rename|json!\(|\.get\("|add_filter\(|\.env\(|Command::new\(|label: Some\(|label:\s*$)'
)
EXTS = {'.rs'}
# Not UI: document content (demo scenes), the bridge / QA protocols, developer tools.
EXCLUDED = ('/demo/', 'debug_capture.rs', '/bridge.rs', 'bridge_ops.rs', 'varstotas_i18n/')


def literal_is_text(s: str) -> bool:
    if not WORD.search(s):
        return False
    if s.startswith(('http', '/', '.', '~', '$', '{', '%', '#', '@', '-', '\\', '<', 'x-')) and ' ' not in s:
        return False
    if re.fullmatch(r'[a-z0-9_:./\-]+', s):  # ids, keys, paths, extensions
        return False
    if re.fullmatch(r'[A-Z0-9_]+', s):  # constants, env names
        return False
    if re.search(r'\bfn |->|;|vec[234]<|\blet |^varstotas |\.(dll|so|dylib|exe)$', s):  # code, GPU labels, libraries
        return False
    return bool(re.match(r'[A-Z]', s)) or bool(re.search(r'[A-Za-z]{2,}\s+[A-Za-z]', s))


def scan(path: pathlib.Path):
    out = []
    lines = path.read_text(encoding='utf-8').splitlines()
    ignore_next = False
    for i, line in enumerate(lines, 1):
        t = line.strip()
        if t.startswith('#[cfg(test)]'):
            break
        if t.startswith('//'):
            ignore_next = 'i18n-ignore-next' in t
            continue
        if 'i18n-ignore' in line or ignore_next:
            ignore_next = False
            continue
        code = re.sub(r'\s//[^"]*$', '', line)
        for m in LIT.finditer(code):
            s = m.group(1)
            if not literal_is_text(s):
                continue
            before = code[:m.start()]
            if SKIP_CALLS.search(before[-60:]) and not re.search(r'\)\s*[,;]?\s*$', before[-60:].split('(')[-1] if '(' in before else ''):
                # the literal sits inside a skipped call's parentheses
                opened = before.rfind('(')
                call = before[:opened + 1]
                if SKIP_CALLS.search(call[-60:]):
                    continue
            out.append((i, s))
    return out


def main():
    args = sys.argv[1:] or ['crates']
    files = []
    for a in args:
        p = pathlib.Path(a)
        files += [p] if p.is_file() else sorted(q for q in p.rglob('*.rs') if '/src/' in str(q) and '/tests/' not in str(q) and not any(x in str(q) for x in EXCLUDED))
    total = 0
    for f in files:
        found = scan(f)
        total += len(found)
        for line, s in found:
            print(f'{f}:{line}: "{s}"')
    print(f'{total} literals', file=sys.stderr)
    sys.exit(1 if total else 0)


main()
