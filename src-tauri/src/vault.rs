use crate::crypto::{self, EncryptedVault, MasterKey};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, Clone)]
pub struct PasswordEntry {
    pub id: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub category: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Default)]
struct VaultData {
    entries: Vec<PasswordEntry>,
}

pub struct VaultState {
    key: Option<MasterKey>,
    data: VaultData,
    path: PathBuf,
}

impl Default for VaultState {
    fn default() -> Self {
        let path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("laspl")
            .join("vault.laspl");
        Self {
            key: None,
            data: VaultData::default(),
            path,
        }
    }
}

// Global state wrapper
pub struct AppState(pub Mutex<VaultState>);

fn ensure_parent(path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn create_vault(
    password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.path.exists() {
        return Err("Vault already exists".into());
    }

    let salt = crypto::generate_salt();
    let key = crypto::derive_key(&password, &salt).map_err(|e| e.to_string())?;

    let data = VaultData::default();
    let plaintext = serde_json::to_vec(&data).map_err(|e| e.to_string())?;
    let encrypted = crypto::encrypt(&key, &plaintext).map_err(|e| e.to_string())?;

    let vault = EncryptedVault {
        version: 1,
        salt: B64.encode(salt),
        data: encrypted,
    };

    ensure_parent(&st.path)?;
    let json = serde_json::to_string_pretty(&vault).map_err(|e| e.to_string())?;
    fs::write(&st.path, json).map_err(|e| e.to_string())?;

    st.key = Some(key);
    st.data = data;
    Ok(())
}

#[tauri::command]
pub fn unlock_vault(
    password: String,
    state: State<'_, AppState>,
) -> Result<Vec<PasswordEntry>, String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;

    // First run: create empty vault in-place
    if !st.path.exists() {
        let salt = crypto::generate_salt();
        let key = crypto::derive_key(&password, &salt).map_err(|e| e.to_string())?;
        let data = VaultData::default();
        let plaintext = serde_json::to_vec(&data).map_err(|e| e.to_string())?;
        let encrypted = crypto::encrypt(&key, &plaintext).map_err(|e| e.to_string())?;

        let vault = EncryptedVault {
            version: 1,
            salt: B64.encode(salt),
            data: encrypted,
        };
        ensure_parent(&st.path)?;
        let json = serde_json::to_string_pretty(&vault).map_err(|e| e.to_string())?;
        fs::write(&st.path, json).map_err(|e| e.to_string())?;

        st.key = Some(key);
        st.data = data;
        return Ok(st.data.entries.clone());
    }

    let raw = fs::read_to_string(&st.path).map_err(|e| e.to_string())?;
    let vault: EncryptedVault = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    let salt = B64.decode(&vault.salt).map_err(|e| e.to_string())?;
    let key = crypto::derive_key(&password, &salt).map_err(|e| e.to_string())?;

    let plaintext = crypto::decrypt(&key, &vault.data).map_err(|_| "Wrong password".to_string())?;
    let data: VaultData = serde_json::from_slice(&plaintext).map_err(|e| e.to_string())?;

    st.key = Some(key);
    st.data = data;
    Ok(st.data.entries.clone())
}

#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(mut key) = st.key.take() {
        key.zeroize();
    }
    st.data = VaultData::default();
    Ok(())
}

#[tauri::command]
pub fn list_entries(state: State<'_, AppState>) -> Result<Vec<PasswordEntry>, String> {
    let st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }
    Ok(st.data.entries.clone())
}

#[tauri::command]
pub fn add_entry(
    title: String,
    username: String,
    password: String,
    url: Option<String>,
    notes: Option<String>,
    category: Option<String>,
    state: State<'_, AppState>,
) -> Result<PasswordEntry, String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    let key = st.key.as_ref().ok_or("Vault is locked")?;

    let now = chrono_now();
    let entry = PasswordEntry {
        id: Uuid::new_v4().to_string(),
        title,
        username,
        password,
        url,
        notes,
        favorite: false,
        category,
        created_at: now,
        updated_at: now,
    };

    st.data.entries.push(entry.clone());
    persist(&st, key)?;
    Ok(entry)
}

#[tauri::command]
pub fn update_entry(entry: PasswordEntry, state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    let key = st.key.as_ref().ok_or("Vault is locked")?;

    if let Some(existing) = st.data.entries.iter_mut().find(|e| e.id == entry.id) {
        *existing = entry;
        existing.updated_at = chrono_now();
    } else {
        return Err("Entry not found".into());
    }
    persist(&st, key)?;
    Ok(())
}

#[tauri::command]
pub fn delete_entry(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    let key = st.key.as_ref().ok_or("Vault is locked")?;

    st.data.entries.retain(|e| e.id != id);
    persist(&st, key)?;
    Ok(())
}

#[tauri::command]
pub fn generate_password(length: usize, symbols: bool) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string();
    if symbols {
        charset.push_str("!@#$%^&*()-_=+[]{}|;:,.<>?");
    }
    let chars: Vec<char> = charset.chars().collect();
    (0..length.max(8).min(128))
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

fn persist(st: &VaultState, key: &MasterKey) -> Result<(), String> {
    let plaintext = serde_json::to_vec(&st.data).map_err(|e| e.to_string())?;
    let encrypted = crypto::encrypt(key, &plaintext).map_err(|e| e.to_string())?;

    // Re-read salt from existing file or generate new (simplified)
    let salt = if st.path.exists() {
        let raw = fs::read_to_string(&st.path).unwrap_or_default();
        if let Ok(v) = serde_json::from_str::<EncryptedVault>(&raw) {
            v.salt
        } else {
            B64.encode(crypto::generate_salt())
        }
    } else {
        B64.encode(crypto::generate_salt())
    };

    let vault = EncryptedVault {
        version: 1,
        salt,
        data: encrypted,
    };
    ensure_parent(&st.path)?;
    let json = serde_json::to_string_pretty(&vault).map_err(|e| e.to_string())?;
    fs::write(&st.path, json).map_err(|e| e.to_string())?;
    Ok(())
}

fn chrono_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
