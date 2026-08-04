use serde::Serialize;
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::{ContractError, Digest, Result};

pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_jcs::to_vec(value).map_err(ContractError::Canonicalization)
}

pub fn canonical_bytes_without<T: Serialize>(value: &T, field: &str) -> Result<Vec<u8>> {
    let mut json = serde_json::to_value(value)?;
    let Value::Object(ref mut object) = json else {
        return Err(ContractError::ReceiptInvariant(
            "signed envelope must serialize as an object".into(),
        ));
    };
    object.remove(field);
    canonical_bytes(&json)
}

pub fn hash_bytes(bytes: &[u8]) -> Digest {
    Digest::from_bytes(bytes)
}

pub fn hash_serializable<T: Serialize>(value: &T) -> Result<Digest> {
    Ok(hash_bytes(&canonical_bytes(value)?))
}

pub(crate) fn sha256_raw(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
