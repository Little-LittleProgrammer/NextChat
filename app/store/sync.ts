import { getClientConfig } from "../config/client";
import { ApiPath, STORAGE_KEY, StoreKey } from "../constant";
import { createPersistStore } from "../utils/store";
import {
  AppState,
  getLocalAppState,
  GetStoreState,
  mergeAppState,
  setLocalAppState,
} from "../utils/sync";
import { downloadAs, readFromFile } from "../utils";
import { showToast } from "../components/ui-lib";
import Locale from "../locales";
import { createSyncClient, ProviderType } from "../utils/cloud";
import { isTauriEnv } from "../mcp/env";

export interface WebDavConfig {
  server: string;
  username: string;
  password: string;
}

const isApp = !!getClientConfig()?.isApp;
export type SyncStore = GetStoreState<typeof useSyncStore>;

const DEFAULT_SYNC_STATE = {
  provider: ProviderType.WebDAV,
  useProxy: true,
  proxyUrl: ApiPath.Cors as string,

  webdav: {
    endpoint: "",
    username: "",
    password: "",
  },

  upstash: {
    endpoint: "",
    username: STORAGE_KEY,
    apiKey: "",
  },

  lastSyncTime: 0,
  lastProvider: "",

  // Daily sync configuration
  dailySyncEnabled: false,
  lastDailySyncDate: "",
  syncTimer: null as number | null,
};

export const useSyncStore = createPersistStore(
  DEFAULT_SYNC_STATE,
  (set, get) => ({
    cloudSync() {
      const config = get()[get().provider];
      return Object.values(config).every((c) => c.toString().length > 0);
    },

    markSyncTime() {
      set({ lastSyncTime: Date.now(), lastProvider: get().provider });
    },

    export() {
      const state = getLocalAppState();
      const datePart = isApp
        ? `${new Date().toLocaleDateString().replace(/\//g, "_")} ${new Date()
            .toLocaleTimeString()
            .replace(/:/g, "_")}`
        : new Date().toLocaleString();

      const fileName = `Backup-${datePart}.json`;
      downloadAs(JSON.stringify(state), fileName);
    },

    async import() {
      const rawContent = await readFromFile();

      try {
        const remoteState = JSON.parse(rawContent) as AppState;
        const localState = getLocalAppState();
        mergeAppState(localState, remoteState);
        setLocalAppState(localState);
        location.reload();
      } catch (e) {
        console.error("[Import]", e);
        showToast(Locale.Settings.Sync.ImportFailed);
      }
    },

    getClient() {
      const provider = get().provider;
      const client = createSyncClient(provider, get());
      return client;
    },

    async sync() {
      const localState = getLocalAppState();
      const provider = get().provider;
      const config = get()[provider];
      const client = this.getClient();

      try {
        const remoteState = await client.get(config.username);
        if (!remoteState || remoteState === "") {
          await client.set(config.username, JSON.stringify(localState));
          console.log(
            "[Sync] Remote state is empty, using local state instead.",
          );
          return;
        } else {
          const parsedRemoteState = JSON.parse(
            await client.get(config.username),
          ) as AppState;
          mergeAppState(localState, parsedRemoteState);
          setLocalAppState(localState);
        }
      } catch (e) {
        console.log("[Sync] failed to get remote state", e);
        throw e;
      }

      await client.set(config.username, JSON.stringify(localState));

      this.markSyncTime();
    },

    async check() {
      const client = this.getClient();
      return await client.check();
    },

    // Daily sync methods
    checkDailySync(): boolean {
      const state = get();
      const now = new Date();
      const today = now.toISOString().split("T")[0];

      return (
        state.dailySyncEnabled &&
        this.cloudSync() &&
        state.lastDailySyncDate !== today &&
        now.getHours() === 8 &&
        now.getMinutes() < 5
      );
    },

    _markDailySyncComplete() {
      const today = new Date().toISOString().split("T")[0];
      set({ lastDailySyncDate: today });
    },

    scheduleDailySync() {
      // Bind sync function to current state
      const doSync = () => {
        if (this.checkDailySync()) {
          console.log("[DailySync] Triggering daily sync...");
          this.sync()
            .then(() => {
              set({ lastDailySyncDate: new Date().toISOString().split("T")[0] });
              console.log("[DailySync] Daily sync completed");
            })
            .catch((e: any) => {
              console.error("[DailySync] Daily sync failed:", e);
            });
        }
      };

      if (isTauriEnv()) {
        // Tauri: Use Rust backend for scheduling
        import("@tauri-apps/api/tauri").then((tauri) => {
          tauri.invoke("schedule_daily_sync").catch(console.error);

          // Listen for sync trigger events from Rust
          import("@tauri-apps/api/event").then((event) => {
            event.listen("daily-sync-trigger", () => {
              doSync();
            });
          });
        });
      } else {
        // Web: Use Service Worker and setInterval
        if ("serviceWorker" in navigator) {
          navigator.serviceWorker
            .register("/sw-daily-sync.js")
            .then((registration) => {
              console.log("[DailySync] SW registered:", registration.scope);
            })
            .catch((err) => {
              console.error("[DailySync] SW registration failed:", err);
            });

          // Listen for SW messages
          navigator.serviceWorker?.addEventListener("message", (event) => {
            if (event.data.type === "DAILY_SYNC_TRIGGER") {
              doSync();
            }
          });
        }

        // Check every minute when page is open
        const timer = window.setInterval(doSync, 60000);
        set({ syncTimer: timer });
      }
    },

    cancelDailySync() {
      const timer = get().syncTimer;
      if (timer) {
        clearInterval(timer);
        set({ syncTimer: null });
      }

      if (isTauriEnv()) {
        import("@tauri-apps/api/tauri").then((tauri) => {
          tauri.invoke("cancel_daily_sync").catch(console.error);
        });
      }
    },
  }),
  {
    name: StoreKey.Sync,
    version: 1.3,

    migrate(persistedState, version) {
      const newState = persistedState as typeof DEFAULT_SYNC_STATE;

      if (version < 1.1) {
        newState.upstash.username = STORAGE_KEY;
      }

      if (version < 1.2) {
        if (
          (persistedState as typeof DEFAULT_SYNC_STATE).proxyUrl ===
          "/api/cors/"
        ) {
          newState.proxyUrl = "";
        }
      }

      // Version 1.3: Add daily sync fields (new fields have defaults, no migration needed)

      return newState as any;
    },
  },
);
