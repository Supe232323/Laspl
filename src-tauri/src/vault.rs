use crate::crypto::{self, EncryptedVault, MasterKey};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::fs;
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

pub struct AppState(pub Mutex<VaultState>);

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn unlock_vault(
    password: String,
    state: State<'_, AppState>,
) -> Result<Vec<PasswordEntry>, String> {
    let mut st = state.0.lock().map_err(|e| e.to_string())?;

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
        created_at: now,
        updated_at: now,
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

    st.data.entries.retain(|e| e.id != id);
    persist(&st, st.key.as_ref().unwrap())?;
    Ok(())
}

/// Cryptographically secure password / passphrase generator.
///
/// - Uses `OsRng` (OS entropy: /dev/urandom, BCryptGenRandom, etc.)
/// - Random mode follows NIST SP 800-63B spirit (min 12 chars, mixed charset)
/// - Passphrase mode: Diceware-style from the full EFF long wordlist (7776 words)
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

    // Random password mode
    let len = length.unwrap_or(20).clamp(12, 128); // NIST-aligned minimum

    let mut charset = String::from("abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789");
    // Default already excludes some ambiguous (0/O, 1/l/I) for better usability.
    // If exclude_ambiguous is false we add the rest back for maximum entropy.
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

    // Full EFF long wordlist (7776 words), embedded at compile time.
    // One word per line in eff_wordlist.txt — no dice codes.
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
