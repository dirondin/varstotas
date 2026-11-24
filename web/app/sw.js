// The app offline (a PWA): its files are cached when this worker installs, then each request
// goes to the network first, refreshing the cache, and to the cache when the network fails.
// Past the browser's own HTTP cache: it would hand back the files of the build before (a server's
// Last-Modified or GitHub Pages' ten minutes let it), and the new build would not show.
// scripts/build-web.sh fills VERSION (a hash of the build) and FILES (the files in app/): a new
// build changes this file, so the browser installs the worker again and caches the new files.
const VERSION = "__VERSION__";
const FILES = __FILES__;
// named for the worker's place too: the release's (app/) and the beta's (beta/app/) share the
// site's caches, and each clears only its own older ones (and the unplaced ones of the builds before)
const PREFIX = `varstotas-${new URL(self.registration.scope).pathname}-`;
const CACHE = PREFIX + VERSION;

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.addAll(FILES.map((f) => new Request(f, { cache: "reload" })))).then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys()
      .then((keys) => Promise.all(keys.filter((k) => (k.startsWith(PREFIX) && k !== CACHE) || /^varstotas-[0-9a-f]{16}$/.test(k)).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET" || new URL(request.url).origin !== self.location.origin) return;
  event.respondWith(
    fetch(request, { cache: "no-cache" })
      .then((response) => {
        if (response.ok) {
          const copy = response.clone();
          caches.open(CACHE).then((cache) => cache.put(request, copy));
        }
        return response;
      })
      .catch(async () => {
        const hit = await caches.match(request, { ignoreSearch: true });
        // a page asked for offline (the app's address with anything after it): the app
        return hit ?? (request.mode === "navigate" ? caches.match("./") : Response.error());
      }),
  );
});
