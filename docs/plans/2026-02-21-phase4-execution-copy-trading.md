# Phase 4 — Execution Queue + Copy Trading Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the execution pipeline that turns strategy signals into real Polymarket CLOB orders, and the copy trading system that detects leader trades and replicates them across follower wallets.

**Architecture:** Strategy engine signals and copy trading signals flow into a shared priority queue. A single executor loop pops orders by priority, applies per-wallet token bucket rate limiting, signs EIP-712 orders with alloy, submits to the Polymarket CLOB API, and updates position state + writes trades to PostgreSQL via sqlx.

**Tech Stack:** Rust, Tokio, alloy (EIP-712), aes-gcm (key decryption), sqlx (PostgreSQL), reqwest (HTTP), uuid

---

## Task 1: Add dependencies to Cargo.toml

**Files:**
- Modify: `engine/Cargo.toml`

### Step 1: Add new dependencies

In `engine/Cargo.toml`, add these dependencies after the existing ones:

```toml
alloy = { version = "0.14", features = ["signers", "signer-local", "sol-types"] }
aes-gcm = "0.10"
zeroize = { version = "1", features = ["derive"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "time", "uuid"] }
uuid = { version = "1", features = ["v4", "serde"] }
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
base64 = "0.22"
rand = "0.8"
```

### Step 2: Verify compilation

```bash
cd engine && cargo check 2>&1
```

Expected: compiles (no code uses these yet, but deps resolve).

### Step 3: Commit

```bash
git add engine/Cargo.toml engine/Cargo.lock
git commit -m "chore: add Phase 4 dependencies — alloy, aes-gcm, sqlx, hmac"
```

---

## Task 2: Execution types

Core types shared across the execution subsystem.

**Files:**
- Create: `engine/src/execution/mod.rs`
- Modify: `engine/src/main.rs` — register module

### Step 1: Create execution module with types

Create `engine/src/execution/mod.rs`:

```rust
pub mod fees;
pub mod orders;
pub mod queue;
pub mod wallet;

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::strategy::{OrderType, Outcome};

/// Priority levels for the execution queue (higher = executed first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum OrderPriority {
    Limit = 0,
    StrategyMarket = 1,
    CopyMarket = 2,
    TakeProfit = 3,
    StopLoss = 4,
}

impl PartialOrd for OrderPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

/// Side of the order (buy outcome tokens or sell them).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

/// An order queued for execution against the Polymarket CLOB.
#[derive(Debug, Clone)]
pub struct ExecutionOrder {
    pub id: uuid::Uuid,
    pub wallet_id: u64,
    pub strategy_id: Option<u64>,
    pub copy_relationship_id: Option<u64>,
    pub symbol: String,
    pub token_id: String,
    pub side: Side,
    pub outcome: Outcome,
    pub price: Option<f64>,
    pub size_usdc: f64,
    pub order_type: OrderType,
    pub priority: OrderPriority,
    pub created_at: i64,
}

impl ExecutionOrder {
    /// Build from a strategy signal.
    pub fn from_signal(
        wallet_id: u64,
        strategy_id: u64,
        symbol: &str,
        token_id: &str,
        outcome: Outcome,
        size_usdc: f64,
        order_type: &OrderType,
    ) -> Self {
        let (side, priority) = match order_type {
            OrderType::Market => (Side::Buy, OrderPriority::StrategyMarket),
            OrderType::Limit { .. } => (Side::Buy, OrderPriority::Limit),
            OrderType::StopLoss { .. } => (Side::Sell, OrderPriority::StopLoss),
            OrderType::TakeProfit { .. } => (Side::Sell, OrderPriority::TakeProfit),
        };
        let price = match order_type {
            OrderType::Limit { price } => Some(*price),
            OrderType::StopLoss { trigger_price } => Some(*trigger_price),
            OrderType::TakeProfit { trigger_price } => Some(*trigger_price),
            OrderType::Market => None,
        };
        Self {
            id: uuid::Uuid::new_v4(),
            wallet_id,
            strategy_id: Some(strategy_id),
            copy_relationship_id: None,
            symbol: symbol.to_string(),
            token_id: token_id.to_string(),
            side,
            outcome,
            price,
            size_usdc,
            order_type: order_type.clone(),
            priority,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Result of a submitted order.
#[derive(Debug, Clone)]
pub struct OrderResult {
    pub polymarket_order_id: String,
    pub status: OrderStatus,
    pub filled_price: Option<f64>,
    pub fee_bps: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Filled,
    Cancelled,
    Failed,
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(OrderPriority::StopLoss > OrderPriority::TakeProfit);
        assert!(OrderPriority::TakeProfit > OrderPriority::CopyMarket);
        assert!(OrderPriority::CopyMarket > OrderPriority::StrategyMarket);
        assert!(OrderPriority::StrategyMarket > OrderPriority::Limit);
    }

    #[test]
    fn test_from_signal_market_buy() {
        let order = ExecutionOrder::from_signal(
            1, 100, "btc-up", "tok123", Outcome::Up, 50.0, &OrderType::Market,
        );
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.priority, OrderPriority::StrategyMarket);
        assert!(order.price.is_none());
        assert_eq!(order.strategy_id, Some(100));
    }

    #[test]
    fn test_from_signal_stoploss() {
        let order = ExecutionOrder::from_signal(
            1, 100, "btc-up", "tok123", Outcome::Up, 50.0,
            &OrderType::StopLoss { trigger_price: 0.45 },
        );
        assert_eq!(order.side, Side::Sell);
        assert_eq!(order.priority, OrderPriority::StopLoss);
        assert_eq!(order.price, Some(0.45));
    }
}
```

### Step 2: Register module in main.rs

In `engine/src/main.rs`, add after `mod strategy;`:

```rust
mod execution;
```

### Step 3: Create empty submodule files

Create placeholder files so the module compiles:

`engine/src/execution/queue.rs`:
```rust
// Execution queue — implemented in Task 3
```

`engine/src/execution/wallet.rs`:
```rust
// Wallet key store — implemented in Task 5
```

`engine/src/execution/fees.rs`:
```rust
// Fee rate cache — implemented in Task 6
```

`engine/src/execution/orders.rs`:
```rust
// CLOB order submission — implemented in Task 7
```

### Step 4: Run tests

```bash
cd engine && cargo test execution 2>&1
```

Expected: all tests pass.

### Step 5: Commit

```bash
git add engine/src/execution/ engine/src/main.rs
git commit -m "feat: add execution module with core types — ExecutionOrder, OrderPriority"
```

---

## Task 3: Token bucket rate limiter + priority queue

**Files:**
- Modify: `engine/src/execution/queue.rs`

### Step 1: Write failing tests

Replace `engine/src/execution/queue.rs` with:

