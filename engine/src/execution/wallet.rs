use std::collections::HashMap;
use std::sync::RwLock;

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, AeadCore, Nonce};
use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use sha2::{Sha256, Digest};
use zeroize::Zeroize;

/// Stores AES-256-GCM encrypted wallet private keys, decrypting only at signing time.
/// Also stores the associated Gnosis Safe address for each wallet (used as `maker` in orders).
///
/// PHP storage format (matched by this module):
///   base64( iv_12_bytes || gcm_tag_16_bytes || ciphertext )
///
/// The AES key is derived as `SHA-256(encryption_key_string)[0..32]`, matching
/// PHP's `substr(hash('sha256', $key, true), 0, 32)`.
pub struct WalletKeyStore {
    keys: RwLock<HashMap<u64, Vec<u8>>>,
    safe_addresses: RwLock<HashMap<u64, Address>>,
    cipher: Aes256Gcm,
}

impl WalletKeyStore {
    /// Creates a new key store. The encryption key is hashed with SHA-256 to derive
    /// the AES-256 key, matching the PHP WalletService encryption.
    pub fn new(encryption_key: &str) -> Result<Self> {
        anyhow::ensure!(
            encryption_key.len() >= 32,
            "encryption key must be at least 32 characters, got {}",
            encryption_key.len()
        );

        let key_bytes = Sha256::digest(encryption_key.as_bytes());

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .context("failed to create AES-256-GCM cipher")?;

        Ok(Self {
            keys: RwLock::new(HashMap::new()),
            safe_addresses: RwLock::new(HashMap::new()),
            cipher,
        })
    }

