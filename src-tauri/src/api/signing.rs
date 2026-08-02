//! Device-key upload signing (ECDSA P-256) for `POST /v6/chat/media/upload`.
//!
//! Protocol: https://opengrind.org/grindr-api/security-headers#device-key-upload-signing
//! Compatible with the open-grind `grindr` crate's signing path without replacing
//! our custom transport client.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use p256::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wreq::Client;

use crate::error::AppError;

use super::client::BASE_URL;
use super::headers::DeviceInfo;

#[derive(Clone, Serialize, Deserialize)]
struct StoredSigningKey {
    /// PKCS#8 DER (base64 std) of the private key
    private_pkcs8_b64: String,
    /// base64url(SPKI)
    public_key: String,
    /// base64url(SHA-256(SPKI))
    key_id: String,
    /// Profile id this key was registered for
    user_id: String,
}

pub(crate) struct DeviceKey {
    signing: SigningKey,
    public_key: String,
    key_id: String,
    user_id: String,
}

impl DeviceKey {
    fn generate(user_id: &str) -> Result<Self, AppError> {
        let signing = SigningKey::random(&mut OsRng);
        let verifying = signing.verifying_key();
        let spki = verifying
            .to_public_key_der()
            .map_err(|e| AppError::Auth(format!("SPKI encode failed: {e}")))?;
        let spki_bytes = spki.as_bytes();
        let public_key = URL_SAFE_NO_PAD.encode(spki_bytes);
        let key_id = URL_SAFE_NO_PAD.encode(Sha256::digest(spki_bytes));
        Ok(Self {
            signing,
            public_key,
            key_id,
            user_id: user_id.to_owned(),
        })
    }

    fn from_stored(stored: StoredSigningKey) -> Result<Self, AppError> {
        let der = base64::engine::general_purpose::STANDARD
            .decode(&stored.private_pkcs8_b64)
            .map_err(|e| AppError::Auth(format!("signing key b64 decode: {e}")))?;
        let signing = SigningKey::from_pkcs8_der(&der)
            .map_err(|e| AppError::Auth(format!("signing key PKCS8 decode: {e}")))?;
        Ok(Self {
            signing,
            public_key: stored.public_key,
            key_id: stored.key_id,
            user_id: stored.user_id,
        })
    }

    fn to_stored(&self) -> Result<StoredSigningKey, AppError> {
        let pkcs8 = self
            .signing
            .to_pkcs8_der()
            .map_err(|e| AppError::Auth(format!("PKCS8 encode failed: {e}")))?;
        Ok(StoredSigningKey {
            private_pkcs8_b64: base64::engine::general_purpose::STANDARD.encode(pkcs8.as_bytes()),
            public_key: self.public_key.clone(),
            key_id: self.key_id.clone(),
            user_id: self.user_id.clone(),
        })
    }

    fn sign_message(&self, message: &[u8]) -> Result<String, AppError> {
        let sig: Signature = self.signing.sign(message);
        let der = sig.to_der();
        Ok(URL_SAFE_NO_PAD.encode(der.as_bytes()))
    }
}

struct SigningKeyStorage;

impl SigningKeyStorage {
    fn entry() -> Result<keyring_core::Entry, AppError> {
        keyring_core::Entry::new("open-grind", "device-signing-key")
            .map_err(|e| AppError::Auth(e.to_string()))
    }