```rust
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::cmp::Reverse;
use std::time::Instant;

use super::{ExecutionOrder, OrderPriority};

/// Per-wallet token bucket rate limiter.
pub struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(max_per_day: u32) -> Self {
        let max_tokens = max_per_day as f64;
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate: max_tokens / 86400.0,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    #[cfg(test)]
    pub fn with_tokens(max_per_day: u32, tokens: f64) -> Self {
        Self {
            tokens,
            max_tokens: max_per_day as f64,
            refill_rate: max_per_day as f64 / 86400.0,
            last_refill: Instant::now(),
        }
    }
}

/// Wrapper to sort ExecutionOrder by priority (highest first).
struct PriorityOrder(ExecutionOrder);

impl PartialEq for PriorityOrder {
    fn eq(&self, other: &Self) -> bool {
        self.0.priority == other.0.priority && self.0.created_at == other.0.created_at
    }
}

impl Eq for PriorityOrder {}

impl PartialOrd for PriorityOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then older orders first (FIFO within same priority)
        self.0
            .priority
            .cmp(&other.0.priority)
            .then_with(|| other.0.created_at.cmp(&self.0.created_at))
    }
}

/// Thread-safe execution queue with priority ordering and per-wallet rate limiting.
pub struct ExecutionQueue {
    heap: BinaryHeap<PriorityOrder>,
    rate_limiters: HashMap<u64, TokenBucket>,
    max_orders_per_day: u32,
}

impl ExecutionQueue {
    pub fn new(max_orders_per_day: u32) -> Self {
        Self {
            heap: BinaryHeap::new(),
            rate_limiters: HashMap::new(),
            max_orders_per_day,
        }
    }

    pub fn push(&mut self, order: ExecutionOrder) {
        tracing::debug!(
            wallet_id = order.wallet_id,
            priority = ?order.priority,
            "execution_queue_push"
        );
        self.heap.push(PriorityOrder(order));
    }

    pub fn pop(&mut self) -> Option<ExecutionOrder> {
        self.heap.pop().map(|po| po.0)
    }

    pub fn try_rate_limit(&mut self, wallet_id: u64) -> bool {
        self.rate_limiters
            .entry(wallet_id)
            .or_insert_with(|| TokenBucket::new(self.max_orders_per_day))
            .try_consume()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{OrderPriority, Side};
    use crate::strategy::{OrderType, Outcome};

    fn make_order(priority: OrderPriority, created_at: i64) -> ExecutionOrder {
        ExecutionOrder {
            id: uuid::Uuid::new_v4(),
            wallet_id: 1,
            strategy_id: Some(1),
            copy_relationship_id: None,
            symbol: "btc".into(),
            token_id: "tok".into(),
            side: Side::Buy,
            outcome: Outcome::Up,
            price: None,
            size_usdc: 50.0,
            order_type: OrderType::Market,
            priority,
            created_at,
        }
    }

    #[test]
    fn test_priority_ordering_stoploss_first() {
        let mut q = ExecutionQueue::new(3000);
        q.push(make_order(OrderPriority::Limit, 1));
        q.push(make_order(OrderPriority::StopLoss, 2));
        q.push(make_order(OrderPriority::StrategyMarket, 3));

        assert_eq!(q.pop().unwrap().priority, OrderPriority::StopLoss);
        assert_eq!(q.pop().unwrap().priority, OrderPriority::StrategyMarket);
        assert_eq!(q.pop().unwrap().priority, OrderPriority::Limit);
    }

    #[test]
    fn test_fifo_within_same_priority() {
        let mut q = ExecutionQueue::new(3000);
        q.push(make_order(OrderPriority::StrategyMarket, 100));
        q.push(make_order(OrderPriority::StrategyMarket, 50)); // older

        let first = q.pop().unwrap();
        assert_eq!(first.created_at, 50); // older first
    }

    #[test]
    fn test_token_bucket_allows_burst() {
        let mut bucket = TokenBucket::new(3000);
        // Fresh bucket should allow many orders
        for _ in 0..100 {
            assert!(bucket.try_consume());
        }
    }

    #[test]
    fn test_token_bucket_exhaustion() {
        let mut bucket = TokenBucket::with_tokens(3000, 2.0);
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(!bucket.try_consume()); // exhausted
    }

    #[test]
    fn test_rate_limit_per_wallet() {
        let mut q = ExecutionQueue::new(3000);
        // wallet 1 should be rate limited independently of wallet 2
        assert!(q.try_rate_limit(1));
        assert!(q.try_rate_limit(2));
    }
}
```

### Step 2: Run tests

```bash
cd engine && cargo test execution::queue 2>&1
```

Expected: all pass.

### Step 3: Commit

```bash
git add engine/src/execution/queue.rs
git commit -m "feat: add priority queue with per-wallet token bucket rate limiter"
```

---

## Task 4: Config updates for execution

**Files:**
- Modify: `engine/src/config.rs`

### Step 1: Add execution config fields

In `engine/src/config.rs`, add fields to the `Config` struct:

```rust
pub struct Config {
    // ... existing fields ...
    pub redis_url: String,
    pub sources: Vec<MarketSource>,
    pub tick_interval_ms: u64,
    pub discovery_interval_secs: u64,
    // NEW — execution config
    pub database_url: String,
    pub clob_api_url: String,
    pub data_api_url: String,
    pub builder_api_key: String,
    pub builder_secret: String,
    pub builder_passphrase: String,
    pub encryption_key: String,
    pub max_orders_per_day: u32,
}
```

Add to `from_env()`, after the existing redis_url line:

```rust
            // Execution config
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://oddex:oddex_secret@localhost:5432/oddex".into()),
            clob_api_url: std::env::var("POLYMARKET_CLOB_URL")
                .unwrap_or_else(|_| "https://clob.polymarket.com".into()),
            data_api_url: std::env::var("POLYMARKET_DATA_API_URL")
                .unwrap_or_else(|_| "https://data-api.polymarket.com".into()),
            builder_api_key: std::env::var("POLYMARKET_BUILDER_API_KEY")
                .unwrap_or_default(),
            builder_secret: std::env::var("POLYMARKET_BUILDER_SECRET")
                .unwrap_or_default(),
            builder_passphrase: std::env::var("POLYMARKET_BUILDER_PASSPHRASE")
                .unwrap_or_default(),
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .unwrap_or_default(),
            max_orders_per_day: std::env::var("ENGINE_MAX_ORDERS_PER_DAY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
```

### Step 2: Verify compilation

```bash
cd engine && cargo check 2>&1
```

Expected: compiles.

### Step 3: Commit

```bash
git add engine/src/config.rs
git commit -m "feat: add execution config — CLOB API, builder keys, rate limits"
```

---

## Task 5: Wallet key store + AES-256-GCM decryption

**Files:**
- Modify: `engine/src/execution/wallet.rs`

### Step 1: Implement wallet key store

Replace `engine/src/execution/wallet.rs`:

```rust
use std::collections::HashMap;
use std::sync::RwLock;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result};
use zeroize::Zeroize;

/// Encrypted wallet private key.
/// Storage format: base64(nonce_12_bytes || ciphertext || tag_16_bytes)
#[derive(Debug, Clone)]
struct EncryptedKey {
    data: Vec<u8>, // raw bytes: nonce || ciphertext || tag
}

/// Manages encrypted wallet private keys.
/// Decryption happens only within `sign_order()` — the decrypted key
/// never leaves that scope and is zeroed immediately after use.
pub struct WalletKeyStore {
    keys: RwLock<HashMap<u64, EncryptedKey>>,
    cipher: Aes256Gcm,
}

impl WalletKeyStore {
    /// Create from hex-encoded 32-byte encryption key.
    pub fn new(encryption_key_hex: &str) -> Result<Self> {
        let key_bytes = hex::decode(encryption_key_hex)
            .context("ENCRYPTION_KEY must be valid hex")?;
        anyhow::ensure!(key_bytes.len() == 32, "ENCRYPTION_KEY must be 32 bytes (64 hex chars)");

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow::anyhow!("invalid encryption key: {e}"))?;

        Ok(Self {
            keys: RwLock::new(HashMap::new()),
            cipher,
        })
    }

    /// Store an encrypted private key for a wallet.
    /// `encrypted_b64` is base64(nonce || ciphertext || tag).
    pub fn store_key(&self, wallet_id: u64, encrypted_b64: &str) -> Result<()> {
        let data = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            encrypted_b64,
        ).context("invalid base64 in encrypted key")?;
        anyhow::ensure!(data.len() > 12, "encrypted key too short");

        let mut keys = self.keys.write().unwrap();
        keys.insert(wallet_id, EncryptedKey { data });
        tracing::info!(wallet_id, "wallet_key_stored");
        Ok(())
    }

    /// Decrypt the private key, create a signer, and return it.
    /// The decrypted bytes are zeroed after the signer is created.
    pub fn get_signer(&self, wallet_id: u64) -> Result<PrivateKeySigner> {
        let keys = self.keys.read().unwrap();
        let encrypted = keys
            .get(&wallet_id)
            .context("wallet key not found")?;

        let nonce = Nonce::from_slice(&encrypted.data[..12]);
        let ciphertext = &encrypted.data[12..];

        let mut decrypted = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("decryption failed: {e}"))?;

        let signer = PrivateKeySigner::from_slice(&decrypted)
            .context("invalid private key bytes")?;

        // Zero the decrypted key immediately
        decrypted.zeroize();

        Ok(signer)
    }

    /// Get the address for a stored wallet.
    pub fn get_address(&self, wallet_id: u64) -> Result<Address> {
        let signer = self.get_signer(wallet_id)?;
        Ok(signer.address())
    }

    /// Encrypt a private key (for testing / initial key storage).
    pub fn encrypt_key(&self, private_key_bytes: &[u8]) -> Result<String> {
        use aes_gcm::aead::OsRng;
        use aes_gcm::AeadCore;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher
            .encrypt(&nonce, private_key_bytes)
            .map_err(|e| anyhow::anyhow!("encryption failed: {e}"))?;

        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce);
        combined.extend_from_slice(&ciphertext);

        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
    }

    pub fn has_key(&self, wallet_id: u64) -> bool {
        self.keys.read().unwrap().contains_key(&wallet_id)
    }

    pub fn remove_key(&self, wallet_id: u64) {
        self.keys.write().unwrap().remove(&wallet_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key_hex() -> String {
        // 32 random bytes as hex (64 chars)
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let store = WalletKeyStore::new(&test_key_hex()).unwrap();

        // Generate a test private key (32 bytes)
        let private_key = [0xABu8; 32];
        let encrypted = store.encrypt_key(&private_key).unwrap();

        store.store_key(1, &encrypted).unwrap();

        let signer = store.get_signer(1);
        assert!(signer.is_ok());
    }

    #[test]
    fn test_missing_wallet_key() {
        let store = WalletKeyStore::new(&test_key_hex()).unwrap();
        assert!(store.get_signer(999).is_err());
    }

    #[test]
    fn test_has_key() {
        let store = WalletKeyStore::new(&test_key_hex()).unwrap();
        assert!(!store.has_key(1));

        let encrypted = store.encrypt_key(&[0xABu8; 32]).unwrap();
        store.store_key(1, &encrypted).unwrap();
        assert!(store.has_key(1));
    }

    #[test]
    fn test_remove_key() {
        let store = WalletKeyStore::new(&test_key_hex()).unwrap();
        let encrypted = store.encrypt_key(&[0xABu8; 32]).unwrap();
        store.store_key(1, &encrypted).unwrap();

        store.remove_key(1);
        assert!(!store.has_key(1));
    }

    #[test]
    fn test_invalid_encryption_key() {
        assert!(WalletKeyStore::new("too_short").is_err());
        assert!(WalletKeyStore::new("not_hex_gggg").is_err());
    }

    #[test]
    fn test_address_derivation() {
        let store = WalletKeyStore::new(&test_key_hex()).unwrap();
        // Known private key → known address
        let pk_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let pk_bytes = hex::decode(pk_hex).unwrap();
        let encrypted = store.encrypt_key(&pk_bytes).unwrap();
        store.store_key(1, &encrypted).unwrap();

        let address = store.get_address(1).unwrap();
        // This is Hardhat account #0
        assert_eq!(
            format!("{address:?}"),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        );
    }
}
```

