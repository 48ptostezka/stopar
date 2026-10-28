// Service worker: keeps a copy of every app file on the phone, so the game
// starts and runs with no network after the first visit.
//
// For Rust readers: a service worker is a small script the browser runs
// beside the page. It sees every request the page makes (the "fetch" event)
// and can answer from its own cache. It is registered from Rust in release
// builds only (platform::register_service_worker).
//
// The file list comes from precache-manifest.js, written at build time by
// scripts/post_build.sh. Each build has its own cache name, so an update
// installs the new files and then deletes the old cache.

importScripts("precache-manifest.js");

const CACHE = "stopa-" + self.PRECACHE_VERSION;

// Install: download every file of this build into its cache.
//
// All or nothing: the new version goes live only once every file is saved,
// so a half-downloaded game never replaces a working one. On a slow
// connection the download may be cut off (the phone goes offline, the
// browser stops the worker). Files already saved stay in this version's
// cache, so the next attempt, on the next visit online, fetches only the
// rest instead of starting over.
//
// While downloading, the worker tells open pages how far it is (Settings
// shows it), and if a file fails, the console names it.
//
// `cache: "reload"` skips the browser's ordinary web cache, so a file changed
// on the server (a picture with the same name) is fetched fresh.
self.addEventListener("install", (event) => {
  event.waitUntil(precache().then(() => self.skipWaiting()));
});

async function precache() {
  const cache = await caches.open(CACHE);
  const urls = self.PRECACHE_URLS;
  const saved = new Set((await cache.keys()).map((request) => request.url));
  const missing = urls.filter((url) => !saved.has(new URL(url, self.location).href));
  let done = urls.length - missing.length;
  await report("progress", done, urls.length);

  // A few downloads at a time: quicker than one by one, gentler than all at once.
  const queue = missing.slice();
  const failures = [];
  const worker = async () => {
    for (let url = queue.shift(); url !== undefined; url = queue.shift()) {
      try {
        const response = await fetch(new Request(url, { cache: "reload" }));
        if (!response.ok) throw new Error(`answered ${response.status}`);
        await cache.put(url, response);
        done += 1;
        await report("progress", done, urls.length);
      } catch (error) {
        failures.push(`${url}: ${error.message}`);
      }
    }
  };
  await Promise.all([worker(), worker(), worker(), worker()]);

  if (failures.length > 0) {
    console.error("Stopa offline copy incomplete:\n" + failures.join("\n"));
    await report("interrupted", done, urls.length);
    throw new Error("offline copy incomplete");
  }
}

// Sends {type: "stopa-offline", state, done, total} to every open page of the
// game, including pages this worker doesn't control yet.
async function report(state, done, total) {
  const pages = await self.clients.matchAll({ type: "window", includeUncontrolled: true });
  for (const page of pages) page.postMessage({ type: "stopa-offline", state, done, total });
}

// Activate: remove caches from older builds and take over open pages.
self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((names) =>
        Promise.all(
          names
            .filter((name) => name.startsWith("stopa-") && name !== CACHE)
            .map((name) => caches.delete(name)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

// Fetch: answer from the cache first, then the network. Page loads that
// miss the cache while offline get the app's start page.
self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET" || new URL(request.url).origin !== self.location.origin) {
    return;
  }
  event.respondWith(
    caches.match(request, { ignoreSearch: true }).then(
      (cached) =>
        cached ||
        fetch(request).catch(() =>
          request.mode === "navigate" ? caches.match("./") : Response.error(),
        ),
    ),
  );
});