    fn load() -> Result<Option<DeviceKey>, AppError> {
        let bytes = match Self::entry()?.get_secret() {
            Ok(b) => b,
            Err(keyring_core::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(AppError::Auth(e.to_string())),
        };
        let stored: StoredSigningKey = rmp_serde::from_slice(&bytes)
            .map_err(|e| AppError::Auth(format!("signing key decode: {e}")))?;
        Ok(Some(DeviceKey::from_stored(stored)?))
    }

    fn save(key: &DeviceKey) -> Result<(), AppError> {
        let stored = key.to_stored()?;
        let bytes = rmp_serde::encode::to_vec(&stored)
            .map_err(|e| AppError::Auth(format!("signing key encode: {e}")))?;
        Self::entry()?
            .set_secret(&bytes)
            .map_err(|e| AppError::Auth(e.to_string()))
    }
}

/// Ensure a device signing key is registered for this user; returns key material.
pub(crate) async fn ensure_device_signing_key(
    http: &Client,
    authorization: &str,
    user_agent: &str,
    device: &DeviceInfo,
    user_id: &str,
) -> Result<(String, DeviceKey), AppError> {
    let android_id = device.device_id.clone();

    if let Some(existing) = SigningKeyStorage::load()? {
        if existing.user_id == user_id {
            return Ok((android_id, existing));
        }
    }

    let key = DeviceKey::generate(user_id)?;

    // 1) Challenge
    let challenge_resp = http
        .post(format!("{BASE_URL}/v1/verification/device-keys/challenge"))
        .header("Authorization", authorization)
        .header("User-Agent", user_agent)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| AppError::Http(format!("device-key challenge failed: {e}")))?;
    let challenge_status = challenge_resp.status().as_u16();
    let challenge_body = challenge_resp.text().await.unwrap_or_default();
    if !(200..300).contains(&challenge_status) {
        return Err(AppError::Http(format!(
            "device-key challenge HTTP {challenge_status}: {}",
            challenge_body.chars().take(200).collect::<String>()
        )));
    }
    let challenge_json: serde_json::Value = serde_json::from_str(&challenge_body)
        .map_err(|e| AppError::Http(format!("challenge JSON: {e}")))?;
    let challenge = challenge_json
        .get("challenge")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Http("challenge missing".into()))?;

    // registration string: userId|keyId|publicKey|androidId|challenge
    let reg_msg = format!(
        "{}|{}|{}|{}|{}",
        user_id, key.key_id, key.public_key, android_id, challenge
    );
    let registration_signature = key.sign_message(reg_msg.as_bytes())?;

    let reg_body = serde_json::json!({
        "publicKey": key.public_key,
        "keyId": key.key_id,
        "registrationSignature": registration_signature,
    });

    let reg_resp = http
        .post(format!("{BASE_URL}/v1/verification/device-keys"))
        .header("Authorization", authorization)
        .header("User-Agent", user_agent)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&reg_body)
        .send()
        .await
        .map_err(|e| AppError::Http(format!("device-key register failed: {e}")))?;
    let reg_status = reg_resp.status().as_u16();
    let reg_text = reg_resp.text().await.unwrap_or_default();
    if !(200..300).contains(&reg_status) {
        return Err(AppError::Http(format!(
            "device-key register HTTP {reg_status}: {}",
            reg_text.chars().take(200).collect::<String>()
        )));
    }

    SigningKeyStorage::save(&key)?;
    Ok((android_id, key))
}

/// Build the four signing headers for a raw body upload.
pub(crate) fn build_upload_signing_headers(
    key: &DeviceKey,
    android_id: &str,
    body: &[u8],
) -> Result<Vec<(String, String)>, AppError> {
    let nonce_bytes: [u8; 32] = rand::random();
    let nonce = URL_SAFE_NO_PAD.encode(nonce_bytes);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string();
    let body_hash = URL_SAFE_NO_PAD.encode(Sha256::digest(body));

    // bodyHash|timestamp|userId|androidId|nonce
    let upload_string = format!(
        "{}|{}|{}|{}|{}",
        body_hash, timestamp, key.user_id, android_id, nonce
    );
    let sig = key.sign_message(upload_string.as_bytes())?;

    Ok(vec![
        ("X-Key-Id".into(), key.key_id.clone()),
        ("X-Sig".into(), sig),
        ("X-Timestamp".into(), timestamp),
        ("X-Nonce".into(), nonce),
    ])
}