    /// Stores an encrypted private key (base64-encoded) for a wallet.
    pub fn store_key(&self, wallet_id: u64, encrypted_b64: &str) -> Result<()> {
        let raw = BASE64
            .decode(encrypted_b64)
            .context("invalid base64 in encrypted key")?;

        anyhow::ensure!(
            raw.len() > 12,
            "encrypted payload too short (must contain 12-byte nonce + ciphertext)"
        );

        let mut keys = self
            .keys
            .write()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))?;

        keys.insert(wallet_id, raw);
        Ok(())
    }

    /// Decrypts the stored key and returns a `PrivateKeySigner`.
    ///
    /// Handles PHP format: `iv(12) || tag(16) || ciphertext`.
    /// The `aes_gcm` crate expects `ciphertext || tag`, so we rearrange.
    ///
    /// The decrypted bytes are zeroized immediately after signer creation.
    pub fn get_signer(&self, wallet_id: u64) -> Result<PrivateKeySigner> {
        let keys = self
            .keys
            .read()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))?;

        let encrypted = keys
            .get(&wallet_id)
            .with_context(|| format!("no key stored for wallet {wallet_id}"))?;

        anyhow::ensure!(
            encrypted.len() >= 28,
            "encrypted payload too short (need iv + tag + ciphertext, got {} bytes)",
            encrypted.len()
        );

        // PHP layout: iv(12) || tag(16) || ciphertext
        let nonce = Nonce::from_slice(&encrypted[..12]);
        let tag = &encrypted[12..28];
        let ciphertext = &encrypted[28..];

        // aes_gcm expects: ciphertext || tag
        let mut ct_with_tag = Vec::with_capacity(ciphertext.len() + 16);
        ct_with_tag.extend_from_slice(ciphertext);
        ct_with_tag.extend_from_slice(tag);

        let mut decrypted = self
            .cipher
            .decrypt(nonce, ct_with_tag.as_slice())
            .map_err(|e| anyhow::anyhow!("decryption failed: {e}"))?;

        // PHP encrypts the hex string of the private key, not raw bytes.
        // Convert from hex string to 32-byte raw key for the signer.
        let hex_str = std::str::from_utf8(&decrypted)
            .context("decrypted key is not valid UTF-8 (expected hex string)")?;
        let mut key_bytes = hex::decode(hex_str)
            .context("decrypted key is not valid hex")?;

        let signer = PrivateKeySigner::from_slice(&key_bytes)
            .context("failed to create signer from decrypted key")?;

        decrypted.zeroize();
        key_bytes.zeroize();

        Ok(signer)
    }

    /// Returns the Ethereum address for the stored wallet key.
    #[allow(dead_code)]
    pub fn get_address(&self, wallet_id: u64) -> Result<Address> {
        let signer = self.get_signer(wallet_id)?;
        Ok(signer.address())
    }

    /// Encrypts a raw private key hex string with a random nonce.
    /// Returns base64-encoded payload in PHP format: `iv(12) || tag(16) || ciphertext`.
    ///
    /// The private key bytes are first hex-encoded (matching PHP's plaintext format),
    /// then encrypted.
    #[allow(dead_code)]
    pub fn encrypt_key(&self, private_key_bytes: &[u8]) -> Result<String> {
        let hex_string = hex::encode(private_key_bytes);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        // aes_gcm produces ciphertext || tag (last 16 bytes)
        let ct_with_tag = self
            .cipher
            .encrypt(&nonce, hex_string.as_bytes())
            .map_err(|e| anyhow::anyhow!("encryption failed: {e}"))?;

        let tag_start = ct_with_tag.len() - 16;
        let ciphertext = &ct_with_tag[..tag_start];
        let tag = &ct_with_tag[tag_start..];

        // PHP format: iv(12) || tag(16) || ciphertext
        let mut payload = nonce.to_vec();
        payload.extend_from_slice(tag);
        payload.extend_from_slice(ciphertext);

        Ok(BASE64.encode(&payload))
    }

    /// Returns whether a key is stored for the given wallet.
    #[allow(dead_code)]
    pub fn has_key(&self, wallet_id: u64) -> bool {
        self.keys
            .read()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))
            .map(|keys| keys.contains_key(&wallet_id))
            .unwrap_or(false)
    }

    /// Removes the stored key for a wallet.
    #[allow(dead_code)]
    pub fn remove_key(&self, wallet_id: u64) {
        match self.keys.write() {
            Ok(mut keys) => { keys.remove(&wallet_id); }
            Err(e) => tracing::warn!(wallet_id, error = %e, "remove_key_lock_poisoned"),
        }
    }

    /// Stores the Gnosis Safe address associated with a wallet.
    pub fn store_safe_address(&self, wallet_id: u64, safe_address: Address) -> Result<()> {
        let mut addrs = self
            .safe_addresses
            .write()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))?;
        addrs.insert(wallet_id, safe_address);
        Ok(())
    }

    /// Returns the Gnosis Safe address for the given wallet.
    pub fn get_safe_address(&self, wallet_id: u64) -> Result<Address> {
        let addrs = self
            .safe_addresses
            .read()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))?;
        addrs
            .get(&wallet_id)
            .copied()
            .with_context(|| format!("no safe_address stored for wallet {wallet_id}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = "test-encryption-key-at-least-32-chars-long";

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let store = WalletKeyStore::new(TEST_KEY).unwrap();

        // A valid 32-byte private key
        let private_key = [0xABu8; 32];
        let encrypted_b64 = store.encrypt_key(&private_key).unwrap();

        store.store_key(1, &encrypted_b64).unwrap();
        let signer = store.get_signer(1);
        assert!(signer.is_ok(), "round-trip decrypt should succeed");
    }

    #[test]
    fn test_missing_wallet_key() {
        let store = WalletKeyStore::new(TEST_KEY).unwrap();
        let result = store.get_signer(999);
        assert!(result.is_err(), "should error for nonexistent wallet");
    }

    #[test]
    fn test_has_key() {
        let store = WalletKeyStore::new(TEST_KEY).unwrap();
        assert!(!store.has_key(1));

        let encrypted = store.encrypt_key(&[0xCDu8; 32]).unwrap();
        store.store_key(1, &encrypted).unwrap();
        assert!(store.has_key(1));
    }

    #[test]
    fn test_remove_key() {
        let store = WalletKeyStore::new(TEST_KEY).unwrap();

        let encrypted = store.encrypt_key(&[0xEFu8; 32]).unwrap();
        store.store_key(1, &encrypted).unwrap();
        assert!(store.has_key(1));

        store.remove_key(1);
        assert!(!store.has_key(1));
    }

    #[test]
    fn test_invalid_encryption_key() {
        let short = WalletKeyStore::new("short");
        assert!(short.is_err(), "short key should fail");
    }

    #[test]
    fn test_safe_address_store_and_get() {
        let store = WalletKeyStore::new(TEST_KEY).unwrap();
        let addr: Address = "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b"
            .parse()
            .unwrap();

        assert!(store.get_safe_address(1).is_err());
        store.store_safe_address(1, addr).unwrap();
        assert_eq!(store.get_safe_address(1).unwrap(), addr);
    }

    #[test]
    fn test_address_derivation() {
        let store = WalletKeyStore::new(TEST_KEY).unwrap();

        // Hardhat account #0
        let private_key_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let private_key_bytes = hex::decode(private_key_hex).unwrap();

        let encrypted = store.encrypt_key(&private_key_bytes).unwrap();
        store.store_key(42, &encrypted).unwrap();

        let address = store.get_address(42).unwrap();
        let expected: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
            .parse()
            .unwrap();

        assert_eq!(address, expected, "should derive Hardhat #0 address");
    }
}