### Step 2: Run tests

```bash
cd engine && cargo test execution::wallet 2>&1
```

Expected: all pass.

### Step 3: Commit

```bash
git add engine/src/execution/wallet.rs
git commit -m "feat: add WalletKeyStore with AES-256-GCM encryption and zeroize"
```

---

## Task 6: Fee rate cache

**Files:**
- Modify: `engine/src/execution/fees.rs`

### Step 1: Implement fee cache

Replace `engine/src/execution/fees.rs`:

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

const CACHE_TTL: Duration = Duration::from_secs(60);

struct CachedFee {
    fee_rate_bps: u16,
    fetched_at: Instant,
}

/// Caches feeRateBps per token_id, refreshing from the CLOB API every 60s.
pub struct FeeCache {
    cache: RwLock<HashMap<String, CachedFee>>,
    http: reqwest::Client,
    clob_url: String,
}

impl FeeCache {
    pub fn new(http: reqwest::Client, clob_url: &str) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            http,
            clob_url: clob_url.to_string(),
        }
    }

    /// Get fee rate for a token, using cache if fresh.
    pub async fn get_fee(&self, token_id: &str) -> Result<u16> {
        // Check cache
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(token_id) {
                if entry.fetched_at.elapsed() < CACHE_TTL {
                    return Ok(entry.fee_rate_bps);
                }
            }
        }

        // Fetch from API
        let fee = self.fetch_fee(token_id).await?;

        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(
                token_id.to_string(),
                CachedFee {
                    fee_rate_bps: fee,
                    fetched_at: Instant::now(),
                },
            );
        }

        Ok(fee)
    }

    async fn fetch_fee(&self, token_id: &str) -> Result<u16> {
        let url = format!("{}/fee-rate?token_id={}", self.clob_url, token_id);
        let resp: serde_json::Value = self
            .http
            .get(&url)
            .send()
            .await
            .context("fee rate request failed")?
            .json()
            .await
            .context("fee rate response parse failed")?;

        let fee = resp["fee_rate_bps"]
            .as_u64()
            .context("fee_rate_bps missing from response")? as u16;

        tracing::debug!(token_id, fee, "fee_rate_fetched");
        Ok(fee)
    }

    /// Manually set a fee (for testing or pre-seeding).
    #[cfg(test)]
    pub fn set_fee(&self, token_id: &str, fee: u16) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(
            token_id.to_string(),
            CachedFee {
                fee_rate_bps: fee,
                fetched_at: Instant::now(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        let cache = FeeCache::new(reqwest::Client::new(), "http://localhost");
        cache.set_fee("token_123", 150);

        // Synchronous cache check
        let c = cache.cache.read().unwrap();
        let entry = c.get("token_123").unwrap();
        assert_eq!(entry.fee_rate_bps, 150);
        assert!(entry.fetched_at.elapsed() < CACHE_TTL);
    }

    #[test]
    fn test_cache_miss_returns_none() {
        let cache = FeeCache::new(reqwest::Client::new(), "http://localhost");
        let c = cache.cache.read().unwrap();
        assert!(c.get("nonexistent").is_none());
    }

    #[test]
    fn test_cache_expiry() {
        let cache = FeeCache::new(reqwest::Client::new(), "http://localhost");
        {
            let mut c = cache.cache.write().unwrap();
            c.insert(
                "old_token".to_string(),
                CachedFee {
                    fee_rate_bps: 100,
                    fetched_at: Instant::now() - Duration::from_secs(120), // expired
                },
            );
        }
        let c = cache.cache.read().unwrap();
        let entry = c.get("old_token").unwrap();
        assert!(entry.fetched_at.elapsed() >= CACHE_TTL);
    }
}
```

### Step 2: Run tests

```bash
cd engine && cargo test execution::fees 2>&1
```

Expected: all pass.

### Step 3: Commit

```bash
git add engine/src/execution/fees.rs
git commit -m "feat: add FeeCache for per-token feeRateBps with 60s TTL"
```

---

## Task 7: CLOB order builder + EIP-712 signing + submission

This is the largest task — it builds the Polymarket CLOB order, signs it with EIP-712, and submits with Builder headers.

**Files:**
- Modify: `engine/src/execution/orders.rs`

### Step 1: Implement order submitter

Replace `engine/src/execution/orders.rs`:

```rust
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use alloy::primitives::{Address, FixedBytes, U256};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use alloy::sol_types::{eip712_domain, SolStruct};
use alloy::sol_types::sol;
use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::fees::FeeCache;
use super::wallet::WalletKeyStore;
use super::{ExecutionOrder, OrderResult, OrderStatus, Side};

type HmacSha256 = Hmac<Sha256>;

// Polymarket CTF Exchange EIP-712 order struct
sol! {
    #[derive(Debug)]
    struct ClobOrder {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

// Polymarket NegRiskCtfExchange on Polygon
const NEG_RISK_EXCHANGE: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";
// Regular CTF Exchange
const CTF_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";
const CHAIN_ID: u64 = 137; // Polygon mainnet

/// Submits signed orders to the Polymarket CLOB API.
pub struct OrderSubmitter {
    http: reqwest::Client,
    clob_url: String,
    builder_api_key: String,
    builder_secret: String,
    builder_passphrase: String,
    pub wallet_keys: Arc<WalletKeyStore>,
    pub fee_cache: Arc<FeeCache>,
    neg_risk: bool, // true for updown markets
}

impl OrderSubmitter {
    pub fn new(
        http: reqwest::Client,
        clob_url: &str,
        builder_api_key: &str,
        builder_secret: &str,
        builder_passphrase: &str,
        wallet_keys: Arc<WalletKeyStore>,
        fee_cache: Arc<FeeCache>,
    ) -> Self {
        Self {
            http,
            clob_url: clob_url.to_string(),
            builder_api_key: builder_api_key.to_string(),
            builder_secret: builder_secret.to_string(),
            builder_passphrase: builder_passphrase.to_string(),
            wallet_keys,
            fee_cache,
            neg_risk: true, // default to neg-risk for updown markets
        }
    }

    /// Submit an order to the CLOB API.
    pub async fn submit(&self, order: &ExecutionOrder) -> Result<OrderResult> {
        // 1. Get signer for this wallet
        let signer = self.wallet_keys.get_signer(order.wallet_id)?;
        let maker_address = signer.address();

        // 2. Fetch fee rate
        let fee_bps = self.fee_cache.get_fee(&order.token_id).await?;

        // 3. Build and sign EIP-712 order
        let (signed_order, signature) =
            self.build_and_sign(order, &signer, maker_address, fee_bps)?;

        // 4. Build request payload
        let payload = self.build_payload(&signed_order, &signature, maker_address, fee_bps);

        // 5. Submit with Builder headers
        let timestamp = now_millis().to_string();
        let body = serde_json::to_string(&payload)?;
        let builder_sig = self.sign_builder_request("POST", "/order", &timestamp, &body)?;

        let resp = self
            .http
            .post(format!("{}/order", self.clob_url))
            .header("Content-Type", "application/json")
            .header("POLY-ADDRESS", format!("{maker_address:?}"))
            .header("POLY-BUILDER-API-KEY", &self.builder_api_key)
            .header("POLY-BUILDER-SIGNATURE", &builder_sig)
            .header("POLY-BUILDER-TIMESTAMP", &timestamp)
            .header("POLY-BUILDER-PASSPHRASE", &self.builder_passphrase)
            .body(body)
            .send()
            .await
            .context("CLOB order submission failed")?;

        let status_code = resp.status();
        let resp_body: serde_json::Value = resp.json().await
            .context("CLOB response parse failed")?;

        tracing::info!(
            wallet_id = order.wallet_id,
            status = %status_code,
            response = %resp_body,
            "clob_order_response"
        );

        if !status_code.is_success() {
            let msg = resp_body["errorMsg"].as_str().unwrap_or("unknown error");
            return Ok(OrderResult {
                polymarket_order_id: String::new(),
                status: OrderStatus::Failed,
                filled_price: None,
                fee_bps: Some(fee_bps),
            });
        }

        let order_id = resp_body["orderID"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // 6. Poll for fill status
        let status = self.poll_order_status(&order_id).await;

        Ok(OrderResult {
            polymarket_order_id: order_id,
            status,
            filled_price: order.price, // TODO: get actual fill price from trade
            fee_bps: Some(fee_bps),
        })
    }

    fn build_and_sign(
        &self,
        order: &ExecutionOrder,
        signer: &PrivateKeySigner,
        maker: Address,
        fee_bps: u16,
    ) -> Result<(ClobOrder, alloy::primitives::PrimitiveSignature)> {
        let salt: u256_salt = U256::from(rand::random::<u128>());
        let token_id = U256::from_str_radix(&order.token_id, 10)
            .or_else(|_| U256::from_str_radix(order.token_id.trim_start_matches("0x"), 16))
            .context("invalid token_id")?;

        // amounts in wei (6 decimals for USDC)
        let size_wei = (order.size_usdc * 1_000_000.0) as u64;
        let price_scaled = order.price.unwrap_or(1.0); // market orders use 1.0

        let (maker_amount, taker_amount) = match order.side {
            Side::Buy => {
                // Buying outcome tokens: pay USDC, receive tokens
                let taker_amt = size_wei;
                let maker_amt = (size_wei as f64 * price_scaled) as u64;
                (U256::from(maker_amt), U256::from(taker_amt))
            }
            Side::Sell => {
                // Selling outcome tokens: pay tokens, receive USDC
                let maker_amt = size_wei;
                let taker_amt = (size_wei as f64 * price_scaled) as u64;
                (U256::from(maker_amt), U256::from(taker_amt))
            }
        };

        let side_byte: u8 = match order.side {
            Side::Buy => 0,
            Side::Sell => 1,
        };

        let clob_order = ClobOrder {
            salt: salt,
            maker,
            signer: maker,
            taker: Address::ZERO,
            tokenId: token_id,
            makerAmount: maker_amount,
            takerAmount: taker_amount,
            expiration: U256::ZERO, // GTC
            nonce: U256::ZERO,
            feeRateBps: U256::from(fee_bps),
            side: side_byte,
            signatureType: 0, // EOA
        };

        let exchange_addr = if self.neg_risk {
            NEG_RISK_EXCHANGE.parse::<Address>()?
        } else {
            CTF_EXCHANGE.parse::<Address>()?
        };

        let domain = eip712_domain! {
            name: "ClobAuthDomain",
            version: "1",
            chain_id: CHAIN_ID,
            verifying_contract: exchange_addr,
        };

        let signing_hash = clob_order.eip712_signing_hash(&domain);
        let signature = signer
            .sign_hash_sync(&signing_hash)
            .context("EIP-712 signing failed")?;

        Ok((clob_order, signature))
    }

    fn build_payload(
        &self,
        order: &ClobOrder,
        signature: &alloy::primitives::PrimitiveSignature,
        maker: Address,
        fee_bps: u16,
    ) -> serde_json::Value {
        let sig_bytes = {
            let mut bytes = [0u8; 65];
            bytes[..32].copy_from_slice(&signature.r().to_be_bytes::<32>());
            bytes[32..64].copy_from_slice(&signature.s().to_be_bytes::<32>());
            bytes[64] = signature.v() as u8;
            hex::encode(bytes)
        };

        serde_json::json!({
            "salt": order.salt.to_string(),
            "maker": format!("{maker:?}"),
            "signer": format!("{maker:?}"),
            "taker": "0x0000000000000000000000000000000000000000",
            "tokenId": order.tokenId.to_string(),
            "makerAmount": order.makerAmount.to_string(),
            "takerAmount": order.takerAmount.to_string(),
            "expiration": "0",
            "nonce": "0",
            "feeRateBps": fee_bps.to_string(),
            "side": order.side.to_string(),
            "signatureType": "0",
            "signature": format!("0x{sig_bytes}")
        })
    }

    fn sign_builder_request(
        &self,
        method: &str,
        path: &str,
        timestamp: &str,
        body: &str,
    ) -> Result<String> {
        let message = format!("{method}{path}{timestamp}{body}");
        let mut mac = HmacSha256::new_from_slice(self.builder_secret.as_bytes())
            .context("invalid builder secret")?;
        mac.update(message.as_bytes());
        let result = mac.finalize();
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(result.into_bytes()))
    }

    async fn poll_order_status(&self, order_id: &str) -> OrderStatus {
        if order_id.is_empty() {
            return OrderStatus::Failed;
        }

        let url = format!("{}/data/order/{}", self.clob_url, order_id);
        for attempt in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let resp = match self.http.get(&url).send().await {
                Ok(r) => r,
                Err(_) => continue,
            };
            let body: serde_json::Value = match resp.json().await {
                Ok(b) => b,
                Err(_) => continue,
            };

            match body["status"].as_str() {
                Some("FILLED") => return OrderStatus::Filled,
                Some("CANCELLED") | Some("REJECTED") => return OrderStatus::Cancelled,
                Some("PENDING") | Some("ACCEPTED") => continue,
                _ => continue,
            }
        }

        OrderStatus::Timeout
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_signature() {
        let keys = Arc::new(WalletKeyStore::new(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ).unwrap());
        let fees = Arc::new(FeeCache::new(reqwest::Client::new(), "http://localhost"));

        let submitter = OrderSubmitter::new(
            reqwest::Client::new(),
            "https://clob.polymarket.com",
            "test_key",
            "test_secret",
            "test_pass",
            keys,
            fees,
        );

        let sig = submitter
            .sign_builder_request("POST", "/order", "1234567890", r#"{"test":"body"}"#)
            .unwrap();

        // Should be a valid base64 string
        use base64::Engine;
        assert!(base64::engine::general_purpose::STANDARD.decode(&sig).is_ok());
    }

    #[test]
    fn test_now_millis() {
        let ts = now_millis();
        // Should be a reasonable timestamp (after 2024)
        assert!(ts > 1_700_000_000_000);
    }
}
```

### Step 2: Run tests

```bash
cd engine && cargo test execution::orders 2>&1
```

Expected: all pass.

### Step 3: Commit

```bash
git add engine/src/execution/orders.rs
git commit -m "feat: add CLOB order builder with EIP-712 signing and Builder headers"
```

---

## Task 8: PostgreSQL storage with sqlx

**Files:**
- Create: `engine/src/storage/postgres.rs`
- Modify: `engine/src/storage/mod.rs`

### Step 1: Create PostgreSQL storage module

Create `engine/src/storage/postgres.rs`:

```rust
use anyhow::{Context, Result};
use sqlx::PgPool;

use crate::execution::{ExecutionOrder, OrderResult, OrderStatus, Side};
use crate::strategy::Outcome;

/// Create a PostgreSQL connection pool.
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url)
        .await
        .context("failed to connect to PostgreSQL")?;
    tracing::info!("postgres_connected");
    Ok(pool)
}

/// Write a filled trade to the trades table.
pub async fn write_trade(
    pool: &PgPool,
    order: &ExecutionOrder,
    result: &OrderResult,
) -> Result<i64> {
    let side_str = match order.side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    };
    let outcome_str = match order.outcome {
        Outcome::Up => "UP",
        Outcome::Down => "DOWN",
    };
    let order_type_str = match &order.order_type {
        crate::strategy::OrderType::Market => "market",
        crate::strategy::OrderType::Limit { .. } => "limit",
        crate::strategy::OrderType::StopLoss { .. } => "stoploss",
        crate::strategy::OrderType::TakeProfit { .. } => "take_profit",
    };
    let status_str = match result.status {
        OrderStatus::Filled => "filled",
        OrderStatus::Cancelled => "cancelled",
        OrderStatus::Failed | OrderStatus::Timeout => "cancelled",
    };

    let row = sqlx::query_scalar::<_, i64>(
        r#"INSERT INTO trades (
            wallet_id, strategy_id, copy_relationship_id,
            market_id, side, outcome, price, size_usdc,
            order_type, status, polymarket_order_id, fee_bps, executed_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
        RETURNING id"#,
    )
    .bind(order.wallet_id as i64)
    .bind(order.strategy_id.map(|id| id as i64))
    .bind(order.copy_relationship_id.map(|id| id as i64))
    .bind(&order.symbol)
    .bind(side_str)
    .bind(outcome_str)
    .bind(result.filled_price)
    .bind(order.size_usdc)
    .bind(order_type_str)
    .bind(status_str)
    .bind(&result.polymarket_order_id)
    .bind(result.fee_bps.map(|f| f as i16))
    .fetch_one(pool)
    .await
    .context("failed to write trade")?;

    tracing::info!(trade_id = row, wallet_id = order.wallet_id, "trade_written");
    Ok(row)
}

/// Write a copy trade audit record.
pub async fn write_copy_trade(
    pool: &PgPool,
    copy_relationship_id: i64,
    follower_trade_id: Option<i64>,
    leader_address: &str,
    leader_market_id: &str,
    leader_outcome: &str,
    leader_price: f64,
    leader_size_usdc: f64,
    leader_tx_hash: &str,
    follower_price: Option<f64>,
    status: &str,
    skip_reason: Option<&str>,
) -> Result<i64> {
    let slippage = follower_price.map(|fp| (fp - leader_price) / leader_price);

    let row = sqlx::query_scalar::<_, i64>(
        r#"INSERT INTO copy_trades (
            copy_relationship_id, follower_trade_id,
            leader_address, leader_market_id, leader_outcome,
            leader_price, leader_size_usdc, leader_tx_hash,
            follower_price, slippage_pct, status, skip_reason,
            detected_at, executed_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(),
                  CASE WHEN $11 = 'filled' THEN NOW() ELSE NULL END)
        RETURNING id"#,
    )
    .bind(copy_relationship_id)
    .bind(follower_trade_id)
    .bind(leader_address)
    .bind(leader_market_id)
    .bind(leader_outcome)
    .bind(leader_price)
    .bind(leader_size_usdc)
    .bind(leader_tx_hash)
    .bind(follower_price)
    .bind(slippage)
    .bind(status)
    .bind(skip_reason)
    .fetch_one(pool)
    .await
    .context("failed to write copy trade")?;

    tracing::info!(copy_trade_id = row, "copy_trade_written");
    Ok(row)
}

