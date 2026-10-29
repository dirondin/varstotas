"""MCP server for Varstotas (stdio): lets an AI agent read and edit the model in the running app.

    claude mcp add varstotas -- python3 "<data folder>/python/varstotas_mcp.py"

(<data folder>: macOS ~/Library/Application Support/Varstotas, Linux ~/.local/share/varstotas,
Flatpak ~/.var/app/io.github.dirondin.Varstotas/data/varstotas, Windows %APPDATA%\\Varstotas\\data;
Preferences → General shows it.) The app must be running with Preferences → General → "Let
scripts and AI agents outside Varstotas connect" on (off by default; it then writes bridge.json
there and installs this file next to varstotas.py). Every edit is one undo step in the app; saving over another file, exporting
over an existing file, opening a file or starting a new one with unsaved changes asks the
user in the app first. No dependencies beyond the standard library.
"""

import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import varstotas  # noqa: E402

PROTOCOL = "2025-06-18"

INSTRUCTIONS = (
    "Varstotas is a 3D modeller. Build and edit geometry with run_script (Rhai code; "
    "call api_reference first for the functions). Coordinates are world millimetres, Z is up. "
    "Each run_script call is one undo step; an error rolls back model changes. Host file and network functions are unavailable in run_script. Check results with "
    "model_info and screenshot."
)


def _obj(props, required=()):
    return {"type": "object", "properties": props, "required": list(required)}


TOOLS = [
    {
        "name": "api_reference",
        "description": "The scripting reference (functions, properties, examples). Read it before run_script.",
        "inputSchema": _obj({}),
        "annotations": {"readOnlyHint": True},
    },
    {
        "name": "model_info",
        "description": "Summary of the open document: file, units, entity counts, size, materials, tags, groups, selection, camera.",
        "inputSchema": _obj({}),
        "annotations": {"readOnlyHint": True},
    },
    {
        "name": "cut_list",
        "description": (
            "The cut list of the model: parts (leaf groups / components with length, width, thickness, "
            "material), the sheets and boards to buy per material with waste % and cost, the pieces' "
            "positions on each, edge banding by part thickness, hardware to buy, and issues (no stock, too big, thickness). Materials need stock first: "
            "run_script with set_stock(material, #{kind: \"sheet\", length: 2440, width: 1220, "
            "thickness: 18, grain: true, price: 60}) or a board #{kind: \"board\", width: 95, "
            "thickness: 45, lengths: [2400, 3600], price: 5} (price per metre), or linear stock "
            "#{kind: \"linear\", name: \"Ø 20 rod\", width: 20, thickness: 20, lengths: [1000, 2000], price: 3}. "
            "Millimetres."
        ),
        "inputSchema": _obj({}),
        "annotations": {"readOnlyHint": True},
    },
    {
        "name": "run_script",
        "description": (
            "Run Rhai code against the model (see api_reference), e.g. "
            "`let b = box(ORIGIN, 600, 400, 18); b.name = \"Shelf\"; bounds(b)`. "
            "Returns printed output and the value of the last expression; entities come back as "
            "{\"$entity\": id, \"kind\": …}. One undo step; an error changes nothing."
        ),
        "inputSchema": _obj(
            {
                "code": {"type": "string", "description": "Rhai source"},
                "label": {"type": "string", "description": "Undo step name (default: Agent)"},
            },
            ["code"],
        ),
    },
    {
        "name": "screenshot",
        "description": "Render the current view to a PNG image.",
        "inputSchema": _obj({"width": {"type": "integer"}, "height": {"type": "integer"}}),
        "annotations": {"readOnlyHint": True},
    },
    {
        "name": "set_view",
        "description": "Move the camera: a standard view (iso, top, bottom, front, back, left, right, zoom_extents) and/or eye/target points in mm.",
        "inputSchema": _obj(
            {
                "view": {"type": "string", "enum": ["iso", "top", "bottom", "front", "back", "left", "right", "zoom_extents"]},
                "eye": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3},
                "target": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3},
                "parallel": {"type": "boolean", "description": "Parallel projection"},
            }
        ),
    },
    {
        "name": "undo",
        "description": "Undo the last step (yours or the user's).",
        "inputSchema": _obj({}),
    },
    {
        "name": "redo",
        "description": "Redo the last undone step.",
        "inputSchema": _obj({}),
    },
    {
        "name": "save",
        "description": "Save the document (to `path`, or to its current file). Saving over another existing file asks the user.",
        "inputSchema": _obj({"path": {"type": "string", "description": "Absolute .vrst path"}}),
        "annotations": {"destructiveHint": True},
    },
    {
        "name": "export",
        "description": "Export the model; the format follows the extension: stl, obj, dxf, glb, 3mf, dae (COLLADA, images next to it), ply, csv (parts list), svg/pdf (2D drawing of the view), png (image). With `what` a workshop document instead: cut_list (pdf/svg/csv: cutting diagrams, parts, costs), labels (pdf: part labels), assembly (pdf: the assembly instructions booklet; set the steps first with suggest_steps() in run_script). Overwriting a file asks the user.",
        "inputSchema": _obj({"path": {"type": "string", "description": "Absolute path"},
                             "what": {"type": "string", "enum": ["model", "cut_list", "labels", "assembly"], "description": "What to export (default model)"}}, ["path"]),
        "annotations": {"destructiveHint": True},
    },
    {
        "name": "open",
        "description": "Open a .vrst file in place of the current document (asks the user when it has unsaved changes).",
        "inputSchema": _obj({"path": {"type": "string"}}, ["path"]),
        "annotations": {"destructiveHint": True},
    },
    {
        "name": "new_document",
        "description": "Start an empty document (asks the user when the current one has unsaved changes).",
        "inputSchema": _obj({}),
        "annotations": {"destructiveHint": True},
    },
]


