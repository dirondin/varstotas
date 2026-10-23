"""Live Python/bridge lock regression, run while Protected is locked by the UI."""
import varstotas
from varstotas import vec3

m = varstotas.connect()
checks = []

def check(name, ok):
    if not ok:
        raise AssertionError(name)
    checks.append(name)

def denied(name, f):
    try:
        f()
    except varstotas.VarstotasError:
        checks.append(name)
    else:
        raise AssertionError(name + " was accepted")

b = m.groups()[0]
check("effective lock is readable", b.locked)
check("geometry remains readable", m.bounds(b)["size"] == vec3(100, 50, 30))
denied("move locked object", lambda: m.move(b, vec3(100, 0, 0)))
denied("delete locked object", lambda: m.erase(b))
denied("unlock through setter", lambda: setattr(b, "locked", False))
denied("undo user lock", m.undo)
with m.operation("Read locked"):
    check("transaction sees lock", m.groups()[0].locked)
try:
    with m.operation("Forbidden edit"):
        m.box(vec3(500, 0, 0), 10, 10, 10)
        m.erase(b)
except varstotas.VarstotasError:
    check("mixed transaction fully rolled back", len(m.groups()) == 1)
else:
    raise AssertionError("locked deletion succeeded")
m.select(b)
check("locked selection filtered", m.selection() == [])
m.box(vec3(500, 0, 0), 10, 10, 10)
check("unrelated edits work", len(m.groups()) == 2)
m.undo()
check("unrelated undo works", len(m.groups()) == 1 and b.locked)
denied("undo still cannot reach lock", m.undo)
print("locking bridge checks: %d ok" % len(checks))
