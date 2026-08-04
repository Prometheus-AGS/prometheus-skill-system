use std::collections::{BTreeMap, BTreeSet};

use prometheus_exec_contracts::{
    hash_bytes, hash_serializable, Digest, ExecutionGrant, SignedExecRequest,
};
use thiserror::Error;

/// Complete one-shot input passed to an execution port.
#[derive(Clone, Debug)]
pub struct ExecutionJob {
    pub request: SignedExecRequest,
    pub code: Vec<u8>,
    pub inputs: BTreeMap<String, Vec<u8>>,
    pub grants: Vec<ExecutionGrant>,
}

/// An execution job whose declared hashes and revision have been checked.
#[derive(Clone, Debug)]
pub struct ValidatedExecutionJob {
    job: ExecutionJob,
    input_set_hash: Digest,
}

impl ValidatedExecutionJob {
    pub fn request(&self) -> &SignedExecRequest {
        &self.job.request
    }

    pub fn code(&self) -> &[u8] {
        &self.job.code
    }

    pub fn inputs(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.job.inputs
    }

    pub fn input_set_hash(&self) -> &Digest {
        &self.input_set_hash
    }

    pub fn grants(&self) -> &[ExecutionGrant] {
        &self.job.grants
    }

    pub fn into_inner(self) -> ExecutionJob {
        self.job
    }
}

#[derive(Debug, Error)]
pub enum JobValidationError {
    #[error("request contract is invalid: {0}")]
    InvalidRequest(#[from] prometheus_exec_contracts::ContractError),
    #[error("code hash mismatch: declared {declared}, observed {observed}")]
    CodeHashMismatch { declared: Digest, observed: Digest },
    #[error("duplicate declared input name: {0}")]
    DuplicateInput(String),
    #[error("missing input bytes for {0}")]
    MissingInput(String),
    #[error("undeclared input bytes supplied for {0}")]
    UndeclaredInput(String),
    #[error("input hash mismatch for {name}: declared {declared}, observed {observed}")]
    InputHashMismatch {
        name: String,
        declared: Digest,
        observed: Digest,
    },
}

impl ExecutionJob {
    pub fn validate(self) -> Result<ValidatedExecutionJob, JobValidationError> {
        self.request.validate()?;

        let observed_code_hash = hash_bytes(&self.code);
        if observed_code_hash != self.request.code.hash {
            return Err(JobValidationError::CodeHashMismatch {
                declared: self.request.code.hash.clone(),
                observed: observed_code_hash,
            });
        }

        let mut declared_names = BTreeSet::new();
        let mut canonical_inputs = BTreeMap::new();
        for input in &self.request.inputs {
            if !declared_names.insert(input.name.as_str()) {
                return Err(JobValidationError::DuplicateInput(input.name.clone()));
            }
            let bytes = self
                .inputs
                .get(&input.name)
                .ok_or_else(|| JobValidationError::MissingInput(input.name.clone()))?;
            let observed = hash_bytes(bytes);
            if observed != input.hash {
                return Err(JobValidationError::InputHashMismatch {
                    name: input.name.clone(),
                    declared: input.hash.clone(),
                    observed,
                });
            }
            canonical_inputs.insert(input.name.clone(), input.hash.clone());
        }

        if let Some(extra) = self
            .inputs
            .keys()
            .find(|name| !declared_names.contains(name.as_str()))
        {
            return Err(JobValidationError::UndeclaredInput(extra.clone()));
        }

        let input_set_hash = hash_serializable(&canonical_inputs)?;
        Ok(ValidatedExecutionJob {
            job: self,
            input_set_hash,
        })
    }
}
