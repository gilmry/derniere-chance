// App-shell service worker: the routes below are precached so the PWA still
// opens with no network. Navigations try the network first so users get
// fresh content when online; other same-origin assets are cache-first.
// Dynamic routes (/offre, /marchand, /reservation) aren't precached here —
// they get cached on first visit by the fetch handler below.

const CACHE_NAME = "derniere-chance-shell-v1";
const APP_SHELL = [
  "/",
  "/feed",
  "/profil",
  "/pro/login",
  "/pro/panier/nouveau",
  "/pro/dashboard",
  "/manifest.json",
];

self.addEventListener("install", (event) => {
  event.waitUntil(caches.open(CACHE_NAME).then((cache) => cache.addAll(APP_SHELL)));
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))),
      )
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET" || new URL(request.url).origin !== self.location.origin) {
    return;
  }

  if (request.mode === "navigate") {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const copy = response.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(request, copy));
          return response;
        })
        .catch(() => caches.match(request).then((cached) => cached || caches.match("/"))),
    );
    return;
  }

  event.respondWith(caches.match(request).then((cached) => cached || fetch(request)));
});
