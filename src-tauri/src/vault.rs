use crate::crypto::{self, EncryptedVault, MasterKey};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
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
    #[serde(default)]
    pub deleted: bool,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub last_used_at: Option<i64>,
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

pub struct AppState(pub Mutex<VaultState>);

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn bak_path(path: &Path) -> PathBuf {
    path.with_extension("laspl.bak")
}

fn tmp_path(path: &Path) -> PathBuf {
    path.with_extension("laspl.tmp")
}

/// Atomic write: write tmp → fsync → backup existing → rename tmp over target.
fn atomic_write(path: &Path, contents: &str) -> Result<(), String> {
    ensure_parent(path)?;
    let tmp = tmp_path(path);
    let bak = bak_path(path);

    {
        let mut f = File::create(&tmp).map_err(|e| format!("Failed to create temp vault: {e}"))?;
        f.write_all(contents.as_bytes())
            .map_err(|e| format!("Failed to write temp vault: {e}"))?;
        f.sync_all()
            .map_err(|e| format!("Failed to sync temp vault: {e}"))?;
    }

    if path.exists() {
        let _ = fs::remove_file(&bak);
        fs::rename(path, &bak).map_err(|e| format!("Failed to backup vault: {e}"))?;
    }

    fs::rename(&tmp, path).map_err(|e| format!("Failed to commit vault: {e}"))?;
    Ok(())
}

fn chrono_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn looks_like_seconds(ts: i64) -> bool {
    ts > 0 && ts < 1_000_000_000_000
}

fn migrate_timestamps(data: &mut VaultData) -> bool {
    let mut changed = false;
    for e in &mut data.entries {
        if looks_like_seconds(e.created_at) {
            e.created_at *= 1000;
            changed = true;
        }
        if looks_like_seconds(e.updated_at) {
            e.updated_at *= 1000;
            changed = true;
        }
        if let Some(lu) = e.last_used_at {
            if looks_like_seconds(lu) {
                e.last_used_at = Some(lu * 1000);
                changed = true;
            }
        }
    }
    changed
}

fn persist(st: &VaultState, key: &MasterKey) -> Result<(), String> {
    let plaintext = serde_json::to_vec(&st.data).map_err(|e| e.to_string())?;
    let encrypted = crypto::encrypt(key, &plaintext).map_err(|e| e.to_string())?;

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
    let json = serde_json::to_string_pretty(&vault).map_err(|e| e.to_string())?;
    atomic_write(&st.path, &json)
}

fn try_load_vault(path: &Path, password: &str) -> Result<(MasterKey, VaultData), String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let vault: EncryptedVault = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let salt = B64.decode(&vault.salt).map_err(|e| e.to_string())?;
    let key = crypto::derive_key(password, &salt).map_err(|e| e.to_string())?;
    let plaintext =
        crypto::decrypt(&key, &vault.data).map_err(|_| "Wrong password".to_string())?;
    let data: VaultData = serde_json::from_slice(&plaintext).map_err(|e| e.to_string())?;
    Ok((key, data))
}

#[tauri::command]
pub fn unlock_vault(
    password: String,
    state: State<'_, AppState>,
) -> Result<Vec<PasswordEntry>, String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;

    if !st.path.exists() {
        let bak = bak_path(&st.path);
        if bak.exists() {
            match try_load_vault(&bak, &password) {
                Ok((key, mut data)) => {
                    let migrated = migrate_timestamps(&mut data);
                    st.key = Some(key);
                    st.data = data;
                    if migrated {
                        let _ = persist(&st, st.key.as_ref().unwrap());
                    }
                    let _ = fs::copy(&bak, &st.path);
                    return Ok(st.data.entries.clone());
                }
                Err(e) if e == "Wrong password" => return Err(e),
                Err(_) => {}
            }
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
        let json = serde_json::to_string_pretty(&vault).map_err(|e| e.to_string())?;
        atomic_write(&st.path, &json)?;

        st.key = Some(key);
        st.data = data;
        return Ok(st.data.entries.clone());
    }

    let loaded = match try_load_vault(&st.path, &password) {
        Ok(v) => v,
        Err(e) if e == "Wrong password" => {
            let main_raw = fs::read_to_string(&st.path).unwrap_or_default();
            if serde_json::from_str::<EncryptedVault>(&main_raw).is_err() {
                let bak = bak_path(&st.path);
                if bak.exists() {
                    try_load_vault(&bak, &password).map_err(|_| e)?
                } else {
                    return Err(e);
                }
            } else {
                return Err(e);
            }
        }
        Err(e) => {
            let bak = bak_path(&st.path);
            if bak.exists() {
                try_load_vault(&bak, &password)
                    .map_err(|_| format!("{e} (backup also failed)"))?
            } else {
                return Err(e);
            }
        }
    };

    let (key, mut data) = loaded;
    let migrated = migrate_timestamps(&mut data);
    st.key = Some(key);
    st.data = data;
    if migrated {
        let _ = persist(&st, st.key.as_ref().unwrap());
    }
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
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

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
        deleted: false,
        created_at: now,
        updated_at: now,
        last_used_at: None,
    };

    st.data.entries.push(entry.clone());
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(entry)
}

#[tauri::command]
pub fn update_entry(entry: PasswordEntry, state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

    if let Some(existing) = st.data.entries.iter_mut().find(|e| e.id == entry.id) {
        *existing = entry;
        existing.updated_at = chrono_now();
    } else {
        return Err("Entry not found".into());
    }
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(())
}

#[tauri::command]
pub fn delete_entry(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

    if let Some(entry) = st.data.entries.iter_mut().find(|e| e.id == id) {
        entry.deleted = true;
        entry.updated_at = chrono_now();
    } else {
        return Err("Entry not found".into());
    }
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(())
}