def _out(o):
    """JSON for values the client decoded (vectors as [x, y, z], entities with their kind)."""
    if isinstance(o, varstotas.Vec3):
        return [o.x, o.y, o.z]
    if isinstance(o, varstotas.Entity):
        return {"$entity": o.id, "kind": o._kind}
    raise TypeError(repr(o))


class Server:
    def __init__(self):
        self.m = None

    def model(self):
        if self.m is None:
            self.m = varstotas.connect()
        return self.m

    def bridge(self, method, **params):
        try:
            return self.model().request(method, **params)
        except (OSError, ValueError):
            # The app was restarted (new port / token): reconnect once.
            self.m = None
            return self.model().request(method, **params)

    def tool(self, name, a):
        text = lambda v: [{"type": "text", "text": v if isinstance(v, str) else json.dumps(v, indent=1, default=_out)}]
        if name == "api_reference":
            return text(self.bridge("docs"))
        if name == "model_info":
            return text(self.bridge("info"))
        if name == "cut_list":
            return text(self.bridge("eval", code="cut_list()", vars={}, label="Cut list").get("value"))
        if name == "run_script":
            r = self.bridge("eval", code=a["code"], vars={}, label=a.get("label") or "Agent")
            out = r.get("output") or ""
            return text(out + "value: " + json.dumps(r.get("value"), default=_out))
        if name == "screenshot":
            p = {k: a[k] for k in ("width", "height") if a.get(k)}
            r = self.bridge("screenshot", base64=True, **p)
            return [{"type": "image", "data": r["png_base64"], "mimeType": "image/png"}]
        if name == "set_view":
            if a.get("view"):
                self.bridge("view", name=a["view"])
            cam = {k: a[k] for k in ("eye", "target", "parallel") if k in a}
            return text(self.bridge("camera", **cam))
        if name in ("undo", "redo"):
            return text(self.bridge(name) or "nothing to %s" % name)
        if name in ("save", "export", "open"):
            return text(self.bridge(name, **{k: v for k, v in a.items() if k == "path" or (k == "what" and name == "export")}))
        if name == "new_document":
            return text(self.bridge("new"))
        raise varstotas.VarstotasError("unknown tool %s" % name)

    def handle(self, req):
        method, rid = req.get("method"), req.get("id")
        if method == "initialize":
            v = (req.get("params") or {}).get("protocolVersion") or PROTOCOL
            return {
                "protocolVersion": v,
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "varstotas", "version": "0.1"},
                "instructions": INSTRUCTIONS,
            }
        if method == "ping":
            return {}
        if method == "tools/list":
            return {"tools": TOOLS}
        if method == "tools/call":
            p = req.get("params") or {}
            try:
                return {"content": self.tool(p.get("name"), p.get("arguments") or {}), "isError": False}
            except (varstotas.VarstotasError, OSError, KeyError) as e:
                return {"content": [{"type": "text", "text": "Error: %s" % e}], "isError": True}
        if rid is None:
            return None  # a notification
        raise LookupError(method)


def main():
    srv = Server()
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except ValueError:
            continue
        rid = req.get("id")
        try:
            res = srv.handle(req)
            if rid is None:
                continue
            resp = {"jsonrpc": "2.0", "id": rid, "result": res}
        except LookupError as e:
            resp = {"jsonrpc": "2.0", "id": rid, "error": {"code": -32601, "message": "method not found: %s" % e}}
        except Exception as e:  # keep serving
            resp = {"jsonrpc": "2.0", "id": rid, "error": {"code": -32603, "message": str(e)}}
        sys.stdout.write(json.dumps(resp) + "\n")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
