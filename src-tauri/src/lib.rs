mod crypto;
mod vault;

use std::sync::Mutex;
use vault::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState(Mutex::new(vault::VaultState::default())))
        .invoke_handler(tauri::generate_handler![
            vault::create_vault,
            vault::unlock_vault,
            vault::lock_vault,
            vault::list_entries,
            vault::add_entry,
            vault::update_entry,
            vault::delete_entry,
            vault::generate_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