/// Load active copy relationships for a watched address.
#[derive(Debug, Clone)]
pub struct CopyRelationship {
    pub id: i64,
    pub follower_wallet_id: i64,
    pub size_mode: String,
    pub size_value: f64,
    pub max_position_usdc: f64,
    pub markets_filter: Option<serde_json::Value>,
}

pub async fn get_active_followers(
    pool: &PgPool,
    watched_address: &str,
) -> Result<Vec<CopyRelationship>> {
    let rows = sqlx::query_as::<_, (i64, i64, String, f64, f64, Option<serde_json::Value>)>(
        r#"SELECT cr.id, cr.follower_wallet_id, cr.size_mode,
                  cr.size_value::float8, cr.max_position_usdc::float8, cr.markets_filter
           FROM copy_relationships cr
           JOIN watched_wallets ww ON ww.id = cr.watched_wallet_id
           WHERE ww.address = $1 AND cr.is_active = true"#,
    )
    .bind(watched_address)
    .fetch_all(pool)
    .await
    .context("failed to get active followers")?;

    Ok(rows
        .into_iter()
        .map(|(id, wallet_id, mode, value, max_pos, filter)| CopyRelationship {
            id,
            follower_wallet_id: wallet_id,
            size_mode: mode,
            size_value: value,
            max_position_usdc: max_pos,
            markets_filter: filter,
        })
        .collect())
}

