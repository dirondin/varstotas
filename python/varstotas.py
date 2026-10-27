"""Python client for Varstotas: drive the running app from a script.

    import varstotas
    from varstotas import vec3, ORIGIN, X, Y, Z

    m = varstotas.connect()
    with m.operation("Table"):            # one undo step; an exception rolls it back
        top = m.box(vec3(0, 0, 720), 1200, 800, 30)
        top.name = "Table top"
    print(m.bounds(m.groups()).size)

Every function of the Rhai API (docs/SCRIPTING.md) is a method of the model object with
the same name and arguments; entities have the same properties. A call outside
`operation()` is one undo step by itself. `m.rhai(code)` runs Rhai code in one call.

The app writes its address and token to bridge.json in its data folder (macOS:
~/Library/Application Support/Varstotas, Linux: ~/.local/share/varstotas or, for the Flatpak,
~/.var/app/io.github.dirondin.Varstotas/data/varstotas, Windows: %APPDATA%\\Varstotas\\data),
next to the python/ folder it installs this module into; scripts started from the app
(File -> Run Script...) get them in VARSTOTAS_BRIDGE / VARSTOTAS_TOKEN.
No dependencies beyond the standard library.
"""

import json
import os
import socket
from contextlib import contextmanager

__all__ = ["connect", "params", "Model", "Entity", "Vec3", "vec3", "ORIGIN", "X", "Y", "Z", "VarstotasError"]


class VarstotasError(Exception):
    """An error reported by Varstotas (a failed operation, a script error, a denied request)."""


class Vec3:
    """A point / vector in millimetres."""

    __slots__ = ("x", "y", "z")

    def __init__(self, x=0.0, y=0.0, z=0.0):
        self.x, self.y, self.z = float(x), float(y), float(z)

    def __iter__(self):
        return iter((self.x, self.y, self.z))

    def __getitem__(self, i):
        return (self.x, self.y, self.z)[i]

    def __add__(self, o):
        o = _v(o)
        return Vec3(self.x + o.x, self.y + o.y, self.z + o.z)

    __radd__ = __add__

    def __sub__(self, o):
        o = _v(o)
        return Vec3(self.x - o.x, self.y - o.y, self.z - o.z)

    def __rsub__(self, o):
        return _v(o) - self

    def __mul__(self, k):
        return Vec3(self.x * k, self.y * k, self.z * k)

    __rmul__ = __mul__

    def __truediv__(self, k):
        return Vec3(self.x / k, self.y / k, self.z / k)

    def __neg__(self):
        return Vec3(-self.x, -self.y, -self.z)

    def __eq__(self, o):
        try:
            return (self - _v(o)).length() < 1e-9
        except (TypeError, ValueError):
            return NotImplemented

    def __hash__(self):
        return hash((round(self.x, 6), round(self.y, 6), round(self.z, 6)))

    def length(self):
        return (self.x * self.x + self.y * self.y + self.z * self.z) ** 0.5

    def normalize(self):
        n = self.length()
        return self / n if n > 0 else Vec3()

    def dot(self, o):
        o = _v(o)
        return self.x * o.x + self.y * o.y + self.z * o.z

    def cross(self, o):
        o = _v(o)
        return Vec3(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)

    def distance(self, o):
        return (self - _v(o)).length()

    def __repr__(self):
        return "vec3(%s, %s, %s)" % (_num(self.x), _num(self.y), _num(self.z))


def _num(c):
    s = ("%.4f" % c).rstrip("0").rstrip(".")
    return "0" if s == "-0" else s


def vec3(x, y, z=0.0):
    return Vec3(x, y, z)


def _v(o):
    if isinstance(o, Vec3):
        return o
    if isinstance(o, (list, tuple)) and len(o) in (2, 3):
        return Vec3(*o)
    raise TypeError("expected a vector, got %r" % (o,))


ORIGIN, X, Y, Z = Vec3(), Vec3(1, 0, 0), Vec3(0, 1, 0), Vec3(0, 0, 1)


