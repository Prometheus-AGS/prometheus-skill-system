use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer as _, SigningKey, Verifier as _, VerifyingKey};
use prometheus_exec_contracts::{canonical_bytes_without, key_id, SignatureAlgorithm};

use crate::{EnrollmentSnapshot, RemoteError, Result, SignedRemoteDispatch};

pub fn sign_dispatch_ed25519(
    dispatch: &mut SignedRemoteDispatch,
    signing_key: &SigningKey,
) -> Result<()> {
    dispatch.sig_alg = SignatureAlgorithm::Ed25519;
    dispatch.signer_key_id = key_id(
        SignatureAlgorithm::Ed25519,
        &signing_key.verifying_key().to_bytes(),
    );
    dispatch.signature = None;
    let payload = canonical_bytes_without(dispatch, "signature")?;
    dispatch.signature = Some(URL_SAFE_NO_PAD.encode(signing_key.sign(&payload).to_bytes()));
    dispatch.validate()
}

pub fn verify_dispatch(
    dispatch: &SignedRemoteDispatch,
    enrollment: &EnrollmentSnapshot,
) -> Result<()> {
    dispatch.validate()?;
    enrollment.validate()?;
    if enrollment.snapshot_hash()? != dispatch.enrollment_snapshot_hash {
        return Err(RemoteError::Contract(
            "dispatch enrollment snapshot hash does not match".into(),
        ));
    }
    let binding = enrollment.binding(&dispatch.origin_endpoint_id)?;
    if binding.sig_alg != dispatch.sig_alg || binding.key_id != dispatch.signer_key_id {
        return Err(RemoteError::SignerMismatch(
            dispatch.origin_endpoint_id.clone(),
        ));
    }
    if binding.sig_alg != SignatureAlgorithm::Ed25519 {
        return Err(RemoteError::Signature(
            "remote v1 supports Ed25519 dispatch signatures".into(),
        ));
    }
    let public = URL_SAFE_NO_PAD
        .decode(&binding.public_key)
        .map_err(|error| RemoteError::Signature(error.to_string()))?;
    let public: [u8; 32] = public
        .as_slice()
        .try_into()
        .map_err(|_| RemoteError::Signature("Ed25519 public key must be 32 bytes".into()))?;
    let key = VerifyingKey::from_bytes(&public)
        .map_err(|error| RemoteError::Signature(error.to_string()))?;
    let signature = URL_SAFE_NO_PAD
        .decode(
            dispatch
                .signature
                .as_deref()
                .ok_or_else(|| RemoteError::Signature("missing signature".into()))?,
        )
        .map_err(|error| RemoteError::Signature(error.to_string()))?;
    let signature = ed25519_dalek::Signature::from_slice(&signature)
        .map_err(|error| RemoteError::Signature(error.to_string()))?;
    key.verify(&canonical_bytes_without(dispatch, "signature")?, &signature)
        .map_err(|_| RemoteError::Signature("signature verification failed".into()))
}