/// Load all watched wallet addresses.
pub async fn load_watched_addresses(pool: &PgPool) -> Result<Vec<String>> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT ww.address FROM watched_wallets ww \
         JOIN copy_relationships cr ON cr.watched_wallet_id = ww.id \
         WHERE cr.is_active = true",
    )
    .fetch_all(pool)
    .await
    .context("failed to load watched addresses")?;

    Ok(rows)
}
```

### Step 2: Register in storage mod

In `engine/src/storage/mod.rs`, add:

```rust
pub mod clickhouse;
pub mod postgres;
pub mod redis;
```

### Step 3: Verify compilation

```bash
cd engine && cargo check 2>&1
```

Expected: compiles.

### Step 4: Commit

```bash
git add engine/src/storage/postgres.rs engine/src/storage/mod.rs
git commit -m "feat: add PostgreSQL storage — trades, copy_trades, followers queries"
```

---

## Task 9: Executor loop + position lifecycle

The central loop that pops from the queue, executes orders, and updates state.

**Files:**
- Create: `engine/src/execution/executor.rs`
- Modify: `engine/src/execution/mod.rs` — register module

### Step 1: Implement executor

Create `engine/src/execution/executor.rs`:

```rust
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use sqlx::PgPool;
use tokio::sync::Mutex;

use super::orders::OrderSubmitter;
use super::queue::ExecutionQueue;
use super::{OrderResult, OrderStatus};
use crate::strategy::registry::AssignmentRegistry;
use crate::strategy::state::Position;
use crate::strategy::Outcome;
use crate::storage;

/// Run the executor loop: pop orders from queue → rate limit → submit → update state.
pub async fn run(
    queue: Arc<Mutex<ExecutionQueue>>,
    submitter: Arc<OrderSubmitter>,
    registry: AssignmentRegistry,
    db: PgPool,
) -> Result<()> {
    tracing::info!("executor_started");

    loop {
        // Pop next order
        let order = {
            let mut q = queue.lock().await;
            q.pop()
        };

        let Some(order) = order else {
            // Queue empty — wait a bit
            tokio::time::sleep(Duration::from_millis(50)).await;
            continue;
        };

        // Rate limit check
        let allowed = {
            let mut q = queue.lock().await;
            q.try_rate_limit(order.wallet_id)
        };

        if !allowed {
            tracing::warn!(
                wallet_id = order.wallet_id,
                priority = ?order.priority,
                "rate_limited_requeueing"
            );
            // Re-queue and wait
            {
                let mut q = queue.lock().await;
                q.push(order);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        }

        // Submit order
        tracing::info!(
            wallet_id = order.wallet_id,
            side = ?order.side,
            outcome = ?order.outcome,
            size = order.size_usdc,
            priority = ?order.priority,
            "executing_order"
        );

        let result = match submitter.submit(&order).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, wallet_id = order.wallet_id, "order_submission_failed");
                OrderResult {
                    polymarket_order_id: String::new(),
                    status: OrderStatus::Failed,
                    filled_price: None,
                    fee_bps: None,
                }
            }
        };

        // Update position state in registry
        if result.status == OrderStatus::Filled {
            update_position(&registry, &order, &result).await;
        }

        // Write trade to PostgreSQL
        let trade_id = match storage::postgres::write_trade(&db, &order, &result).await {
            Ok(id) => Some(id),
            Err(e) => {
                tracing::error!(error = %e, "trade_write_failed");
                None
            }
        };

        // If copy trade, write copy_trade audit
        if let Some(copy_rel_id) = order.copy_relationship_id {
            let status_str = match result.status {
                OrderStatus::Filled => "filled",
                OrderStatus::Cancelled => "failed",
                OrderStatus::Failed => "failed",
                OrderStatus::Timeout => "failed",
            };
            if let Err(e) = storage::postgres::write_copy_trade(
                &db,
                copy_rel_id as i64,
                trade_id,
                "", // leader_address — set by watcher
                &order.symbol,
                match order.outcome {
                    Outcome::Up => "UP",
                    Outcome::Down => "DOWN",
                },
                order.price.unwrap_or(0.0),
                order.size_usdc,
                "", // leader_tx_hash — set by watcher
                result.filled_price,
                status_str,
                None,
            )
            .await
            {
                tracing::error!(error = %e, "copy_trade_write_failed");
            }
        }

        tracing::info!(
            wallet_id = order.wallet_id,
            order_id = %result.polymarket_order_id,
            status = ?result.status,
            "order_completed"
        );
    }
}