class Entity:
    """A face, edge, vertex or group of the model. Properties (`area`, `normal`, `name`,
    `vertices`, ...) are read from the app on access; `name` can be assigned."""

    def __init__(self, model, eid, kind):
        object.__setattr__(self, "_m", model)
        object.__setattr__(self, "id", eid)
        object.__setattr__(self, "_kind", kind)

    def __getattr__(self, prop):
        if prop.startswith("_") or not prop.isidentifier():
            raise AttributeError(prop)
        if prop == "kind" and self._kind != "?":
            return self._kind
        return self._m._eval("e.%s" % prop, {"e": self}, change=False)

    def __setattr__(self, prop, value):
        if not prop.isidentifier():
            raise AttributeError(prop)
        self._m._eval("e.%s = v" % prop, {"e": self, "v": value})

    def __eq__(self, o):
        return isinstance(o, Entity) and o.id == self.id

    def __hash__(self):
        return hash(self.id)

    def __repr__(self):
        return "%s(%s)" % (self._kind, self.id)


class Model:
    """A connection to the running app. Use `connect()`."""

    def __init__(self, host, port, token, timeout=None):
        self._sock = socket.create_connection((host, port), timeout=10)
        self._sock.settimeout(timeout)
        self._r = self._sock.makefile("r", encoding="utf-8")
        self._token = token
        self._id = 0
        self._in_op = False
        hello = self.request("hello")
        self.api = list(hello.get("api", []))
        self.version = hello.get("version")

    # ---- protocol -----------------------------------------------------------

    def request(self, method, **params):
        """Send one bridge request and return its result (raises VarstotasError)."""
        self._id += 1
        msg = {"id": self._id, "method": method, "params": params}
        if self._id == 1:
            msg["token"] = self._token
        self._sock.sendall((json.dumps(msg, default=_enc) + "\n").encode("utf-8"))
        line = self._r.readline()
        if not line:
            raise VarstotasError("the connection to Varstotas was closed")
        resp = json.loads(line, object_hook=self._dec)
        if "error" in resp:
            raise VarstotasError(resp["error"].get("message", "error"))
        return resp.get("result")

    def _dec(self, d):
        if "$vec" in d:
            return Vec3(*d["$vec"])
        if "$entity" in d:
            return Entity(self, d["$entity"], d.get("kind", "?"))
        return d

    def _eval(self, code, vars=None, label="Python", change=True):
        r = self.request("eval", code=code, vars=vars or {}, label=label)
        out = r.get("output") or ""
        if out:
            print(out, end="" if out.endswith("\n") else "\n")
        return r.get("value")

    def close(self):
        self._sock.close()

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.close()

    # ---- API ------------------------------------------------------------------

    def call(self, name, *args):
        """Call a Rhai API function by name: `m.call("box", ORIGIN, 10, 10, 10)`."""
        if not name.isidentifier():
            raise ValueError("bad function name %r" % name)
        names = ["a%d" % i for i in range(len(args))]
        return self._eval("%s(%s)" % (name, ", ".join(names)), dict(zip(names, args)), label="Python: " + name)

    def __getattr__(self, name):
        if name.startswith("_") or name not in self.__dict__.get("api", ()):
            raise AttributeError(name)
        return lambda *args: self.call(name, *args)

    def __dir__(self):
        return list(super().__dir__()) + self.api

    def rhai(self, code, label="Python", **vars):
        """Run Rhai code; keyword arguments become variables. Returns the last value."""
        return self._eval(code, vars, label)

    @contextmanager
    def operation(self, label="Python"):
        """Group calls into one undo step; an exception inside rolls everything back."""
        if self._in_op:
            yield self
            return
        self.request("begin", label=label)
        self._in_op = True
        try:
            yield self
        except BaseException:
            self._in_op = False
            self.request("abort")
            raise
        self._in_op = False
        self.request("commit")

    # ---- app ------------------------------------------------------------------

    def info(self):
        """Document summary: name, path, units, counts, bounds, materials, tags, selection."""
        return self.request("info")

    def undo(self):
        return self.request("undo")

    def redo(self):
        return self.request("redo")

    def view(self, name):
        """Standard view: iso, top, bottom, front, back, left, right, or zoom_extents."""
        return self.request("view", name=name)

    def camera(self, eye=None, target=None, up=None, fov=None, parallel=None):
        """Read the camera (no arguments) or move it."""
        p = {k: v for k, v in dict(eye=eye, target=target, up=up, fov=fov, parallel=parallel).items() if v is not None}
        return self.request("camera", **p)

    def screenshot(self, path=None, width=None, height=None):
        """Render the view to a PNG; returns its path (a temp file when `path` is None)."""
        p = {k: v for k, v in dict(path=path and os.path.abspath(path), width=width, height=height).items() if v}
        return self.request("screenshot", **p)["path"]

    def save(self, path=None):
        """Save the document (to `path`, or its current file). Overwriting another file asks the user."""
        return self.request("save", **({"path": os.path.abspath(path)} if path else {}))

    def export(self, path):
        """Export by extension: stl, obj, dxf, glb, 3mf, dae, ply, csv, svg, pdf, png."""
        return self.request("export", path=os.path.abspath(path))

    def open(self, path):
        """Open a .vrst file (asks the user when the current document has changes)."""
        return self.request("open", path=os.path.abspath(path))

    def new(self):
        """Start an empty document (asks the user when the current one has changes)."""
        return self.request("new")


