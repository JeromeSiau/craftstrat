use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use alloy::primitives::{Address, U256};
use alloy::signers::SignerSync;
use alloy::sol;
use alloy::sol_types::{eip712_domain, SolStruct};
use anyhow::{Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use sha2::Sha256;
use tracing::{debug, warn};

use super::fees::FeeCache;
use super::wallet::WalletKeyStore;
use super::{ExecutionOrder, OrderResult, OrderStatus, Side};
use crate::proxy::HttpPool;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const NEG_RISK_EXCHANGE: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";
const CTF_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";
const CHAIN_ID: u64 = 137;

// ---------------------------------------------------------------------------
// EIP-712 Order struct (Polymarket CTF Exchange)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SubmitOrderResponse {
    #[serde(rename = "orderID")]
    order_id: String,
}

#[derive(Debug, Deserialize)]
struct OrderStatusResponse {
    status: Option<String>,
    #[serde(rename = "associate_trades")]
    associate_trades: Option<Vec<AssociateTrade>>,
}

#[derive(Debug, Deserialize)]
struct AssociateTrade {
    price: Option<String>,
}

// ---------------------------------------------------------------------------
// OrderSubmitter
// ---------------------------------------------------------------------------

/// Polymarket Builder Program authentication credentials.
pub struct BuilderCredentials {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

pub struct OrderSubmitter {
    http: HttpPool,
    clob_url: String,
    credentials: BuilderCredentials,
    wallet_keys: Arc<WalletKeyStore>,
    fee_cache: Arc<FeeCache>,
    neg_risk: bool,
}

impl OrderSubmitter {
    pub fn new(
        http: HttpPool,
        clob_url: &str,
        credentials: BuilderCredentials,
        wallet_keys: Arc<WalletKeyStore>,
        fee_cache: Arc<FeeCache>,
        neg_risk: bool,
    ) -> Self {
        Self {
            http,
            clob_url: clob_url.trim_end_matches('/').to_string(),
            credentials,
            wallet_keys,
            fee_cache,
            neg_risk,
        }
    }

