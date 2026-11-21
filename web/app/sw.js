// The app offline (a PWA): its files are cached when this worker installs, then each request
// goes to the network first, refreshing the cache, and to the cache when the network fails.
// scripts/build-web.sh fills VERSION (a hash of the build) and FILES (the files in app/): a new
// build changes this file, so the browser installs the worker again and caches the new files.
const VERSION = "__VERSION__";
const FILES = __FILES__;
const CACHE = `varstotas-${VERSION}`;

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.addAll(FILES)).then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys()
      .then((keys) => Promise.all(keys.filter((k) => k.startsWith("varstotas-") && k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET" || new URL(request.url).origin !== self.location.origin) return;
  event.respondWith(
    fetch(request)
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
