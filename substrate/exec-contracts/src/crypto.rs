use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer as _, SigningKey as Ed25519SigningKey};
use p256::ecdsa::{SigningKey as P256SigningKey, VerifyingKey as P256VerifyingKey};

use crate::{
    canonical::sha256_raw, ContractError, ExecutionReceipt, Result, SignatureAlgorithm,
    SignedExecRequest,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationKey {
    pub algorithm: SignatureAlgorithm,
    pub bytes: Vec<u8>,
}

impl VerificationKey {
    pub fn ed25519(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            algorithm: SignatureAlgorithm::Ed25519,
            bytes: bytes.into(),
        }
    }

    pub fn p256_sec1(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            algorithm: SignatureAlgorithm::P256,
            bytes: bytes.into(),
        }
    }

    pub fn from_base64url(algorithm: SignatureAlgorithm, encoded: &str) -> Result<Self> {
        let bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|error| ContractError::InvalidPublicKey(error.to_string()))?;
        Ok(Self { algorithm, bytes })
    }

    pub fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(&self.bytes)
    }

    pub fn key_id(&self) -> String {
        key_id(self.algorithm, &self.bytes)
    }
}

pub fn key_id(algorithm: SignatureAlgorithm, public_key: &[u8]) -> String {
    format!("{}:{}", algorithm, hex::encode(sha256_raw(public_key)))
}

pub fn sign_request_ed25519(
    request: &mut SignedExecRequest,
    key: &Ed25519SigningKey,
) -> Result<()> {
    let public = key.verifying_key().to_bytes();
    request.sig_alg = SignatureAlgorithm::Ed25519;
    request.signer_key_id = Some(key_id(request.sig_alg, &public));
    request.signature = None;
    let signature: ed25519_dalek::Signature = key.sign(&request.canonical_unsigned()?);
    request.signature = Some(URL_SAFE_NO_PAD.encode(signature.to_bytes()));
    Ok(())
}

pub fn sign_request_p256(request: &mut SignedExecRequest, key: &P256SigningKey) -> Result<()> {
    let public = key.verifying_key().to_encoded_point(true);
    request.sig_alg = SignatureAlgorithm::P256;
    request.signer_key_id = Some(key_id(request.sig_alg, public.as_bytes()));
    request.signature = None;
    let signature: p256::ecdsa::Signature = key.sign(&request.canonical_unsigned()?);
    request.signature = Some(URL_SAFE_NO_PAD.encode(signature.to_bytes()));
    Ok(())
}

pub fn sign_receipt_ed25519(receipt: &mut ExecutionReceipt, key: &Ed25519SigningKey) -> Result<()> {
    let public = key.verifying_key().to_bytes();
    receipt.executing_device.sig_alg = SignatureAlgorithm::Ed25519;
    receipt.executing_device.key_id = key_id(receipt.executing_device.sig_alg, &public);
    receipt.signature = None;
    let signature: ed25519_dalek::Signature = key.sign(&receipt.canonical_unsigned()?);
    receipt.signature = Some(URL_SAFE_NO_PAD.encode(signature.to_bytes()));
    Ok(())
}

pub fn sign_receipt_p256(receipt: &mut ExecutionReceipt, key: &P256SigningKey) -> Result<()> {
    let public = key.verifying_key().to_encoded_point(true);
    receipt.executing_device.sig_alg = SignatureAlgorithm::P256;
    receipt.executing_device.key_id = key_id(receipt.executing_device.sig_alg, public.as_bytes());
    receipt.signature = None;
    let signature: p256::ecdsa::Signature = key.sign(&receipt.canonical_unsigned()?);
    receipt.signature = Some(URL_SAFE_NO_PAD.encode(signature.to_bytes()));
    Ok(())
}

pub fn verify_request_signature(request: &SignedExecRequest, key: &VerificationKey) -> Result<()> {
    let signer = request
        .signer_key_id
        .as_deref()
        .ok_or_else(|| ContractError::InvalidSignature("request has no signerKeyId".into()))?;
    let signature = request
        .signature
        .as_deref()
        .ok_or_else(|| ContractError::InvalidSignature("request has no signature".into()))?;
    verify(
        request.sig_alg,
        signer,
        signature,
        &request.canonical_unsigned()?,
        key,
    )
}

pub fn verify_receipt_signature(receipt: &ExecutionReceipt, key: &VerificationKey) -> Result<()> {
    let signature = receipt
        .signature
        .as_deref()
        .ok_or_else(|| ContractError::InvalidSignature("receipt has no signature".into()))?;
    verify(
        receipt.executing_device.sig_alg,
        &receipt.executing_device.key_id,
        signature,
        &receipt.canonical_unsigned()?,
        key,
    )
}

fn verify(
    algorithm: SignatureAlgorithm,
    declared_key_id: &str,
    signature: &str,
    payload: &[u8],
    key: &VerificationKey,
) -> Result<()> {
    if algorithm != key.algorithm {
        return Err(ContractError::AlgorithmMismatch);
    }
    let expected_key_id = key.key_id();
    if declared_key_id != expected_key_id {
        return Err(ContractError::KeyIdMismatch {
            expected: expected_key_id,
            actual: declared_key_id.into(),
        });
    }
    let signature = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|error| ContractError::InvalidSignature(error.to_string()))?;
    match algorithm {
        SignatureAlgorithm::Ed25519 => {
            let key_bytes: [u8; 32] = key.bytes.as_slice().try_into().map_err(|_| {
                ContractError::InvalidPublicKey("Ed25519 key must be 32 bytes".into())
            })?;
            let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes)
                .map_err(|error| ContractError::InvalidPublicKey(error.to_string()))?;
            let signature = ed25519_dalek::Signature::from_slice(&signature)
                .map_err(|error| ContractError::InvalidSignature(error.to_string()))?;
            verifying_key
                .verify_strict(payload, &signature)
                .map_err(|_| ContractError::SignatureVerification)
        }
        SignatureAlgorithm::P256 => {
            use p256::ecdsa::signature::Verifier as _;
            let verifying_key = P256VerifyingKey::from_sec1_bytes(&key.bytes)
                .map_err(|error| ContractError::InvalidPublicKey(error.to_string()))?;
            let signature = p256::ecdsa::Signature::from_slice(&signature)
                .map_err(|error| ContractError::InvalidSignature(error.to_string()))?;
            verifying_key
                .verify(payload, &signature)
                .map_err(|_| ContractError::SignatureVerification)
        }
    }
}