    /// Submit an order to the Polymarket CLOB.
    ///
    /// Flow: build EIP-712 struct -> sign -> POST /order with Builder headers -> poll status.
    pub async fn submit(&self, order: &ExecutionOrder) -> Result<OrderResult> {
        // 1. Get signer and address
        let signer = self
            .wallet_keys
            .get_signer(order.wallet_id)
            .context("failed to get wallet signer")?;
        let maker_address = signer.address();

        // 2. Get fee rate
        let fee_rate_bps = self
            .fee_cache
            .get_fee(&order.token_id)
            .await
            .context("failed to get fee rate")?;

        // 3. Build amounts (USDC has 6 decimals)
        let size_usdc_wei = (order.size_usdc * 1_000_000.0) as u64;
        let price = order.price.unwrap_or(0.5); // fallback for market orders

        let (maker_amount, taker_amount) = match order.side {
            Side::Buy => {
                // BUY: pay USDC (makerAmount), receive tokens (takerAmount)
                let taker_amt = if price > 0.0 {
                    (size_usdc_wei as f64 / price) as u64
                } else {
                    size_usdc_wei
                };
                (size_usdc_wei, taker_amt)
            }
            Side::Sell => {
                // SELL: send tokens (makerAmount), receive USDC (takerAmount)
                let maker_amt = if price > 0.0 {
                    (size_usdc_wei as f64 / price) as u64
                } else {
                    size_usdc_wei
                };
                (maker_amt, size_usdc_wei)
            }
        };

        // 4. Parse token_id as U256 (try decimal first, then hex)
        let token_id_u256 = U256::from_str_radix(&order.token_id, 10)
            .or_else(|_| U256::from_str_radix(&order.token_id, 16))
            .context("failed to parse token_id as U256")?;

        // 5. Generate salt from UUID
        let salt_bytes = order.id.as_bytes();
        let salt = U256::from_be_slice(salt_bytes);

        // 6. Build the ClobOrder struct
        let side_u8: u8 = match order.side {
            Side::Buy => 0,
            Side::Sell => 1,
        };

        let clob_order = ClobOrder {
            salt,
            maker: maker_address,
            signer: maker_address,
            taker: Address::ZERO,
            tokenId: token_id_u256,
            makerAmount: U256::from(maker_amount),
            takerAmount: U256::from(taker_amount),
            expiration: U256::ZERO, // GTC (good-till-cancelled)
            nonce: U256::ZERO,
            feeRateBps: U256::from(fee_rate_bps),
            side: side_u8,
            signatureType: 0, // EOA
        };

        // 7. Build EIP-712 domain
        let exchange_address: Address = if self.neg_risk {
            NEG_RISK_EXCHANGE.parse().context("invalid neg risk exchange address")?
        } else {
            CTF_EXCHANGE.parse().context("invalid CTF exchange address")?
        };

        let domain = eip712_domain! {
            name: "ClobExchange",
            version: "1",
            chain_id: CHAIN_ID,
            verifying_contract: exchange_address,
        };

        // 8. Sign the EIP-712 hash
        let signing_hash = clob_order.eip712_signing_hash(&domain);
        let signature = signer
            .sign_hash_sync(&signing_hash)
            .context("failed to sign EIP-712 hash")?;

        // Encode signature as hex: r (32 bytes) + s (32 bytes) + v (1 byte)
        let r_bytes = signature.r().to_be_bytes::<32>();
        let s_bytes = signature.s().to_be_bytes::<32>();
        let v_byte = if signature.v() { 28u8 } else { 27u8 };
        let mut sig_bytes = Vec::with_capacity(65);
        sig_bytes.extend_from_slice(&r_bytes);
        sig_bytes.extend_from_slice(&s_bytes);
        sig_bytes.push(v_byte);
        let signature_hex = format!("0x{}", hex::encode(&sig_bytes));

        // 9. Build JSON payload
        let order_type_str = if order.price.is_some() { "GTC" } else { "FOK" };

        let payload = serde_json::json!({
            "order": {
                "salt": salt.to_string(),
                "maker": format!("{:?}", maker_address),
                "signer": format!("{:?}", maker_address),
                "taker": format!("{:?}", Address::ZERO),
                "tokenId": order.token_id,
                "makerAmount": maker_amount.to_string(),
                "takerAmount": taker_amount.to_string(),
                "expiration": "0",
                "nonce": "0",
                "feeRateBps": fee_rate_bps.to_string(),
                "side": side_u8.to_string(),
                "signatureType": "0",
                "signature": signature_hex,
            },
            "orderType": order_type_str,
            "negRisk": self.neg_risk,
        });

        let body = serde_json::to_string(&payload).context("failed to serialize order payload")?;

        // 10. Sign builder request (HMAC-SHA256)
        let timestamp = now_millis().to_string();
        let path = "/order";
        let hmac_sig =
            self.sign_builder_request("POST", path, &timestamp, &body)?;

        // 11. Build headers
        let mut headers = HeaderMap::new();
        headers.insert(
            "POLY_ADDRESS",
            HeaderValue::from_str(&format!("{:?}", maker_address))
                .context("invalid address header")?,
        );
        headers.insert(
            "POLY_SIGNATURE",
            HeaderValue::from_str(&hmac_sig).context("invalid signature header")?,
        );
        headers.insert(
            "POLY_TIMESTAMP",
            HeaderValue::from_str(&timestamp).context("invalid timestamp header")?,
        );
        headers.insert(
            "POLY_API_KEY",
            HeaderValue::from_str(&self.credentials.api_key)
                .context("invalid api key header")?,
        );
        headers.insert(
            "POLY_PASSPHRASE",
            HeaderValue::from_str(&self.credentials.passphrase)
                .context("invalid passphrase header")?,
        );

        // 12. POST /order
        let url = format!("{}{}", self.clob_url, path);
        debug!(order_id = %order.id, %url, "submitting order to CLOB");

        let resp = self
            .http
            .proxied()
            .post(&url)
            .headers(headers)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .context("order submission HTTP request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "order submission failed with status {status}: {body}"
            );
        }

        let submit_resp: SubmitOrderResponse = resp
            .json()
            .await
            .context("failed to parse order submission response")?;

        debug!(
            polymarket_order_id = %submit_resp.order_id,
            "order submitted, polling status"
        );

        // 13. Poll for order status + extract filled price
        let (status, filled_price) = self.poll_order_status(&submit_resp.order_id).await;

        Ok(OrderResult {
            polymarket_order_id: submit_resp.order_id,
            status,
            filled_price,
            fee_bps: Some(fee_rate_bps),
        })
    }

