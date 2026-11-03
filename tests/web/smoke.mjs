// Smoke test of the web build in headless Chrome (WebGPU; SwiftShader where there is no GPU):
// the page starts the app, a document is opened from "disk", exported as PNG and saved, both
// come back as downloads. No dependencies: Node 22+ (fetch, WebSocket) and Chrome over CDP.
// usage: node tests/web/smoke.mjs [site dir, default target/web] [out dir, default target/web-smoke]
// CHROME=<path> picks the browser, CHROME_FLAGS adds switches, CHROME_LOG=1 shows its log. Screenshots of each step are left in the out dir.
// The clicks are at fixed points of a 1400×900 page: move them when the layout changes.
import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { extname, join, resolve } from "node:path";

const site = resolve(process.argv[2] ?? "target/web");
const out = resolve(process.argv[3] ?? "target/web-smoke");
const model = resolve("tests/ui/models/x1.vrst");
rmSync(out, { recursive: true, force: true });
const downloads = join(out, "downloads");
mkdirSync(downloads, { recursive: true });

const chrome = process.env.CHROME ?? [
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/usr/bin/google-chrome",
  "/usr/bin/chromium",
].find(existsSync);
if (!chrome) fail("no Chrome: set CHROME");

// the site, served as GitHub Pages would
const types = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".png": "image/png" };
const server = createServer((req, res) => {
  const path = join(site, decodeURIComponent(new URL(req.url, "http://x").pathname).replace(/\/$/, "/index.html"));
  if (!path.startsWith(site) || !existsSync(path)) return res.writeHead(404).end();
  res.writeHead(200, { "content-type": types[extname(path)] ?? "application/octet-stream" }).end(readFileSync(path));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const url = `http://127.0.0.1:${server.address().port}/`;

const port = 9300 + Math.floor(Math.random() * 500);
const browser = spawn(chrome, [
  "--headless=new", `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), "varstotas-"))}`,
  "--enable-unsafe-webgpu", "--enable-unsafe-swiftshader", ...(process.env.CHROME_FLAGS ?? "").split(" ").filter(Boolean), "--no-first-run", "--window-size=1400,900", ...(process.env.CHROME_LOG ? ["--enable-logging=stderr", "--v=0"] : []), "about:blank",
], { stdio: ["ignore", "ignore", process.env.CHROME_LOG ? "inherit" : "ignore"] });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let targets;
for (let i = 0; i < 100 && !targets; i++) {
  try { targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json(); } catch { await sleep(200); }
}
if (!targets) fail("Chrome did not start");
const ws = new WebSocket(targets.find((t) => t.type === "page").webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener("open", r));

let seq = 0;
const replies = new Map();
const errors = [];
const saved = [];
ws.addEventListener("message", (e) => {
  const m = JSON.parse(e.data);
  if (m.id) return replies.get(m.id)?.(m), replies.delete(m.id);
  const p = m.params;
  if (m.method === "Runtime.exceptionThrown") errors.push(p.exceptionDetails.exception?.description ?? p.exceptionDetails.text);
  if (m.method === "Runtime.consoleAPICalled" && p.type === "error") errors.push(p.args.map((a) => a.value ?? a.description).join(" "));
  if (m.method === "Page.fileChooserOpened") send("DOM.setFileInputFiles", { files: [model], backendNodeId: p.backendNodeId });
  if (m.method === "Page.javascriptDialogOpening") send("Page.handleJavaScriptDialog", { accept: true });
  if (m.method === "Page.downloadWillBegin") saved.push(p.suggestedFilename);
});
function send(method, params = {}) {
  return new Promise((r) => {
    replies.set(++seq, (m) => (m.error ? fail(`${method}: ${m.error.message}`) : r(m.result)));
    ws.send(JSON.stringify({ id: seq, method, params }));
  });
}
async function shot(name) {
  const { data } = await send("Page.captureScreenshot", { format: "png" });
  writeFileSync(join(out, `${name}.png`), Buffer.from(data, "base64"));
}
async function click(x, y, secs = 2) {
  for (const type of ["mouseMoved", "mousePressed", "mouseReleased"]) {
    await send("Input.dispatchMouseEvent", { type, x, y, button: type === "mouseMoved" ? "none" : "left", clickCount: 1 });
  }
  await sleep(secs * 1000);
}
async function key(k, modifiers) {
  for (const type of ["keyDown", "keyUp"]) {
    await send("Input.dispatchKeyEvent", { type, key: k, code: `Key${k.toUpperCase()}`, modifiers, windowsVirtualKeyCode: k.toUpperCase().charCodeAt(0) });
  }
  await sleep(3000);
}
async function evaluate(expression) {
  return (await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true })).result.value;
}
/** A downloaded file once it is complete (Chrome writes `<name>.crdownload` until then). */
async function download(name, secs = 30) {
  for (let i = 0; i < secs * 4; i++) {
    const f = join(downloads, name);
    if (existsSync(f) && !readdirSync(downloads).some((n) => n.endsWith(".crdownload"))) return f;
    await sleep(250);
  }
  fail(`no download ${name} (downloads: ${saved.join(", ") || "none"})`);
}
function check(ok, what) {
  if (!ok) fail(what);
  console.log(`ok   ${what}`);
}
function fail(what) {
  console.error(`FAIL ${what}`);
  for (const e of errors) console.error(`  page: ${e.slice(0, 800)}`);
  browser?.kill();
  process.exit(1);
}

await send("Runtime.enable");
await send("Page.enable");
await send("DOM.enable");
await send("Page.setInterceptFileChooserDialog", { enabled: true });
await send("Browser.setDownloadBehavior", { behavior: "allow", downloadPath: downloads });
await send("Emulation.setDeviceMetricsOverride", { width: 1400, height: 900, deviceScaleFactor: 1, mobile: false });

await send("Page.navigate", { url });
for (let i = 0; i < 60 && (await evaluate("location.href")) !== url; i++) await sleep(500);
const adapter = await evaluate(`(async () => { if (!navigator.gpu) return "no navigator.gpu"; const a = await navigator.gpu.requestAdapter(); return a ? "ok" : "no adapter"; })()`);
check(adapter === "ok", `WebGPU adapter (${adapter})`);
let started = false;
for (let i = 0; i < 120 && !started; i++) {
  await sleep(500);
  started = await evaluate(`(() => { const c = document.getElementById("varstotas"); const n = document.getElementById("no-webgpu"); return !!c && c.width > 0 && (!n || n.hidden || getComputedStyle(n).display === "none"); })()`);
}
await sleep(5000);
await shot("1-start");
check(started && errors.length === 0, "app started");

// start screen → Open a document… → x1.vrst
await click(553, 479, 5);
await shot("2-opened");
check(errors.length === 0, "document opened");

// File → Export → 2D Graphic (PNG)… → Export
await click(20, 15, 1);
await click(56, 309, 1);
await click(266, 309, 2);
await shot("3-export-dialog");
await click(834, 560, 1);
const png = readFileSync(await download("x1.png"));
check(png.subarray(1, 4).toString() === "PNG" && png.length > 10_000, `PNG export (${png.length} bytes)`);

// Save (Ctrl+S, Cmd+S on a Mac) → x1.vrst
await key("s", process.platform === "darwin" ? 4 : 2);
const vrst = readFileSync(await download("x1.vrst"));
check(vrst.length > 0 && statSync(model).size > 0, `save (${vrst.length} bytes)`);
await shot("4-saved");
check(errors.length === 0, "no page errors");

browser.kill();
server.close();
process.exit(0);