/// Update strategy state position after a fill.
async fn update_position(
    registry: &AssignmentRegistry,
    order: &super::ExecutionOrder,
    result: &OrderResult,
) {
    let Some(strategy_id) = order.strategy_id else {
        return;
    };

    let reg = registry.read().await;
    for assignments in reg.values() {
        for a in assignments {
            if a.wallet_id == order.wallet_id && a.strategy_id == strategy_id {
                let mut state = match a.state.lock() {
                    Ok(s) => s,
                    Err(p) => p.into_inner(),
                };

                match order.side {
                    super::Side::Buy => {
                        state.position = Some(Position {
                            outcome: order.outcome,
                            entry_price: result.filled_price.unwrap_or(0.0),
                            size_usdc: order.size_usdc,
                            entry_at: chrono::Utc::now().timestamp(),
                        });
                        tracing::info!(
                            wallet_id = order.wallet_id,
                            strategy_id,
                            outcome = ?order.outcome,
                            price = ?result.filled_price,
                            "position_opened"
                        );
                    }
                    super::Side::Sell => {
                        if let Some(ref pos) = state.position {
                            let exit_price = result.filled_price.unwrap_or(0.0);
                            let pnl = (exit_price - pos.entry_price) * pos.size_usdc;
                            state.pnl += pnl;
                            tracing::info!(
                                wallet_id = order.wallet_id,
                                strategy_id,
                                pnl,
                                total_pnl = state.pnl,
                                "position_closed"
                            );
                        }
                        state.position = None;
                    }
                }
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{ExecutionOrder, OrderPriority, Side};
    use crate::strategy::{OrderType, Outcome};
    use crate::strategy::registry::{self, AssignmentRegistry};

    fn test_order(side: Side) -> ExecutionOrder {
        ExecutionOrder {
            id: uuid::Uuid::new_v4(),
            wallet_id: 1,
            strategy_id: Some(100),
            copy_relationship_id: None,
            symbol: "btc".into(),
            token_id: "tok".into(),
            side,
            outcome: Outcome::Up,
            price: Some(0.60),
            size_usdc: 50.0,
            order_type: OrderType::Market,
            priority: OrderPriority::StrategyMarket,
            created_at: 0,
        }
    }

    #[tokio::test]
    async fn test_update_position_buy_sets_position() {
        let reg = AssignmentRegistry::new();
        registry::activate(
            &reg, 1, 100,
            serde_json::json!({"mode": "form"}),
            vec!["btc".into()], 200.0, None,
        ).await;

        let order = test_order(Side::Buy);
        let result = OrderResult {
            polymarket_order_id: "test".into(),
            status: OrderStatus::Filled,
            filled_price: Some(0.60),
            fee_bps: Some(100),
        };

        update_position(&reg, &order, &result).await;

        let r = reg.read().await;
        let a = &r["btc"][0];
        let state = a.state.lock().unwrap();
        assert!(state.position.is_some());
        let pos = state.position.as_ref().unwrap();
        assert_eq!(pos.outcome, Outcome::Up);
        assert!((pos.entry_price - 0.60).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_update_position_sell_clears_and_updates_pnl() {
        let reg = AssignmentRegistry::new();
        registry::activate(
            &reg, 1, 100,
            serde_json::json!({"mode": "form"}),
            vec!["btc".into()], 200.0, None,
        ).await;

        // Set initial position
        {
            let r = reg.read().await;
            let mut state = r["btc"][0].state.lock().unwrap();
            state.position = Some(Position {
                outcome: Outcome::Up,
                entry_price: 0.50,
                size_usdc: 50.0,
                entry_at: 0,
            });
        }

        let order = test_order(Side::Sell);
        let result = OrderResult {
            polymarket_order_id: "test".into(),
            status: OrderStatus::Filled,
            filled_price: Some(0.70),
            fee_bps: Some(100),
        };

        update_position(&reg, &order, &result).await;

        let r = reg.read().await;
        let state = r["btc"][0].state.lock().unwrap();
        assert!(state.position.is_none());
        // PnL = (0.70 - 0.50) * 50.0 = 10.0
        assert!((state.pnl - 10.0).abs() < f64::EPSILON);
    }
}
```

### Step 2: Register in execution mod

In `engine/src/execution/mod.rs`, add after the existing `pub mod` lines:

```rust
pub mod executor;
```

### Step 3: Run tests

```bash
cd engine && cargo test execution::executor 2>&1
```

Expected: all pass.

### Step 4: Commit

```bash
git add engine/src/execution/executor.rs engine/src/execution/mod.rs
git commit -m "feat: add executor loop with position lifecycle management"
```

---

## Task 10: Copy trading watcher

**Files:**
- Create: `engine/src/watcher/mod.rs`
- Create: `engine/src/watcher/polymarket.rs`
- Modify: `engine/src/main.rs` — register module

### Step 1: Create watcher module

Create `engine/src/watcher/mod.rs`:

```rust
pub mod polymarket;
```

Create `engine/src/watcher/polymarket.rs`:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::execution::queue::ExecutionQueue;
use crate::execution::{ExecutionOrder, OrderPriority, Side};
use crate::storage;
use crate::strategy::{OrderType, Outcome};

/// A trade detected on a watched leader wallet.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LeaderTrade {
    pub side: String,            // "BUY" or "SELL"
    pub asset: String,           // token_id
    #[serde(alias = "conditionId")]
    pub condition_id: String,
    pub size: f64,
    pub price: f64,
    pub timestamp: i64,
    #[serde(alias = "transactionHash")]
    pub transaction_hash: String,
    pub outcome: Option<String>, // "Yes" / "No"
}

/// Run the copy trading watcher loop.
pub async fn run(
    data_api_url: &str,
    http: reqwest::Client,
    queue: Arc<Mutex<ExecutionQueue>>,
    db: PgPool,
    mut redis_conn: redis::aio::MultiplexedConnection,
) -> Result<()> {
    tracing::info!("copy_watcher_started");
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        // Load watched addresses (from DB, could cache in Redis)
        let addresses = match storage::postgres::load_watched_addresses(&db).await {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!(error = %e, "load_watched_addresses_failed");
                continue;
            }
        };

        if addresses.is_empty() {
            continue;
        }

        // Check each address for new trades
        let results = futures_util::future::join_all(
            addresses.iter().map(|addr| {
                check_new_trades(
                    &http,
                    data_api_url,
                    addr,
                    &mut redis_conn,
                )
            }),
        )
        .await;

        for (addr, new_trades) in addresses.iter().zip(results) {
            let trades = match new_trades {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(address = %addr, error = %e, "check_trades_failed");
                    continue;
                }
            };

            for trade in trades {
                // Get active followers for this address
                let followers = match storage::postgres::get_active_followers(&db, addr).await {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::warn!(error = %e, "get_followers_failed");
                        continue;
                    }
                };

                for follower in &followers {
                    match build_copy_order(&trade, follower, addr) {
                        Some(order) => {
                            tracing::info!(
                                leader = %addr,
                                follower_wallet = follower.follower_wallet_id,
                                size = order.size_usdc,
                                "copy_order_queued"
                            );
                            let mut q = queue.lock().await;
                            q.push(order);
                        }
                        None => {
                            // Write skipped copy trade
                            let _ = storage::postgres::write_copy_trade(
                                &db,
                                follower.id,
                                None,
                                addr,
                                &trade.condition_id,
                                trade.outcome.as_deref().unwrap_or(""),
                                trade.price,
                                trade.size,
                                &trade.transaction_hash,
                                None,
                                "skipped",
                                Some("markets_filter_or_limit"),
                            )
                            .await;
                        }
                    }
                }

                // Update last_seen for this address
                update_last_seen(&mut redis_conn, addr, &trade).await;
            }
        }
    }
}

/// Poll the Polymarket data API for recent trades by a wallet address.
async fn check_new_trades(
    http: &reqwest::Client,
    data_api_url: &str,
    address: &str,
    redis_conn: &mut redis::aio::MultiplexedConnection,
) -> Result<Vec<LeaderTrade>> {
    let url = format!("{}/trades?user={}&limit=5&sortBy=TIMESTAMP&sortDirection=DESC",
        data_api_url, address);

    let resp: Vec<LeaderTrade> = http
        .get(&url)
        .send()
        .await
        .context("data API request failed")?
        .json()
        .await
        .context("data API response parse failed")?;

    // Filter to only new trades (after last_seen_at)
    let last_seen = get_last_seen(redis_conn, address).await;
    let new_trades: Vec<LeaderTrade> = resp
        .into_iter()
        .filter(|t| t.timestamp > last_seen)
        .collect();

    if !new_trades.is_empty() {
        tracing::info!(
            address,
            count = new_trades.len(),
            "new_leader_trades_detected"
        );
    }

    Ok(new_trades)
}

async fn get_last_seen(
    conn: &mut redis::aio::MultiplexedConnection,
    address: &str,
) -> i64 {
    let key = format!("oddex:watcher:last_seen:{address}");
    redis::cmd("GET")
        .arg(&key)
        .query_async::<Option<i64>>(conn)
        .await
        .unwrap_or(None)
        .unwrap_or(0)
}

async fn update_last_seen(
    conn: &mut redis::aio::MultiplexedConnection,
    address: &str,
    trade: &LeaderTrade,
) {
    let key = format!("oddex:watcher:last_seen:{address}");
    let _: Result<(), _> = redis::cmd("SET")
        .arg(&key)
        .arg(trade.timestamp)
        .query_async(conn)
        .await;
}

/// Build a copy execution order from a leader trade and follower config.
fn build_copy_order(
    trade: &LeaderTrade,
    follower: &storage::postgres::CopyRelationship,
    leader_address: &str,
) -> Option<ExecutionOrder> {
    // Check markets filter
    if let Some(ref filter) = follower.markets_filter {
        if let Some(arr) = filter.as_array() {
            let market_ids: Vec<&str> = arr.iter().filter_map(|v| v.as_str()).collect();
            if !market_ids.contains(&trade.condition_id.as_str()) {
                return None;
            }
        }
    }

    // Calculate copy size
    let size_usdc = match follower.size_mode.as_str() {
        "fixed" => follower.size_value,
        "proportional" => trade.size * follower.size_value,
        _ => return None,
    };

    // Check max position
    if size_usdc > follower.max_position_usdc {
        return None;
    }

    let outcome = match trade.outcome.as_deref() {
        Some("Yes") | Some("UP") => Outcome::Up,
        Some("No") | Some("DOWN") => Outcome::Down,
        _ => Outcome::Up, // default
    };

    let side = match trade.side.as_str() {
        "BUY" => Side::Buy,
        "SELL" => Side::Sell,
        _ => return None,
    };

    Some(ExecutionOrder {
        id: uuid::Uuid::new_v4(),
        wallet_id: follower.follower_wallet_id as u64,
        strategy_id: None,
        copy_relationship_id: Some(follower.id as u64),
        symbol: trade.condition_id.clone(),
        token_id: trade.asset.clone(),
        side,
        outcome,
        price: None, // market order for copies
        size_usdc,
        order_type: OrderType::Market,
        priority: OrderPriority::CopyMarket,
        created_at: chrono::Utc::now().timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::postgres::CopyRelationship;

    fn test_trade() -> LeaderTrade {
        LeaderTrade {
            side: "BUY".into(),
            asset: "token_123".into(),
            condition_id: "condition_456".into(),
            size: 100.0,
            price: 0.65,
            timestamp: 1700000000,
            transaction_hash: "0xabc".into(),
            outcome: Some("Yes".into()),
        }
    }

    fn test_follower() -> CopyRelationship {
        CopyRelationship {
            id: 1,
            follower_wallet_id: 42,
            size_mode: "fixed".into(),
            size_value: 50.0,
            max_position_usdc: 200.0,
            markets_filter: None,
        }
    }

    #[test]
    fn test_build_copy_order_fixed_size() {
        let trade = test_trade();
        let follower = test_follower();
        let order = build_copy_order(&trade, &follower, "0xleader").unwrap();

        assert_eq!(order.wallet_id, 42);
        assert_eq!(order.size_usdc, 50.0);
        assert_eq!(order.priority, OrderPriority::CopyMarket);
        assert!(order.strategy_id.is_none());
        assert_eq!(order.copy_relationship_id, Some(1));
    }

    #[test]
    fn test_build_copy_order_proportional_size() {
        let trade = test_trade(); // size = 100.0
        let mut follower = test_follower();
        follower.size_mode = "proportional".into();
        follower.size_value = 0.5; // 50% of leader

        let order = build_copy_order(&trade, &follower, "0xleader").unwrap();
        assert!((order.size_usdc - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_build_copy_order_exceeds_max_position() {
        let trade = test_trade();
        let mut follower = test_follower();
        follower.max_position_usdc = 10.0; // too small for 50 USDC fixed

        assert!(build_copy_order(&trade, &follower, "0xleader").is_none());
    }

    #[test]
    fn test_build_copy_order_markets_filter_pass() {
        let trade = test_trade(); // condition_id = "condition_456"
        let mut follower = test_follower();
        follower.markets_filter = Some(serde_json::json!(["condition_456", "condition_789"]));

        assert!(build_copy_order(&trade, &follower, "0xleader").is_some());
    }

    #[test]
    fn test_build_copy_order_markets_filter_reject() {
        let trade = test_trade();
        let mut follower = test_follower();
        follower.markets_filter = Some(serde_json::json!(["other_market"]));

        assert!(build_copy_order(&trade, &follower, "0xleader").is_none());
    }

    #[test]
    fn test_build_copy_order_null_filter_passes_all() {
        let trade = test_trade();
        let follower = test_follower(); // markets_filter = None

        assert!(build_copy_order(&trade, &follower, "0xleader").is_some());
    }

    #[test]
    fn test_build_copy_order_sell_side() {
        let mut trade = test_trade();
        trade.side = "SELL".into();
        let follower = test_follower();

        let order = build_copy_order(&trade, &follower, "0xleader").unwrap();
        assert_eq!(order.side, Side::Sell);
    }
}
```

### Step 2: Register watcher module in main.rs

In `engine/src/main.rs`, add after `mod execution;`:

```rust
mod watcher;
```

### Step 3: Run tests

```bash
cd engine && cargo test watcher 2>&1
```

Expected: all pass.

### Step 4: Commit

```bash
git add engine/src/watcher/ engine/src/main.rs
git commit -m "feat: add copy trading watcher with trade detection and fan-out"
```

---

## Task 11: Wire execution + watcher into main.rs

Replace the signal logger with the full execution pipeline.

**Files:**
- Modify: `engine/src/tasks/mod.rs` — add execution tasks
- Create: `engine/src/tasks/execution_tasks.rs`
- Modify: `engine/src/tasks/engine_tasks.rs` — remove signal logger

### Step 1: Create execution tasks spawner

Create `engine/src/tasks/execution_tasks.rs`:

```rust
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinSet;

use super::SharedState;
use crate::execution::executor;
use crate::execution::fees::FeeCache;
use crate::execution::orders::OrderSubmitter;
use crate::execution::queue::ExecutionQueue;
use crate::execution::wallet::WalletKeyStore;
use crate::execution::{ExecutionOrder, Side};
use crate::strategy::registry::AssignmentRegistry;
use crate::strategy::{EngineOutput, OrderType, Signal};

/// Spawn all execution-related tasks.
pub fn spawn_execution(
    state: &SharedState,
    registry: AssignmentRegistry,
    signal_rx: mpsc::Receiver<EngineOutput>,
    db: PgPool,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let cfg = &state.config;

    // Shared execution queue
    let queue = Arc::new(Mutex::new(ExecutionQueue::new(cfg.max_orders_per_day)));

    // Wallet key store
    let wallet_keys = Arc::new(
        WalletKeyStore::new(&cfg.encryption_key)
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "wallet_key_store_init_failed_using_dummy");
                // Use a dummy key for development (no real orders)
                WalletKeyStore::new(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap()
            }),
    );

    // Fee cache
    let fee_cache = Arc::new(FeeCache::new(state.http.clone(), &cfg.clob_api_url));

    // Order submitter
    let submitter = Arc::new(OrderSubmitter::new(
        state.http.clone(),
        &cfg.clob_api_url,
        &cfg.builder_api_key,
        &cfg.builder_secret,
        &cfg.builder_passphrase,
        wallet_keys,
        fee_cache,
    ));

    // Signal → queue bridge: converts EngineOutput into ExecutionOrders
    let bridge_queue = queue.clone();
    tasks.spawn(async move {
        signal_to_queue(signal_rx, bridge_queue).await
    });

    // Executor loop
    let exec_queue = queue.clone();
    let exec_registry = registry;
    let exec_db = db.clone();
    tasks.spawn(async move {
        executor::run(exec_queue, submitter, exec_registry, exec_db).await
    });
}

/// Bridge strategy engine signals into the execution queue.
async fn signal_to_queue(
    mut signal_rx: mpsc::Receiver<EngineOutput>,
    queue: Arc<Mutex<ExecutionQueue>>,
) -> anyhow::Result<()> {
    tracing::info!("signal_to_queue_bridge_started");
    while let Some(output) = signal_rx.recv().await {
        let (outcome, size_usdc, order_type) = match &output.signal {
            Signal::Buy { outcome, size_usdc, order_type } => (*outcome, *size_usdc, order_type.clone()),
            Signal::Sell { outcome, size_usdc, order_type } => (*outcome, *size_usdc, order_type.clone()),
            Signal::Hold => continue,
        };

        let order = ExecutionOrder::from_signal(
            output.wallet_id,
            output.strategy_id,
            &output.symbol,
            "", // token_id — resolved by OrderSubmitter from symbol/outcome
            outcome,
            size_usdc,
            &order_type,
        );

        let mut q = queue.lock().await;
        q.push(order);
    }
    Ok(())
}

/// Spawn the copy trading watcher.
pub fn spawn_watcher(
    state: &SharedState,
    queue: Arc<Mutex<ExecutionQueue>>,
    db: PgPool,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let data_api_url = state.config.data_api_url.clone();
    let http = state.http.clone();
    let redis_url = state.config.redis_url.clone();

    tasks.spawn(async move {
        let client = redis::Client::open(redis_url.as_str())?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        crate::watcher::polymarket::run(&data_api_url, http, queue, db, conn).await
    });
}
```

### Step 2: Update tasks/mod.rs to use execution tasks

Replace `engine/src/tasks/mod.rs`:

```rust
mod data_feed;
mod engine_tasks;
mod execution_tasks;
mod persistence;
mod writers;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinSet;

use crate::config::Config;
use crate::fetcher::models::{ActiveMarket, Tick};
use crate::fetcher::tick_builder::PriceCache;
use crate::fetcher::websocket::{OrderBookCache, WsCommand};

pub struct SharedState {
    pub config: Config,
    pub books: OrderBookCache,
    pub markets: Arc<RwLock<HashMap<String, ActiveMarket>>>,
    pub prices: PriceCache,
    pub tick_tx: broadcast::Sender<Tick>,
    pub ws_cmd_tx: mpsc::Sender<WsCommand>,
    pub http: reqwest::Client,
}

pub async fn spawn_all(
    state: &SharedState,
    ws_cmd_rx: mpsc::Receiver<WsCommand>,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) -> anyhow::Result<()> {
    // Data feed tasks
    data_feed::spawn_ws_feed(state, ws_cmd_rx, tasks);
    data_feed::spawn_price_poller(state, tasks);
    data_feed::spawn_market_discovery(state, tasks);
    data_feed::spawn_tick_builder(state, tasks);

    // Writer tasks
    writers::spawn_clickhouse_writer(state, tasks);
    writers::spawn_kafka_publisher(state, tasks)?;

    // Strategy engine
    let engine_registry = crate::strategy::registry::AssignmentRegistry::new();
    let (signal_tx, signal_rx) = mpsc::channel::<crate::strategy::EngineOutput>(256);

    engine_tasks::spawn_strategy_engine(state, engine_registry.clone(), signal_tx, tasks);

    // PostgreSQL connection pool
    let db = crate::storage::postgres::create_pool(&state.config.database_url).await?;

    // Execution pipeline (replaces signal logger)
    let exec_queue = Arc::new(tokio::sync::Mutex::new(
        crate::execution::queue::ExecutionQueue::new(state.config.max_orders_per_day),
    ));
    execution_tasks::spawn_execution(state, engine_registry.clone(), signal_rx, db.clone(), tasks);

    // Copy trading watcher
    execution_tasks::spawn_watcher(state, exec_queue, db, tasks);

    // Redis state persistence
    persistence::spawn_redis_state_persister(state, engine_registry, tasks);

    Ok(())
}
```

### Step 3: Remove signal logger from engine_tasks.rs

Replace `engine/src/tasks/engine_tasks.rs`:

```rust
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use super::SharedState;
use crate::strategy::registry::AssignmentRegistry;
use crate::strategy::EngineOutput;

pub fn spawn_strategy_engine(
    state: &SharedState,
    engine_registry: AssignmentRegistry,
    signal_tx: mpsc::Sender<EngineOutput>,
    tasks: &mut JoinSet<anyhow::Result<()>>,
) {
    let eng_brokers = state.config.kafka_brokers.clone();
    tasks.spawn(async move {
        crate::strategy::engine::run(&eng_brokers, engine_registry, signal_tx).await
    });
}
```

### Step 4: Update main.rs for async spawn_all

In `engine/src/main.rs`, change the `spawn_all` call from:

```rust
    tasks::spawn_all(&state, ws_cmd_rx, &mut tasks)?;
```

to:

```rust
    tasks::spawn_all(&state, ws_cmd_rx, &mut tasks).await?;
```

### Step 5: Verify compilation

```bash
cd engine && cargo check 2>&1
```

Expected: compiles. Fix any import issues.

### Step 6: Run all tests

```bash
cd engine && cargo test 2>&1
```

Expected: all tests pass.

### Step 7: Commit

```bash
git add engine/src/tasks/ engine/src/main.rs
git commit -m "feat: wire execution pipeline and copy watcher into main loop"
```

---

## Task 12: Final compilation + full test suite

**Files:** none (verification only)

### Step 1: Full build

```bash
cd engine && cargo build 2>&1
```

Expected: compiles cleanly.

### Step 2: Run all tests

```bash
cd engine && cargo test 2>&1
```

Expected: all tests pass.

### Step 3: Run clippy

```bash
cd engine && cargo clippy 2>&1
```

Expected: no errors (warnings OK).

### Step 4: Commit any fixes

If any fixes were needed:

```bash
git add -A engine/
git commit -m "fix: resolve compilation and clippy issues for Phase 4"
```

---

## Summary

| Task | Description | Key files | Tests |
|------|-------------|-----------|-------|
| 1 | Add Cargo.toml dependencies | `Cargo.toml` | compilation |
| 2 | Execution types (ExecutionOrder, Priority) | `execution/mod.rs` | 3 tests |
| 3 | Priority queue + token bucket rate limiter | `execution/queue.rs` | 5 tests |
| 4 | Config updates for execution | `config.rs` | compilation |
| 5 | Wallet key store + AES-256-GCM | `execution/wallet.rs` | 6 tests |
| 6 | Fee rate cache | `execution/fees.rs` | 3 tests |
| 7 | CLOB order builder + EIP-712 + submission | `execution/orders.rs` | 2 tests |
| 8 | PostgreSQL storage (sqlx) | `storage/postgres.rs` | compilation |
| 9 | Executor loop + position lifecycle | `execution/executor.rs` | 2 tests |
| 10 | Copy trading watcher + fan-out | `watcher/polymarket.rs` | 7 tests |
| 11 | Wire into main.rs | `tasks/mod.rs`, `tasks/execution_tasks.rs` | compilation |
| 12 | Final build + test suite | — | full suite |

**New directory structure:**
```
engine/src/
├── execution/
│   ├── mod.rs              # ExecutionOrder, OrderPriority, Side, OrderResult
│   ├── queue.rs            # PriorityQueue + TokenBucket rate limiter
│   ├── orders.rs           # CLOB order builder, EIP-712 signing, Builder headers
│   ├── wallet.rs           # WalletKeyStore + AES-256-GCM
│   ├── fees.rs             # FeeCache (60s TTL per token)
│   └── executor.rs         # Main executor loop + position lifecycle
├── watcher/
│   ├── mod.rs
│   └── polymarket.rs       # Leader trade detection + copy fan-out
├── storage/
│   ├── postgres.rs         # NEW — trades, copy_trades, followers
│   ├── clickhouse.rs
│   └── redis.rs
└── tasks/
    ├── execution_tasks.rs  # NEW — spawn execution + watcher
    ├── engine_tasks.rs     # MODIFIED — removed signal logger
    └── mod.rs              # MODIFIED — async spawn_all
```