#[tauri::command]
pub fn restore_entry(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

    if let Some(entry) = st.data.entries.iter_mut().find(|e| e.id == id) {
        entry.deleted = false;
        entry.updated_at = chrono_now();
    } else {
        return Err("Entry not found".into());
    }
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(())
}

#[tauri::command]
pub fn purge_deleted(state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

    st.data.entries.retain(|e| !e.deleted);
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(())
}

#[tauri::command]
pub fn touch_entry(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

    if let Some(entry) = st.data.entries.iter_mut().find(|e| e.id == id) {
        entry.last_used_at = Some(chrono_now());
    } else {
        return Err("Entry not found".into());
    }
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(())
}

#[tauri::command]
pub fn change_master_password(
    current_password: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if new_password.len() < 8 {
        return Err("New password must be at least 8 characters".into());
    }

    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }

    let raw = fs::read_to_string(&st.path).map_err(|e| e.to_string())?;
    let vault: EncryptedVault = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let old_salt = B64.decode(&vault.salt).map_err(|e| e.to_string())?;
    let verify_key =
        crypto::derive_key(&current_password, &old_salt).map_err(|e| e.to_string())?;
    let _ = crypto::decrypt(&verify_key, &vault.data)
        .map_err(|_| "Current password is incorrect".to_string())?;

    let new_salt = crypto::generate_salt();
    let new_key = crypto::derive_key(&new_password, &new_salt).map_err(|e| e.to_string())?;

    let plaintext = serde_json::to_vec(&st.data).map_err(|e| e.to_string())?;
    let encrypted = crypto::encrypt(&new_key, &plaintext).map_err(|e| e.to_string())?;

    let new_vault = EncryptedVault {
        version: 1,
        salt: B64.encode(new_salt),
        data: encrypted,
    };
    let json = serde_json::to_string_pretty(&new_vault).map_err(|e| e.to_string())?;
    atomic_write(&st.path, &json)?;

    if let Some(mut old) = st.key.take() {
        old.zeroize();
    }
    st.key = Some(new_key);
    Ok(())
}

#[tauri::command]
pub fn export_vault(state: State<'_, AppState>) -> Result<String, String> {
    let st = state.0.lock().map_err(|e| e.to_string())?;
    if st.key.is_none() {
        return Err("Vault is locked".into());
    }
    if !st.path.exists() {
        return Err("No vault file found".into());
    }
    fs::read_to_string(&st.path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_vault(data: String, state: State<'_, AppState>) -> Result<(), String> {
    let vault: EncryptedVault =
        serde_json::from_str(&data).map_err(|_| "Invalid vault file format".to_string())?;
    if vault.version == 0 || vault.salt.is_empty() || vault.data.is_empty() {
        return Err("Invalid vault file".into());
    }

    let mut st = state.0.lock().map_err(|e| e.to_string())?;
    atomic_write(&st.path, &data)?;

    if let Some(mut key) = st.key.take() {
        key.zeroize();
    }
    st.data = VaultData::default();
    Ok(())
}

#[tauri::command]
pub fn generate_password(
    mode: Option<String>,
    length: Option<usize>,
    symbols: Option<bool>,
    exclude_ambiguous: Option<bool>,
) -> String {
    use rand::rngs::OsRng;
    use rand::RngCore;

    let mode = mode.unwrap_or_else(|| "password".into());
    let symbols = symbols.unwrap_or(true);
    let exclude_ambiguous = exclude_ambiguous.unwrap_or(false);

    if mode == "passphrase" {
        let word_count = length.unwrap_or(5).clamp(4, 10);
        return generate_passphrase(word_count);
    }

    let len = length.unwrap_or(20).clamp(12, 128);

    let mut charset = String::from("abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789");
    if !exclude_ambiguous {
        charset.push_str("0OlI1");
    }
    if symbols {
        charset.push_str("!@#$%^&*()-_=+[]{}|;:,.<>?");
    }

    let chars: Vec<char> = charset.chars().collect();
    let mut rng = OsRng;
    let mut result = String::with_capacity(len);

    for _ in 0..len {
        let idx = (rng.next_u32() as usize) % chars.len();
        result.push(chars[idx]);
    }
    result
}

fn generate_passphrase(word_count: usize) -> String {
    use rand::rngs::OsRng;
    use rand::RngCore;

    const WORDLIST_RAW: &str = include_str!("eff_wordlist.txt");

    let words: Vec<&str> = WORDLIST_RAW
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    let mut rng = OsRng;
    let mut chosen = Vec::with_capacity(word_count);

    for _ in 0..word_count {
        let idx = (rng.next_u32() as usize) % words.len();
        chosen.push(words[idx]);
    }

    chosen.join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn atomic_write_creates_and_backs_up() {
        let dir = std::env::temp_dir().join(format!("laspl-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vault.laspl");

        atomic_write(&path, "first").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "first");
        assert!(!bak_path(&path).exists());

        atomic_write(&path, "second").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
        assert_eq!(fs::read_to_string(bak_path(&path)).unwrap(), "first");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn migrate_seconds_to_millis() {
        let mut data = VaultData {
            entries: vec![PasswordEntry {
                id: "1".into(),
                title: "t".into(),
                username: "u".into(),
                password: "p".into(),
                url: None,
                notes: None,
                favorite: false,
                category: None,
                deleted: false,
                created_at: 1_700_000_000,
                updated_at: 1_700_000_001,
                last_used_at: Some(1_700_000_002),
            }],
        };
        assert!(migrate_timestamps(&mut data));
        assert!(data.entries[0].created_at > 1_000_000_000_000);
        assert!(!migrate_timestamps(&mut data));
    }
}
