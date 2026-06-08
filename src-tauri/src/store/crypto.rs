/// API Key 加解密（AES-256-GCM，密钥由设备标识派生）

use data_encoding::BASE64_NOPAD;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::digest::{digest, SHA256};
use ring::rand::{SecureRandom, SystemRandom};

fn derive_key() -> [u8; 32] {
    let machine = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "ds-monitor".to_string());
    let seed = format!("ds-monitor:v1:{machine}");
    let hash = digest(&SHA256, seed.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(hash.as_ref());
    key
}

fn cipher_key() -> Result<LessSafeKey, String> {
    let key_bytes = derive_key();
    let unbound =
        UnboundKey::new(&AES_256_GCM, &key_bytes).map_err(|e| format!("密钥创建失败: {e}"))?;
    Ok(LessSafeKey::new(unbound))
}

/// 加密 API Key
pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let rng = SystemRandom::new();
    let key = cipher_key()?;

    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| format!("Nonce 生成失败: {e}"))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| format!("加密失败: {e}"))?;

    let mut result = Vec::with_capacity(nonce_bytes.len() + in_out.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);

    Ok(BASE64_NOPAD.encode(&result))
}

/// 解密 API Key
pub fn decrypt(ciphertext_b64: &str) -> Result<String, String> {
    let data = BASE64_NOPAD
        .decode(ciphertext_b64.as_bytes())
        .map_err(|e| format!("Base64 解码失败: {e}"))?;

    if data.len() < 12 {
        return Err("密文格式错误".to_string());
    }

    let (nonce_bytes, encrypted) = data.split_at(12);
    let nonce = Nonce::assume_unique_for_key(
        nonce_bytes.try_into().map_err(|_| "Nonce 长度错误")?,
    );

    let key = cipher_key()?;
    let mut in_out = encrypted.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "解密失败，请重新输入 API Key".to_string())?;

    String::from_utf8(plaintext.to_vec()).map_err(|e| format!("UTF-8 解码失败: {e}"))
}