def _enc(o):
    if isinstance(o, Vec3):
        return {"$vec": [o.x, o.y, o.z]}
    if isinstance(o, Entity):
        return {"$entity": o.id}
    if isinstance(o, (set, frozenset)):
        return list(o)
    raise TypeError("cannot send %r to Varstotas" % (o,))


def params():
    """Parameters of a plugin run (its @param values, `points`, `path`) as a dict."""

    def dec(d):
        if "$vec" in d:
            return Vec3(*d["$vec"])
        return d

    return json.loads(os.environ.get("VARSTOTAS_PARAMS") or "{}", object_hook=dec)


def lang():
    """The UI language code of Varstotas ("en", "ru", …) for the texts a plugin prints or shows."""
    return os.environ.get("VARSTOTAS_UI_LANG") or "en"


def data_dirs():
    """The places of the app's data folder, the likeliest first (see the module's docstring)."""
    home = os.path.expanduser("~")
    dirs = [os.environ.get("VARSTOTAS_DATA_DIR"),
            # this module installed by the app: <data folder>/python/varstotas.py
            os.path.dirname(os.path.dirname(os.path.abspath(__file__)))]
    if os.name == "nt":
        dirs.append(os.path.join(os.environ.get("APPDATA", ""), "Varstotas", "data"))
    elif os.uname().sysname == "Darwin":
        dirs.append(os.path.join(home, "Library", "Application Support", "Varstotas"))
    else:
        dirs.append(os.path.join(os.environ.get("XDG_DATA_HOME") or os.path.join(home, ".local", "share"), "varstotas"))
        dirs.append(os.path.join(home, ".var", "app", "io.github.dirondin.Varstotas", "data", "varstotas"))
    return [d for d in dirs if d]


def bridge_file():
    """The bridge.json of the running app (the first data folder that has one), or None."""
    for d in data_dirs():
        path = os.path.join(d, "bridge.json")
        if os.path.isfile(path):
            return path
    return None


def connect(host=None, port=None, token=None, timeout=None):
    """Connect to the running Varstotas (VARSTOTAS_BRIDGE / VARSTOTAS_TOKEN, else bridge.json)."""
    if port is None and os.environ.get("VARSTOTAS_BRIDGE"):
        h, _, p = os.environ["VARSTOTAS_BRIDGE"].rpartition(":")
        host, port = host or h, int(p)
        token = token or os.environ.get("VARSTOTAS_TOKEN")
    if port is None or token is None:
        path = bridge_file()
        try:
            with open(path or "", encoding="utf-8") as f:
                info = json.load(f)
        except OSError:
            raise VarstotasError("Varstotas is not running (no bridge.json in %s)" % ", ".join(data_dirs())) from None
        host, port, token = host or info["host"], port or info["port"], token or info["token"]
    try:
        return Model(host or "127.0.0.1", port, token, timeout)
    except ConnectionRefusedError:
        raise VarstotasError("Varstotas is not running (nothing listens on port %s)" % port) from None
