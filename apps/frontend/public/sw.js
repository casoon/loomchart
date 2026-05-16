// LoomChart Service Worker — Web Push notification handler
'use strict';

const CACHE_NAME = 'loomchart-v1';

// ---------------------------------------------------------------------------
// Install & activate
// ---------------------------------------------------------------------------
self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', e => e.waitUntil(self.clients.claim()));

// ---------------------------------------------------------------------------
// Push event — show notification
// ---------------------------------------------------------------------------
self.addEventListener('push', event => {
  if (!event.data) return;

  let data;
  try {
    data = event.data.json();
  } catch {
    data = { title: 'LoomChart Alert', body: event.data.text() };
  }

  const title = data.title || 'LoomChart Alert';
  const options = {
    body:    data.body  || 'An alert has been triggered.',
    icon:    data.icon  || '/favicon.ico',
    badge:   '/favicon.ico',
    data:    data.data  || {},
    actions: [
      { action: 'open',    title: 'Open Chart' },
      { action: 'dismiss', title: 'Dismiss' },
    ],
    requireInteraction: false,
    tag: `loom-alert-${data.data?.alert_id || Date.now()}`,
    renotify: true,
  };

  event.waitUntil(self.registration.showNotification(title, options));
});

// ---------------------------------------------------------------------------
// Notification click — focus or open the app
// ---------------------------------------------------------------------------
self.addEventListener('notificationclick', event => {
  event.notification.close();

  if (event.action === 'dismiss') return;

  const symbol = event.notification.data?.symbol || '';
  const url = symbol ? `/?symbol=${encodeURIComponent(symbol)}` : '/';

  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then(clients => {
      for (const client of clients) {
        if ('focus' in client) {
          client.postMessage({ type: 'alert-clicked', symbol });
          return client.focus();
        }
      }
      return self.clients.openWindow(url);
    })
  );
});
