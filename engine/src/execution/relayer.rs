use std::sync::Arc;

use alloy::primitives::Address;
use anyhow::{Context, Result};
use base64::engine::general_purpose::{STANDARD as BASE64, URL_SAFE as BASE64_URL, URL_SAFE_NO_PAD as BASE64_URL_NOPAD};
use base64::Engine as _;
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tracing::{debug, warn};

use super::orders::BuilderCredentials;
use super::wallet::WalletKeyStore;
use crate::proxy::HttpPool;

// ---------------------------------------------------------------------------
// Polygon contract addresses
// ---------------------------------------------------------------------------

const SAFE_FACTORY: &str = "0xaacFeEa03eb1561C4e67d661e40682Bd20E3541b";
const SAFE_MULTISEND: &str = "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761";
/// USDC.e on Polygon
const USDC_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
const NEG_RISK_EXCHANGE: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";
const CTF_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";

// ---------------------------------------------------------------------------
// Relayer response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct NoncePayload {
    nonce: String,
}

#[derive(Debug, Deserialize)]
struct RelayPayload {
    address: String,
    nonce: String,
}

#[derive(Debug, Deserialize)]
struct SubmitResponse {
    #[serde(rename = "transactionID")]
    transaction_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelayerTransaction {
    #[serde(rename = "transactionID")]
    pub transaction_id: String,
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<String>,
    pub state: String,
    #[serde(rename = "proxyAddress")]
    pub proxy_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeployedResponse {
    deployed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SafeDeployResult {
    pub safe_address: String,
    pub transaction_hash: String,
}

// ---------------------------------------------------------------------------
// RelayerClient
// ---------------------------------------------------------------------------

pub struct RelayerClient {
    http: HttpPool,
    relayer_url: String,
    credentials: BuilderCredentials,
    wallet_keys: Arc<WalletKeyStore>,
}

impl RelayerClient {
    pub fn new(
        http: HttpPool,
        relayer_url: &str,
        credentials: BuilderCredentials,
        wallet_keys: Arc<WalletKeyStore>,
    ) -> Self {
        Self {
            http,
            relayer_url: relayer_url.trim_end_matches('/').to_string(),
            credentials,
            wallet_keys,
        }
    }

    /// Deploy a Gnosis Safe for the given wallet, then approve USDC on both exchanges.
    pub async fn deploy_safe(&self, wallet_id: u64) -> Result<SafeDeployResult> {
        let signer = self.wallet_keys.get_signer(wallet_id)?;
        let signer_address = signer.address();

        // 1. Check if already deployed
        if self.is_deployed(&signer_address).await? {
            anyhow::bail!("Safe already deployed for signer {signer_address:?}");
        }

        // 2. Build and submit Safe creation transaction
        debug!(%wallet_id, ?signer_address, "deploying_safe");

        let nonce = self.get_nonce(&signer_address).await?;

        let create_payload = serde_json::json!({
            "type": "SAFE_CREATE",
            "from": format!("{:?}", signer_address),
            "to": SAFE_FACTORY,
            "data": "",
            "signature": "",
            "signatureParams": {
                "paymentToken": format!("{:?}", Address::ZERO),
                "payment": "0",
                "paymentReceiver": format!("{:?}", Address::ZERO)
            },
            "nonce": nonce,
        });

        let tx_id = self.submit_transaction(&create_payload).await?;
        let tx = self.poll_transaction(&tx_id).await?;

        let safe_address = tx
            .proxy_address
            .with_context(|| "relayer did not return proxyAddress for Safe deploy")?;

        let tx_hash = tx
            .transaction_hash
            .unwrap_or_default();

        debug!(%wallet_id, %safe_address, %tx_hash, "safe_deployed");

        // 3. Store Safe address in wallet key store
        let addr: Address = safe_address.parse().context("invalid safe address from relayer")?;
        self.wallet_keys.store_safe_address(wallet_id, addr)?;

        // 4. Approve USDC on both CTF and NegRisk exchanges
        self.approve_usdc(wallet_id, &safe_address).await?;

        Ok(SafeDeployResult {
            safe_address,
            transaction_hash: tx_hash,
        })
    }

    /// Approve USDC spending for both CTF and NegRisk exchanges.
    async fn approve_usdc(&self, wallet_id: u64, safe_address: &str) -> Result<()> {
        debug!(%wallet_id, %safe_address, "approving_usdc");

        let safe_addr: Address = safe_address.parse()?;
        let relay = self.get_relay_payload(&safe_addr).await?;

        // ERC20 approve(address,uint256) selector = 0x095ea7b3
        // Approve max uint256 for both exchanges
        let max_uint = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let ctf_padded = format!("{:0>64}", &CTF_EXCHANGE[2..]);
        let neg_risk_padded = format!("{:0>64}", &NEG_RISK_EXCHANGE[2..]);

        let approve_ctf_data = format!("0x095ea7b3{ctf_padded}{max_uint}");
        let approve_neg_data = format!("0x095ea7b3{neg_risk_padded}{max_uint}");

        // Submit as a Safe transaction batch
        let payload = serde_json::json!({
            "type": "SAFE",
            "from": safe_address,
            "to": USDC_ADDRESS,
            "data": approve_ctf_data,
            "signature": "",
            "signatureParams": {},
            "nonce": relay.nonce,
            "metadata": "USDC approval for CTF Exchange"
        });

        let tx_id = self.submit_transaction(&payload).await?;
        self.poll_transaction(&tx_id).await?;

        // Second approval for NegRisk exchange
        let relay2 = self.get_relay_payload(&safe_addr).await?;
        let payload2 = serde_json::json!({
            "type": "SAFE",
            "from": safe_address,
            "to": USDC_ADDRESS,
            "data": approve_neg_data,
            "signature": "",
            "signatureParams": {},
            "nonce": relay2.nonce,
            "metadata": "USDC approval for NegRisk Exchange"
        });

        let tx_id2 = self.submit_transaction(&payload2).await?;
        self.poll_transaction(&tx_id2).await?;

        debug!(%wallet_id, "usdc_approved_for_both_exchanges");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Relayer HTTP helpers
    // -----------------------------------------------------------------------

    async fn is_deployed(&self, address: &Address) -> Result<bool> {
        let url = format!("{}/deployed?address={:?}", self.relayer_url, address);
        let resp: DeployedResponse = self
            .http
            .proxied()
            .get(&url)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.deployed)
    }

    async fn get_nonce(&self, address: &Address) -> Result<String> {
        let url = format!(
            "{}/nonce?address={:?}&type=SAFE",
            self.relayer_url, address
        );
        let resp: NoncePayload = self
            .http
            .proxied()
            .get(&url)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.nonce)
    }

    async fn get_relay_payload(&self, address: &Address) -> Result<RelayPayload> {
        let url = format!(
            "{}/relay-payload?address={:?}&type=SAFE",
            self.relayer_url, address
        );
        let resp: RelayPayload = self
            .http
            .proxied()
            .get(&url)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    async fn submit_transaction(&self, payload: &serde_json::Value) -> Result<String> {
        let body = serde_json::to_string(payload)?;

        let timestamp = now_secs().to_string();
        let hmac_sig = self.sign_request("POST", "/submit", &timestamp, &body)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "POLY_BUILDER_API_KEY",
            HeaderValue::from_str(&self.credentials.api_key)?,
        );
        headers.insert(
            "POLY_BUILDER_SIGNATURE",
            HeaderValue::from_str(&hmac_sig)?,
        );
        headers.insert(
            "POLY_BUILDER_TIMESTAMP",
            HeaderValue::from_str(&timestamp)?,
        );
        headers.insert(
            "POLY_BUILDER_PASSPHRASE",
            HeaderValue::from_str(&self.credentials.passphrase)?,
        );

        let url = format!("{}/submit", self.relayer_url);
        let resp = self
            .http
            .proxied()
            .post(&url)
            .headers(headers)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("relayer submit failed ({status}): {body}");
        }

        let submit: SubmitResponse = resp.json().await?;
        Ok(submit.transaction_id)
    }

    async fn poll_transaction(&self, tx_id: &str) -> Result<RelayerTransaction> {
        for attempt in 1..=60 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let url = format!("{}/transaction?id={}", self.relayer_url, tx_id);
            let resp = match self.http.proxied().get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    warn!(attempt, %tx_id, error = %e, "relayer_poll_failed");
                    continue;
                }
            };

