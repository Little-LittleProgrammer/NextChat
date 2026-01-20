// Daily Sync Service Worker for NextChat
// Runs in the background to trigger daily sync at 8:00 AM

const SYNC_HOUR = 8; // Daily sync at 8:00 AM
const SYNC_MINUTE_WINDOW = 5; // Sync window: 8:00 - 8:05

self.addEventListener("install", () => {
  console.log("[DailySync SW] Installing...");
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  console.log("[DailySync SW] Activating...");
  event.waitUntil(self.clients.claim());
});

// Check every minute if it's time to sync
setInterval(checkAndSync, 60000);

async function checkAndSync() {
  const now = new Date();

  // Only trigger sync during the sync window (e.g., 8:00 - 8:05)
  if (now.getHours() === SYNC_HOUR && now.getMinutes() < SYNC_MINUTE_WINDOW) {
    console.log("[DailySync SW] Sync window reached, notifying clients...");

    // Notify all clients to perform sync
    const clients = await self.clients.matchAll();
    for (const client of clients) {
      client.postMessage({ type: "DAILY_SYNC_TRIGGER", timestamp: Date.now() });
    }
  }
}

self.addEventListener("message", (event) => {
  if (event.data === "SKIP_WAITING") {
    self.skipWaiting();
  }
});

// Handle push notifications (for sync completion notification)
self.addEventListener("push", (event) => {
  if (event.data?.type === "SYNC_COMPLETE") {
    event.waitUntil(
      self.registration.showNotification("同步完成", {
        body: event.data.message,
        icon: "/icon.png",
        tag: "daily-sync",
        requireInteraction: true,
      }),
    );
  }
});
