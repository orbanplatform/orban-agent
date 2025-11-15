//! 認證模組 - 處理 Agent 身份驗證

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use crate::error::{Error, Result};
use std::fs;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};

/// 認證器
pub struct Authenticator {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    agent_id: String,
}

impl Authenticator {
    /// 從私鑰文件創建認證器
    pub fn from_private_key_file<P: AsRef<Path>>(path: P, agent_id: String) -> Result<Self> {
        let secret_bytes = fs::read(path)?;
        if secret_bytes.len() != 32 {
            return Err(Error::InvalidConfig(
                "Private key must be 32 bytes".to_string(),
            ));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&secret_bytes);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self { signing_key, verifying_key, agent_id })
    }

    /// 生成新的密鑰對
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut rng = OsRng;
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);

        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        let agent_id = Self::derive_agent_id(&verifying_key);

        Self { signing_key, verifying_key, agent_id }
    }

    /// 從公鑰推導 Agent ID
    fn derive_agent_id(public_key: &VerifyingKey) -> String {
        let mut hasher = Sha256::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();

        // 使用哈希的前 16 字節作為 ID
        let id_bytes = &hash[..16];
        format!("agent-{}", hex::encode(id_bytes))
    }

    /// 保存私鑰到文件
    pub fn save_private_key<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::write(path, self.signing_key.as_bytes())?;
        Ok(())
    }

    /// 獲取 Agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// 獲取公鑰 (base64 編碼)
    pub fn public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.verifying_key.as_bytes())
    }

    /// 簽署挑戰
    pub fn sign_challenge(&self, challenge: &[u8]) -> String {
        let signature = self.signing_key.sign(challenge);
        general_purpose::STANDARD.encode(signature.to_bytes())
    }

    /// 驗證簽名
    pub fn verify_signature(&self, message: &[u8], signature: &str) -> Result<bool> {
        let signature_bytes = general_purpose::STANDARD.decode(signature)
            .map_err(|e| Error::EncryptionError(e.to_string()))?;

        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|e| Error::EncryptionError(e.to_string()))?;

        match self.verifying_key.verify(message, &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 響應認證挑戰
    pub fn respond_to_challenge(&self, challenge: &str) -> Result<(String, String)> {
        // 解碼挑戰
        let challenge_bytes = general_purpose::STANDARD.decode(challenge)
            .map_err(|e| Error::EncryptionError(e.to_string()))?;

        // 簽署挑戰
        let signature = self.sign_challenge(&challenge_bytes);
        let public_key = self.public_key_base64();

        Ok((signature, public_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticator_generation() {
        let auth = Authenticator::generate();
        println!("Agent ID: {}", auth.agent_id());
        println!("Public Key: {}", auth.public_key_base64());

        assert!(auth.agent_id().starts_with("agent-"));
    }

    #[test]
    fn test_sign_and_verify() {
        let auth = Authenticator::generate();
        let message = b"test message";

        let signature = auth.sign_challenge(message);
        let verified = auth.verify_signature(message, &signature).unwrap();

        assert!(verified);
    }

    #[test]
    fn test_challenge_response() {
        let auth = Authenticator::generate();
        let challenge = general_purpose::STANDARD.encode(b"random_challenge_data");

        let (signature, public_key) = auth.respond_to_challenge(&challenge).unwrap();

        assert!(!signature.is_empty());
        assert!(!public_key.is_empty());
    }
}