            let txs: Vec<RelayerTransaction> = match resp.json().await {
                Ok(t) => t,
                Err(e) => {
                    warn!(attempt, %tx_id, error = %e, "relayer_poll_parse_failed");
                    continue;
                }
            };

            if let Some(tx) = txs.first() {
                match tx.state.as_str() {
                    "STATE_MINED" | "STATE_CONFIRMED" | "STATE_EXECUTED" => {
                        debug!(attempt, %tx_id, state = %tx.state, "relayer_tx_confirmed");
                        return Ok(tx.clone());
                    }
                    "STATE_FAILED" | "STATE_INVALID" => {
                        anyhow::bail!("relayer transaction {tx_id} failed with state: {}", tx.state);
                    }
                    _ => {
                        debug!(attempt, %tx_id, state = %tx.state, "relayer_tx_pending");
                    }
                }
            }
        }

        anyhow::bail!("relayer transaction {tx_id} poll timed out after 120s");
    }

    /// HMAC-SHA256 signature for builder auth (same pattern as CLOB API).
    fn sign_request(
        &self,
        method: &str,
        path: &str,
        timestamp: &str,
        body: &str,
    ) -> Result<String> {
        let secret_bytes = BASE64_URL
            .decode(&self.credentials.secret)
            .or_else(|_| BASE64_URL_NOPAD.decode(&self.credentials.secret))
            .or_else(|_| BASE64.decode(&self.credentials.secret))
            .context("builder_secret is not valid base64")?;

        let mut mac = Hmac::<Sha256>::new_from_slice(&secret_bytes)
            .context("HMAC key creation failed")?;

        let message = format!("{timestamp}{method}{path}{body}");
        mac.update(message.as_bytes());

        let result = mac.finalize().into_bytes();
        Ok(BASE64.encode(result))
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs()
}
