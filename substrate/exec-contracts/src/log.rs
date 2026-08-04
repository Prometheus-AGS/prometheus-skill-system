use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    canonical_bytes_without, hash_bytes, verify_receipt, ContractError, Digest, ExecutionReceipt,
    Result, SignatureAlgorithm, VerificationKey, VerificationResult, SCHEMA_VERSION,
};

pub const MAX_SEGMENT_ENTRIES: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptLogSegmentHeader {
    pub schema_version: String,
    pub sequence: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_segment_hash: Option<Digest>,
    pub created_at: DateTime<Utc>,
    pub receipt_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptLogEntry {
    pub receipt_hash: Digest,
    pub receipt: ExecutionReceipt,
}

impl ReceiptLogEntry {
    pub fn new(receipt: ExecutionReceipt) -> Result<Self> {
        let receipt_hash = receipt.receipt_hash()?;
        Ok(Self {
            receipt_hash,
            receipt,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptLogSegment {
    pub header: ReceiptLogSegmentHeader,
    pub entries: Vec<ReceiptLogEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment_hash: Option<Digest>,
}

impl ReceiptLogSegment {
    pub fn seal(
        sequence: u64,
        previous_segment_hash: Option<Digest>,
        created_at: DateTime<Utc>,
        entries: Vec<ReceiptLogEntry>,
    ) -> Result<Self> {
        if entries.len() > MAX_SEGMENT_ENTRIES {
            return Err(ContractError::ReceiptLog(format!(
                "segment contains {} entries; maximum is {}",
                entries.len(),
                MAX_SEGMENT_ENTRIES
            )));
        }
        let mut segment = Self {
            header: ReceiptLogSegmentHeader {
                schema_version: SCHEMA_VERSION.into(),
                sequence,
                previous_segment_hash,
                created_at,
                receipt_count: entries.len() as u32,
            },
            entries,
            segment_hash: None,
        };
        segment.segment_hash = Some(segment.compute_hash()?);
        Ok(segment)
    }

    pub fn compute_hash(&self) -> Result<Digest> {
        Ok(hash_bytes(&canonical_bytes_without(self, "segmentHash")?))
    }

    pub fn verify<F>(
        &self,
        expected_previous: Option<&Digest>,
        mut resolve_key: F,
    ) -> Result<Vec<VerificationResult>>
    where
        F: FnMut(&str, SignatureAlgorithm) -> Option<VerificationKey>,
    {
        crate::ensure_schema(&self.header.schema_version)?;
        if self.entries.len() > MAX_SEGMENT_ENTRIES {
            return Err(ContractError::ReceiptLog(
                "segment entry limit exceeded".into(),
            ));
        }
        if self.header.receipt_count as usize != self.entries.len() {
            return Err(ContractError::ReceiptLog("receipt count mismatch".into()));
        }
        if self.header.previous_segment_hash.as_ref() != expected_previous {
            return Err(ContractError::ReceiptLog(
                "previous segment hash mismatch".into(),
            ));
        }
        let declared = self
            .segment_hash
            .as_ref()
            .ok_or_else(|| ContractError::ReceiptLog("unsealed segment".into()))?;
        let computed = self.compute_hash()?;
        if declared != &computed {
            return Err(ContractError::ReceiptLog("segment hash mismatch".into()));
        }

        let mut results = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            if entry.receipt.receipt_hash()? != entry.receipt_hash {
                return Err(ContractError::ReceiptLog(format!(
                    "receipt entry hash mismatch for run {}",
                    entry.receipt.run_id
                )));
            }
            let device = &entry.receipt.executing_device;
            let key = resolve_key(&device.key_id, device.sig_alg).ok_or_else(|| {
                ContractError::ReceiptLog(format!("unresolved signer key: {}", device.key_id))
            })?;
            let result = verify_receipt(&entry.receipt, &key, None, None);
            if !result.valid {
                return Err(ContractError::ReceiptLog(format!(
                    "invalid receipt {}",
                    entry.receipt.run_id
                )));
            }
            results.push(result);
        }
        Ok(results)
    }
}
