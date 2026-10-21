//! `prometheus kbd phase bind-contract | close | disposition | successor start`
//! (issue #192): the single typed authority for phase closure.
//!
//! Exit codes: 0 recorded or identical no-op; 1 usage/IO; 4 phase not found;
//! 6 the phase is already closed under a different identity; 7 refused as
//! invalid (unbound, missing PASS evidence, budget regression, run/contract
//! mismatch, or missing/invalid scope authority).

use super::{state_or_initialize, state_or_replay, submit_typed, ControlClient};
use anyhow::{anyhow, Context, Result};
use kbd_runtime::phase_closure::{self, ScenarioOutcome};
use kbd_runtime::{
    CommandKind, ContractBindingInput, PhaseCloseInput, PhaseDisposition, Runtime, RuntimeError,
    RuntimeState, ScopeAuthorityInput,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const EXIT_NOT_FOUND: i32 = 4;
const EXIT_ALREADY_CLOSED: i32 = 6;
const EXIT_REFUSED: i32 = 7;
const NO_EVIDENCE_DIGEST: &str = "none";
const COMMAND_ID_DIGEST_CHARS: usize = 16;
const METERED_CLASSES: [&str; 2] = ["measured", "estimate"];

pub enum PhaseClosureAction {
    BindContract {
        command_id: Option<String>,
        phase_id: String,
        contract_path: String,
        contract_digest: String,
        scenario_digests: Vec<String>,
        plan_revision: u64,
        adoption_decision: String,
        amendment: Option<PathBuf>,
        signature: Option<PathBuf>,
    },
    Close {
        command_id: Option<String>,
        phase_id: String,
        evidence_file: Option<PathBuf>,
        disposition: PhaseDisposition,
        reason_code: Option<String>,
    },
    Disposition {
        phase_id: String,
        json: bool,
    },
    SuccessorStart {
        command_id: Option<String>,
        predecessor_phase_id: String,
        phase_id: String,
        title: String,
        slug: Option<String>,
        exact_next_work: Option<String>,
        authorization: PathBuf,
        signature: PathBuf,
    },
}

pub async fn run(
    runtime: &Runtime,
    client: &ControlClient,
    action: PhaseClosureAction,
) -> Result<()> {
    if let PhaseClosureAction::Disposition { phase_id, json } = &action {
        let state = state_or_replay(client, runtime).await?;
        return match phase_closure::disposition_view(&state, phase_id) {
            Ok(view) if *json => {
                println!("{}", serde_json::to_string_pretty(&view)?);
                Ok(())
            }
            Ok(view) => {
                println!("{}", view.disposition);
                Ok(())
            }
            Err(error) => refuse(&anyhow::Error::new(error)),
        };
    }
    let state = state_or_initialize(client, runtime).await?;
    let (derived_id, phase_id, command) = build(&state, action)?;
    match phase_closure::idempotent_replay(&state, &command) {
        Ok(Some(revision)) => {
            return print_outcome(&state, &phase_id, &derived_id, &command, revision, true)
        }
        Ok(None) => {}
        Err(error) => return refuse(&anyhow::Error::new(error)),
    }
    match submit_typed(client, &state, derived_id.clone(), command.clone()).await {
        Ok(result) => {
            let state: RuntimeState =
                serde_json::from_value(result["state"].clone()).context("parse committed state")?;
            print_outcome(
                &state,
                &phase_id,
                &derived_id,
                &command,
                state.revision,
                false,
            )
        }
        Err(error) => refuse(&error),
    }
}

fn build(
    state: &RuntimeState,
    action: PhaseClosureAction,
) -> Result<(String, String, CommandKind)> {
    match action {
        PhaseClosureAction::BindContract {
            command_id,
            phase_id,
            contract_path,
            contract_digest,
            scenario_digests,
            plan_revision,
            adoption_decision,
            amendment,
            signature,
        } => {
            let authority = match (amendment, signature) {
                (Some(manifest), Some(signature)) => Some(read_authority(&manifest, &signature)?),
                (None, None) => None,
                _ => {
                    return Err(anyhow!(
                        "--amendment and --signature must be given together"
                    ))
                }
            };
            let derived = format!("phase-bind:{phase_id}:{}", short(&contract_digest));
            let command = CommandKind::PhaseContractBind {
                phase_id: phase_id.clone(),
                binding: ContractBindingInput {
                    contract_path,
                    contract_digest,
                    scenario_digests: parse_scenario_digests(&scenario_digests)?,
                    plan_revision,
                    adoption_decision_id: adoption_decision,
                },
                authority,
            };
            Ok((checked_id(command_id, derived)?, phase_id, command))
        }
        PhaseClosureAction::Close {
            command_id,
            phase_id,
            evidence_file,
            disposition,
            reason_code,
        } => {
            let closure = close_input(
                state,
                &phase_id,
                evidence_file.as_deref(),
                disposition,
                reason_code,
            )?;
            let key = phase_closure::closure_key(&state.project_id, &phase_id, &closure)?;
            let derived = format!("phase-close:{phase_id}:{}", short(&key));
            let command = CommandKind::PhaseClose {
                phase_id: phase_id.clone(),
                closure,
            };
            Ok((checked_id(command_id, derived)?, phase_id, command))
        }
        PhaseClosureAction::SuccessorStart {
            command_id,
            predecessor_phase_id,
            phase_id,
            title,
            slug,
            exact_next_work,
            authorization,
            signature,
        } => {
            let authority = read_authority(&authorization, &signature)?;
            let derived = format!(
                "phase-successor:{predecessor_phase_id}:{phase_id}:{}",
                short(&sha256_hex(authority.manifest_raw.as_bytes()))
            );
            let command = CommandKind::PhaseSuccessorStart {
                predecessor_phase_id: predecessor_phase_id.clone(),
                successor_phase_id: phase_id,
                title,
                slug,
                exact_next_work,
                authority,
            };
            Ok((
                checked_id(command_id, derived)?,
                predecessor_phase_id,
                command,
            ))
        }
        PhaseClosureAction::Disposition { .. } => unreachable!("disposition is read-only"),
    }
}

fn close_input(
    state: &RuntimeState,
    phase_id: &str,
    evidence_file: Option<&Path>,
    disposition: PhaseDisposition,
    reason_code: Option<String>,
) -> Result<PhaseCloseInput> {
    let Some(file) = evidence_file else {
        if disposition != PhaseDisposition::Blocked {
            return Err(anyhow!(
                "without --evidence-file only --disposition blocked can be recorded"
            ));
        }
        let bound = phase_closure::disposition_view(state, phase_id)?
            .binding
            .map(|binding| binding.contract_digest)
            .ok_or_else(|| anyhow!("phase {phase_id} is not contract-bound"))?;
        return Ok(PhaseCloseInput {
            run_id: state.run_id.clone(),
            contract_digest: bound,
            evidence_digest: NO_EVIDENCE_DIGEST.into(),
            disposition,
            reason_code,
            scenarios: Vec::new(),
            budget_consumption: BTreeMap::new(),
        });
    };
    let bytes = std::fs::read(file).with_context(|| format!("read evidence {}", file.display()))?;
    let evidence: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("evidence {} is not JSON", file.display()))?;
    let text = |key: &str| -> Result<String> {
        evidence[key]
            .as_str()
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| anyhow!("evidence field {key} is missing"))
    };
    Ok(PhaseCloseInput {
        run_id: text("runId")?,
        contract_digest: text("contractDigest")?,
        evidence_digest: sha256_hex(&bytes),
        disposition,
        reason_code,
        scenarios: scenario_outcomes(&evidence),
        budget_consumption: metered_consumption(&evidence),
    })
}

