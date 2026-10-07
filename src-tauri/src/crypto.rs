use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

const NONCE_LEN: usize = 12;
const SALT_LEN: usize = 16;
const KEY_LEN: usize = 32;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("encryption failed")]
    Encrypt,
    #[error("decryption failed")]
    Decrypt,
    #[error("key derivation failed")]
    Kdf,
    #[error("invalid data")]
    InvalidData,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey([u8; KEY_LEN]);

impl MasterKey {
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

/// Derive a 256-bit key from the master password using Argon2id.
pub fn derive_key(password: &str, salt: &[u8]) -> Result<MasterKey, CryptoError> {
    let params = Params::new(65536, 3, 4, Some(KEY_LEN)).map_err(|_| CryptoError::Kdf)?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|_| CryptoError::Kdf)?;

    Ok(MasterKey(key))
}

pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Encrypt plaintext → base64(nonce || ciphertext)
pub fn encrypt(key: &MasterKey, plaintext: &[u8]) -> Result<String, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|_| CryptoError::Encrypt)?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::Encrypt)?;

    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);

    Ok(B64.encode(out))
}

/// Decrypt base64(nonce || ciphertext) → plaintext
pub fn decrypt(key: &MasterKey, data: &str) -> Result<Vec<u8>, CryptoError> {
    let raw = B64.decode(data).map_err(|_| CryptoError::InvalidData)?;
    if raw.len() < NONCE_LEN {
        return Err(CryptoError::InvalidData);
    }

    let (nonce_bytes, ciphertext) = raw.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|_| CryptoError::Decrypt)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::Decrypt)
}

#[derive(Serialize, Deserialize)]
pub struct EncryptedVault {
    pub version: u32,
    pub salt: String, // base64
    pub data: String, // base64(nonce || ct)
}
