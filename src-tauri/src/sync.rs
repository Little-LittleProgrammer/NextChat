// Daily Sync Module for NextChat Tauri
// Provides scheduled sync functionality

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Window;

use chrono::Timelike;

// Daily sync configuration
const SYNC_HOUR: u32 = 8; // 8:00 AM
const SYNC_MINUTE_WINDOW: u32 = 5; // Sync window: 8:00 - 8:05
const CHECK_INTERVAL_SECS: u64 = 60; // Check every minute

// Global flag to control sync thread
static SHOULD_STOP: AtomicBool = AtomicBool::new(false);

/// Get today's date string in YYYY-MM-DD format
fn get_today_string() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d").to_string()
}

/// Check if current time is within the sync window
fn is_in_sync_window() -> bool {
    let now = chrono::Local::now();
    now.hour() == SYNC_HOUR && now.minute() < SYNC_MINUTE_WINDOW
}

/// Check if today has already been synced by reading the stored date
async fn is_today_synced(app: &AppHandle) -> bool {
    let store_path = app.path_resolver().app_data_dir();

    if let Some(path) = store_path {
        let sync_date_file = path.join("daily_sync_date.txt");

        if let Ok(content) = std::fs::read_to_string(sync_date_file) {
            return content.trim() == get_today_string();
        }
    }

    false
}

/// Mark today as synced by writing the date to storage
async fn mark_today_synced(app: &AppHandle) {
    let store_path = app.path_resolver().app_data_dir();

    if let Some(path) = store_path {
        let sync_date_file = path.join("daily_sync_date.txt");

        let _ = std::fs::write(sync_date_file, get_today_string());
    }
}

/// Schedule daily sync in the background
/// This runs a thread that checks the time every minute and triggers sync
#[tauri::command]
pub async fn schedule_daily_sync(app: AppHandle, window: Window) {
    log::info!("[DailySync] Scheduling daily sync at {}:00", SYNC_HOUR);

    // Reset the stop flag
    SHOULD_STOP.store(false, Ordering::SeqCst);

    let app_handle = app.clone();
    let window_clone = window.clone();

    // Spawn a background thread for time checking
    thread::spawn(move || {
        log::info!("[DailySync] Sync thread started");

        while !SHOULD_STOP.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(CHECK_INTERVAL_SECS));

            if SHOULD_STOP.load(Ordering::SeqCst) {
                break;
            }

            // Check if we should trigger sync
            if is_in_sync_window() {
                let app_clone = app_handle.clone();

                // Check if already synced today
                let today_synced = tauri::async_runtime::block_on(async {
                    is_today_synced(&app_clone).await
                });

                if !today_synced {
                    log::info!("[DailySync] Triggering daily sync");

                    // Emit event to frontend to perform sync
                    let _ = window_clone.emit("daily-sync-trigger", ());

                    // Mark as synced
                    tauri::async_runtime::block_on(async {
                        mark_today_synced(&app_clone).await
                    });
                } else {
                    log::debug!("[DailySync] Already synced today");
                }
            }
        }

        log::info!("[DailySync] Sync thread stopped");
    });
}

/// Cancel the daily sync thread
#[tauri::command]
pub fn cancel_daily_sync() {
    log::info!("[DailySync] Cancelling daily sync");
    SHOULD_STOP.store(true, Ordering::SeqCst);
}

/// Get daily sync status
#[tauri::command]
pub async fn get_daily_sync_status(app: AppHandle) -> serde_json::Value {
    let last_sync_date = {
        let store_path = app.path_resolver().app_data_dir();
        if let Some(path) = store_path {
            let sync_date_file = path.join("daily_sync_date.txt");
            std::fs::read_to_string(sync_date_file).unwrap_or_default()
        } else {
            String::new()
        }
    };

    serde_json::json!({
        "enabled": true, // This would need to be stored/persisted
        "hour": SYNC_HOUR,
        "lastSyncDate": last_sync_date.trim(),
        "isTodaySynced": last_sync_date.trim() == get_today_string()
    })
}
