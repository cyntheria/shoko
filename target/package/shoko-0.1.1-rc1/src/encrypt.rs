use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use rand::{RngCore, rng};
use std::io;
use std::env;

fn get_encryption_key() -> io::Result<[u8; 32]> {
    let key_str = env::var("SHOKO_KEY").map_err(|_| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Encryption key not found. Please set the 'SHOKO_KEY' environment variable.",
        )
    })?;

    let key_bytes = key_str.as_bytes();
    if key_bytes.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Invalid SHOKO_KEY length: expected 32 bytes, got {}. Use a 32-character string.",
                key_bytes.len()
            ),
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(key_bytes);
    Ok(key)
}

pub fn encrypt_data(data: &[u8]) -> io::Result<Vec<u8>> {
    let raw_key = get_encryption_key()?;
    let key = Aes256Gcm::new_from_slice(&raw_key)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    let mut nonce_bytes = [0u8; 12];
    rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = key
        .encrypt(nonce, data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Encryption failed: {}", e)))?;
    let mut out = nonce_bytes.to_vec();
    out.extend(ciphertext);
    Ok(out)
}

pub fn decrypt_data(data: &[u8]) -> io::Result<Vec<u8>> {
    if data.len() < 12 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Data too short for decryption"));
    }

    let raw_key = get_encryption_key()?;
    let key = Aes256Gcm::new_from_slice(&raw_key)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = key
        .decrypt(nonce, ciphertext)
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Decryption failed (Invalid key or tampered data): {}", e),
            )
        })?;

    Ok(plaintext)
}
