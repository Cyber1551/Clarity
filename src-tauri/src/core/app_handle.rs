use std::sync::OnceLock;
use tauri::AppHandle;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_handle(handle: AppHandle) {
    let _ = APP_HANDLE.set(handle);
}

pub fn get_handle() -> &'static AppHandle {
    APP_HANDLE.get().expect("AppHandle not initialized")
}
