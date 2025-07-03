use std::{fs, io, path::Path};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit, generic_array::GenericArray};
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use argon2::{Argon2, Params, PasswordHasher};
use zeroize::Zeroize;

const CONFIG_FILE: &str = "config.env";

#[derive(Serialize, Deserialize, Clone)]
pub struct BotConfig {
    pub vk_token: String,
    pub user_id: u64,
}

fn gen_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

fn derive_key(password: &str, pin: &str, salt: &[u8]) -> [u8; 32] {
    let mut secret = password.as_bytes().to_vec();
    secret.extend_from_slice(pin.as_bytes());

    let params = Params::new(
        65536,
        5,
        4,
        None
    ).expect("Bad Argon2 params");
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2.hash_password_into(&secret, salt, &mut key).expect("Argon2 error");
    secret.zeroize();
    key
}

pub fn save_config_secure(config: &BotConfig, password: &str, pin: &str) -> io::Result<()> {
    let salt = gen_salt();
    let key = derive_key(password, pin, &salt);
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    let config_bytes = serde_json::to_vec(config).unwrap();
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), config_bytes.as_ref()).unwrap();

    let mut out = Vec::new();
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);

    fs::write(CONFIG_FILE, general_purpose::STANDARD.encode(out))
}

pub fn load_config_secure(password: &str, pin: &str) -> io::Result<BotConfig> {
    let data = fs::read_to_string(CONFIG_FILE)?;
    let decoded = general_purpose::STANDARD.decode(data.trim()).unwrap();
    let (salt, rest) = decoded.split_at(16);
    let (nonce, ciphertext) = rest.split_at(12);

    let key = derive_key(password, pin, salt);
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
    let plain = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Неверный пароль или PIN, либо повреждён файл"))?;
    Ok(serde_json::from_slice(&plain).unwrap())
}

pub fn config_exists() -> bool {
    Path::new(CONFIG_FILE).exists()
}

pub fn delete_config() {
    let _ = fs::remove_file(CONFIG_FILE);
}