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

// Install: download every file of this build into a fresh cache.
self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then((cache) => cache.addAll(self.PRECACHE_URLS))
      .then(() => self.skipWaiting()),
  );
});

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
