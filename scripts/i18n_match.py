#!/usr/bin/env python3
"""Moves the texts of a `fn …(self) -> &'static str { match self { A::B => "Text", … } }` to a
locale file: the arms get keys `<prefix>-<variant>` (kebab case), the function returns
`String` through `varstotas_i18n::tr`, and `key = Text` lines are appended to
locales/en.ftl (under a `## AREA` comment: move them into their section). Arms that are not a plain string literal are reported and left alone.

usage: scripts/i18n_match.py FILE FN PREFIX AREA [--suffix SUFFIX] [--keep-sig]
"""
import re, sys, pathlib

args = sys.argv[1:]
suffix = ''
keep = False
if '--suffix' in args:
    i = args.index('--suffix'); suffix = '-' + args[i + 1]; del args[i:i + 2]
if '--keep-sig' in args:
    args.remove('--keep-sig'); keep = True
path, fn, prefix, area = args
src = pathlib.Path(path).read_text()
m = re.search(r'(\n[ \t]*(?:pub(?:\(crate\))? )?fn ' + re.escape(fn) + r'\((?:self|&self)\) -> )&\'static str( \{)', src)
if not m:
    sys.exit(f'{fn} not found')
start = m.end()
# the function body: brace matching
depth, i = 1, start
while depth:
    c = src[i]
    if c == '"':
        i += 1
        while src[i] != '"':
            i += 2 if src[i] == '\\' else 1
    elif c == '{':
        depth += 1
    elif c == '}':
        depth -= 1
    i += 1
body = src[start:i - 1]
mm = re.match(r'(\s*)match self \{', body)
if not mm:
    sys.exit(f'{fn}: body is not `match self {{ … }}`')

def kebab(v):
    v = re.sub(r'\(.*', '', v)
    return re.sub(r'(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])', '-', v).lower()

lines = []
seen = {}
def arm(a):
    pat, text = a.group(1), a.group(2)
    variants = re.findall(r'::(\w+)', pat) or re.findall(r'\b(\w+)\b', pat)
    if not variants:
        return a.group(0)
    if text == '':
        return f'{pat} => return String::new(),'
    key = f'{prefix}-{kebab(variants[0])}{suffix}'
    text_u = bytes(text, 'utf-8').decode('unicode_escape').encode('latin-1').decode('utf-8')
    if key not in seen:
        seen[key] = text_u
        lines.append(f'{key} = {text_u}')
    return f'{pat} => "{key}",'

new_body, n = re.subn(r'([A-Za-z0-9_:\(\)\. |{}]+?)\s*=>\s*"((?:[^"\\]|\\.)*)",', arm, body)
left = [l.strip() for l in new_body.splitlines() if '=>' in l and f'"{prefix}-' not in l]
inner = new_body.strip()
if keep:
    out = src[:m.start()] + m.group(1) + "&'static str" + m.group(2) + new_body + src[i - 1:]
else:
    indent = re.match(r'\n([ \t]*)', m.group(1)).group(1)
    out = (src[:m.start()] + m.group(1) + 'String' + m.group(2) + '\n' + indent + '    varstotas_i18n::tr(' + inner + ')\n' + indent + src[i - 1:])
pathlib.Path(path).write_text(out)
ftl = pathlib.Path('locales/en.ftl')
with ftl.open('a') as f:
    f.write(f'\n## {area}\n\n' + '\n'.join(lines) + '\n')
print(f'{fn}: {len(lines)} keys → {ftl}')
for l in left:
    print('  left alone:', l)