fn scenario_outcomes(evidence: &Value) -> Vec<ScenarioOutcome> {
    evidence["scenarios"]
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    Some(ScenarioOutcome {
                        id: entry["id"].as_str()?.to_string(),
                        status: entry["status"].as_str().unwrap_or("").to_string(),
                        scenario_digest: entry["scenarioDigest"].as_str().map(str::to_string),
                        carried_from_contract: entry["carriedFrom"]["contractDigest"]
                            .as_str()
                            .map(str::to_string),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Only integer measured/estimate values are cumulative accounting; unknown
/// classes carry no number and are governed by the contract's fallback.
fn metered_consumption(evidence: &Value) -> BTreeMap<String, u64> {
    evidence["budgetConsumption"]["values"]
        .as_object()
        .map(|values| {
            values
                .iter()
                .filter(|(_, v)| {
                    v["class"]
                        .as_str()
                        .is_some_and(|c| METERED_CLASSES.contains(&c))
                })
                .filter_map(|(dimension, v)| {
                    v["value"].as_u64().map(|value| (dimension.clone(), value))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_scenario_digests(pairs: &[String]) -> Result<BTreeMap<String, String>> {
    pairs
        .iter()
        .map(|pair| {
            pair.split_once('=')
                .filter(|(id, digest)| !id.is_empty() && !digest.is_empty())
                .map(|(id, digest)| (id.to_string(), digest.to_string()))
                .ok_or_else(|| anyhow!("--scenario-digest expects ID=SHA256, got {pair}"))
        })
        .collect()
}

/// A manifest file is the canonical JSON plus one trailing newline, exactly
/// the bytes `ssh-keygen -Y sign` signed.
fn read_authority(manifest: &Path, signature: &Path) -> Result<ScopeAuthorityInput> {
    let raw = std::fs::read_to_string(manifest)
        .with_context(|| format!("read scope manifest {}", manifest.display()))?;
    let signature = std::fs::read_to_string(signature)
        .with_context(|| format!("read scope signature {}", signature.display()))?;
    Ok(ScopeAuthorityInput {
        manifest_raw: raw.strip_suffix('\n').unwrap_or(&raw).to_string(),
        signature,
    })
}

fn checked_id(explicit: Option<String>, derived: String) -> Result<String> {
    match explicit {
        Some(id) if id != derived => Err(anyhow!(
            "--command-id {id} does not match the identity-derived command id {derived}"
        )),
        _ => Ok(derived),
    }
}

fn short(digest: &str) -> &str {
    &digest[..digest.len().min(COMMAND_ID_DIGEST_CHARS)]
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn print_outcome(
    state: &RuntimeState,
    phase_id: &str,
    command_id: &str,
    command: &CommandKind,
    committed_revision: u64,
    duplicate: bool,
) -> Result<()> {
    let view = phase_closure::disposition_view(state, phase_id)?;
    let closure_key = match command {
        CommandKind::PhaseClose { closure, .. } => Some(phase_closure::closure_key(
            &state.project_id,
            phase_id,
            closure,
        )?),
        _ => None,
    };
    let successor = view.successor.as_ref().map(|s| s.phase_id.clone());
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "kbd.phase-command/v1",
            "commandId": command_id,
            "phaseId": phase_id,
            "disposition": view.disposition,
            "closureKey": closure_key,
            "contractDigest": view.binding.as_ref().map(|b| b.contract_digest.clone()),
            "bindingRevision": view.binding.as_ref().map(|b| b.binding_revision),
            "successorPhaseId": successor,
            "duplicate": duplicate,
            "committedRevision": committed_revision,
        }))?
    );
    Ok(())
}

fn exit_code(error: &anyhow::Error) -> i32 {
    let runtime = error
        .chain()
        .find_map(|cause| cause.downcast_ref::<RuntimeError>());
    match runtime {
        Some(RuntimeError::PhaseAlreadyClosed { .. }) => EXIT_ALREADY_CLOSED,
        Some(RuntimeError::WorkItemNotFound { .. }) => EXIT_NOT_FOUND,
        Some(
            RuntimeError::PhaseCloseRefused { .. }
            | RuntimeError::ContractBoundPhase { .. }
            | RuntimeError::ScopeAuthority(_),
        ) => EXIT_REFUSED,
        Some(_) => 1,
        None => {
            let message = format!("{error:#}");
            if message.contains("is already closed") {
                EXIT_ALREADY_CLOSED
            } else if [
                "close refused",
                "scope authority refused",
                "is contract-bound",
            ]
            .iter()
            .any(|needle| message.contains(needle))
            {
                EXIT_REFUSED
            } else {
                1
            }
        }
    }
}

fn refuse(error: &anyhow::Error) -> Result<()> {
    let code = exit_code(error);
    let message = format!("{error:#}");
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "kbd.phase-command/v1",
            "refused": true,
            "exitCode": code,
            "error": message,
        }))?
    );
    eprintln!("prometheus kbd phase: {message}");
    std::process::exit(code)
}