    /// Compute HMAC-SHA256 of `{method}{path}{timestamp}{body}`, base64-encoded.
    pub(crate) fn sign_builder_request(
        &self,
        method: &str,
        path: &str,
        timestamp: &str,
        body: &str,
    ) -> Result<String> {
        let secret_bytes = BASE64
            .decode(&self.credentials.secret)
            .context("builder_secret is not valid base64")?;

        let mut mac = Hmac::<Sha256>::new_from_slice(&secret_bytes)
            .context("HMAC key creation failed")?;

        let message = format!("{method}{path}{timestamp}{body}");
        mac.update(message.as_bytes());

        let result = mac.finalize().into_bytes();
        Ok(BASE64.encode(result))
    }

    /// Poll GET /data/order/{id} every 1s, up to 30 times.
    /// Returns (status, filled_price) — price extracted from associate_trades if filled.
    async fn poll_order_status(&self, order_id: &str) -> (OrderStatus, Option<f64>) {
        for attempt in 1..=30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            let url = format!("{}/data/order/{}", self.clob_url, order_id);
            let resp = match self.http.proxied().get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    warn!(attempt, %order_id, error = %e, "poll request failed");
                    continue;
                }
            };

            let status_resp: OrderStatusResponse = match resp.json().await {
                Ok(s) => s,
                Err(e) => {
                    warn!(attempt, %order_id, error = %e, "poll response parse failed");
                    continue;
                }
            };

            match status_resp.status.as_deref() {
                Some("matched") | Some("filled") => {
                    let filled_price = status_resp
                        .associate_trades
                        .as_ref()
                        .and_then(|trades| trades.first())
                        .and_then(|t| t.price.as_ref())
                        .and_then(|p| p.parse::<f64>().ok());
                    debug!(attempt, %order_id, ?filled_price, "order filled");
                    return (OrderStatus::Filled, filled_price);
                }
                Some("cancelled") => {
                    debug!(attempt, %order_id, "order cancelled");
                    return (OrderStatus::Cancelled, None);
                }
                Some("failed") => {
                    debug!(attempt, %order_id, "order failed");
                    return (OrderStatus::Failed, None);
                }
                _ => {
                    debug!(attempt, %order_id, status = ?status_resp.status, "order still pending");
                }
            }
        }

        warn!(%order_id, "order status poll timed out after 30 attempts");
        (OrderStatus::Timeout, None)
    }
}

/// Returns current time in milliseconds since UNIX epoch.
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_millis() as u64
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::fees::FeeCache;
    use crate::execution::wallet::WalletKeyStore;

    fn make_submitter() -> OrderSubmitter {
        let wallet_keys = Arc::new(
            WalletKeyStore::new(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .unwrap(),
        );
        let pool = HttpPool::new(&[], std::time::Duration::from_secs(10)).unwrap();
        let fee_cache = Arc::new(FeeCache::new(
            pool.clone(),
            "http://localhost",
        ));

        // Use a base64-encoded secret for HMAC
        let secret = BASE64.encode(b"test-secret-key-for-hmac-signing");

        let credentials = BuilderCredentials {
            api_key: "test-api-key".to_string(),
            secret,
            passphrase: "test-passphrase".to_string(),
        };

        OrderSubmitter::new(
            pool,
            "http://localhost:8080",
            credentials,
            wallet_keys,
            fee_cache,
            true,
        )
    }

    #[test]
    fn test_builder_signature() {
        let submitter = make_submitter();

        let result = submitter
            .sign_builder_request("POST", "/order", "1700000000000", r#"{"test":"body"}"#)
            .unwrap();

        // Verify result is valid base64
        let decoded = BASE64.decode(&result);
        assert!(decoded.is_ok(), "signature should be valid base64");

        // HMAC-SHA256 produces 32 bytes
        let bytes = decoded.unwrap();
        assert_eq!(bytes.len(), 32, "HMAC-SHA256 should produce 32 bytes");

        // Same inputs should produce the same signature (deterministic)
        let result2 = submitter
            .sign_builder_request("POST", "/order", "1700000000000", r#"{"test":"body"}"#)
            .unwrap();
        assert_eq!(result, result2, "HMAC should be deterministic");

        // Different inputs should produce different signatures
        let result3 = submitter
            .sign_builder_request("GET", "/order", "1700000000000", r#"{"test":"body"}"#)
            .unwrap();
        assert_ne!(result, result3, "different method should produce different signature");
    }

    #[test]
    fn test_now_millis() {
        let ts = now_millis();
        assert!(
            ts > 1_700_000_000_000,
            "timestamp {ts} should be after Nov 2023"
        );
        assert!(
            ts < 3_000_000_000_000,
            "timestamp {ts} should be before year 2065"
        );
    }
}
