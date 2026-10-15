"""QA (qa/scripts/bridge.txt): the Python client, HTTP and the MCP server against the running app.
Prints one line per check; the QA script expects the final line."""
import json
import os
import subprocess
import sys
import tempfile
import urllib.request

import varstotas
from varstotas import ORIGIN, Z, vec3

m = varstotas.connect()
ok = []


def check(name, cond):
    print(("ok " if cond else "FAIL ") + name)
    if not cond:
        sys.exit(1)
    ok.append(name)


# Calls outside an operation: one undo step each.
b = m.box(ORIGIN, 1000, 600, 18)
check("box is a group", b.kind == "group" and isinstance(b, varstotas.Entity))
b.name = "Top"
check("name round trip", b.name == "Top")
check("bounds vec", m.bounds(b)["size"] == vec3(1000, 600, 18))
check("undo label", m.info()["undo"] == "Python")

# An exception inside an operation rolls everything back.
try:
    with m.operation("Broken"):
        m.box(vec3(0, 0, 100), 10, 10, 10)
        m.pushpull(42, 1)
except varstotas.VarstotasError as e:
    check("error message", "expected an entity" in str(e))
check("rolled back", len(m.groups()) == 1 and m.info()["undo"] == "Python")

# A successful operation is one step.
with m.operation("Legs"):
    for x in (0, 960):
        leg = m.box(vec3(x, 0, -700), 40, 40, 700)
        leg.name = "Leg"
check("one step", m.info()["undo"] == "Legs" and len(m.groups()) == 3)
check("rhai with vars", m.rhai("bounds(groups()).size.z + k", k=1) == 719.0)
m.undo()
check("undo", len(m.groups()) == 1)

# Camera, screenshot.
cam = m.view("top")
check("top view", abs(cam["eye"][2] - cam["target"][2]) > 0)
shot = m.screenshot(width=320, height=200)
check("screenshot", os.path.getsize(shot) > 1000 and open(shot, "rb").read(4) == b"\x89PNG")

# Plain HTTP with a bearer token.
info = json.load(open(os.path.expanduser("~/.varstotas/bridge.json")))
req = urllib.request.Request(
    "http://127.0.0.1:%d/rpc" % info["port"],
    data=json.dumps({"id": 7, "method": "info"}).encode(),
    headers={"Authorization": "Bearer " + info["token"], "Content-Type": "application/json"},
)
r = json.load(urllib.request.urlopen(req))
check("http info", r["id"] == 7 and r["result"]["root"]["groups"] == 1)
try:
    bad = urllib.request.Request(req.full_url, data=b'{"id":1,"method":"info","token":"nope"}')
    urllib.request.urlopen(bad)
    check("http bad token refused", False)
except urllib.error.HTTPError as e:
    check("http bad token refused", e.code == 401)

# Destructive: exporting over an existing file asks the user (the QA script denies).
target = os.path.join(tempfile.gettempdir(), "varstotas_bridge_export.stl")
open(target, "w").write("old")
try:
    m.export(target)
    check("export denied", False)
except varstotas.VarstotasError as e:
    check("export denied", "declined" in str(e) and open(target).read() == "old")

# The MCP server over stdio.
mcp = os.path.join(os.path.dirname(varstotas.__file__), "varstotas_mcp.py")
p = subprocess.Popen([sys.executable, mcp], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)


def rpc(i, method, params=None):
    p.stdin.write(json.dumps({"jsonrpc": "2.0", "id": i, "method": method, "params": params or {}}) + "\n")
    p.stdin.flush()
    return json.loads(p.stdout.readline())


init = rpc(1, "initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "qa"}})
check("mcp initialize", init["result"]["serverInfo"]["name"] == "varstotas")
p.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
tools = {t["name"] for t in rpc(2, "tools/list")["result"]["tools"]}
check("mcp tools", {"run_script", "model_info", "screenshot", "save", "api_reference"} <= tools)
r = rpc(3, "tools/call", {"name": "run_script", "arguments": {"code": "let c = cylinder(vec3(500, 300, 18), 50, 100); c.name = \"Cup\"; c.kind"}})
check("mcp run_script", not r["result"]["isError"] and '"group"' in r["result"]["content"][0]["text"])
r = rpc(4, "tools/call", {"name": "run_script", "arguments": {"code": "pushpull(1, 2)"}})
check("mcp error", r["result"]["isError"])
r = rpc(5, "tools/call", {"name": "screenshot", "arguments": {"width": 200, "height": 120}})
check("mcp screenshot", r["result"]["content"][0]["type"] == "image" and len(r["result"]["content"][0]["data"]) > 1000)
r = rpc(6, "tools/call", {"name": "api_reference", "arguments": {}})
check("mcp docs", "pushpull" in r["result"]["content"][0]["text"])
p.stdin.close()
p.wait()
check("agent step", m.info()["undo"] == "Agent" and len(m.groups()) == 2)
print("bridge checks: %d ok" % len(ok))
