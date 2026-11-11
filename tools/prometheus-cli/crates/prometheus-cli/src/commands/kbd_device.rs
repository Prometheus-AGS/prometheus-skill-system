//! `prometheus kbd device identity | list | enroll | grant-operator | revoke`.
//!
//! Lets a project's operator admit a teammate's machine to the signed journal. A teammate runs
//! `device identity` and sends the printed public key; an operator enrolls it, optionally as a
//! co-operator (who may then pause, resume, start or cancel runs, resolve conflicts, and enroll others). Every change is an
//! operator-signed journal event, visible in `prometheus kbd audit` and undone with `revoke`.

use super::{state_or_initialize, state_or_replay, submit_typed, ControlClient};
use anyhow::{Context, Result};
use clap::Subcommand;
use kbd_runtime::{CommandKind, DeviceRecord, DeviceStatus, Runtime, RuntimeState};
use serde_json::json;

#[derive(Subcommand)]
pub enum DeviceAction {
    /// Print this machine's signing key for the project, to send to an operator
    Identity {
        #[arg(long)]
        json: bool,
    },
    /// List enrolled devices and which hold operator authority
    List {
        #[arg(long)]
        json: bool,
    },
    /// Enroll a teammate's device from the public key printed by `device identity`
    Enroll {
        #[arg(long)]
        command_id: String,
        #[arg(long)]
        public_key: String,
        #[arg(long)]
        device_id: String,
        /// Also grant co-operator authority (run lifecycle, conflict resolution, enrolling others)
        #[arg(long)]
        operator: bool,
    },
    /// Grant co-operator authority to an already-enrolled device
    GrantOperator {
        #[arg(long)]
        command_id: String,
        #[arg(long)]
        key_id: String,
    },
    /// Revoke a device; it can no longer sign, and loses any operator authority
    Revoke {
        #[arg(long)]
        command_id: String,
        #[arg(long)]
        key_id: String,
        #[arg(long)]
        reason: String,
    },
}

pub async fn run(runtime: &Runtime, client: &ControlClient, action: DeviceAction) -> Result<()> {
    match action {
        DeviceAction::Identity { json } => identity(runtime, json),
        DeviceAction::List { json } => list(&state_or_replay(client, runtime).await?, json),
        DeviceAction::Enroll {
            command_id,
            public_key,
            device_id,
            operator,
        } => {
            let state = state_or_initialize(client, runtime).await?;
            let device = DeviceRecord::enrollable(&device_id, &public_key, state.revision + 1)?;
            let key_id = device.key_id.clone();
            let state = submit(client, &state, command_id.clone(), CommandKind::DeviceEnroll { device }).await?;
            if operator {
                submit(client, &state, format!("{command_id}:operator"), CommandKind::OperatorGrant { key_id: key_id.clone() }).await?;
            }
            println!(
                "ENROLLED key={key_id} device={device_id} operator={operator}"
            );
            Ok(())
        }
        DeviceAction::GrantOperator { command_id, key_id } => {
            let state = state_or_initialize(client, runtime).await?;
            submit(client, &state, command_id, CommandKind::OperatorGrant { key_id: key_id.clone() }).await?;
            println!("OPERATOR-GRANTED key={key_id}");
            Ok(())
        }
        DeviceAction::Revoke {
            command_id,
            key_id,
            reason,
        } => {
            let state = state_or_initialize(client, runtime).await?;
            submit(client, &state, command_id, CommandKind::DeviceRevoke { key_id: key_id.clone(), reason }).await?;
            println!("REVOKED key={key_id}");
            Ok(())
        }
    }
}

async fn submit(
    client: &ControlClient,
    state: &RuntimeState,
    command_id: String,
    command: CommandKind,
) -> Result<RuntimeState> {
    let result = submit_typed(client, state, command_id, command).await?;
    serde_json::from_value(result["state"].clone()).context("parse committed state")
}

fn identity(runtime: &Runtime, json: bool) -> Result<()> {
    let identity = runtime.local_device_identity()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&identity)?);
        return Ok(());
    }
    println!("Device: {}", identity.device_id);
    println!("Key: {}", identity.key_id);
    println!("Public key: {}", identity.public_key);
    println!(
        "Send the public key to a project operator, who runs:\n  prometheus kbd device enroll --command-id <id> --device-id {} --public-key {} [--operator]",
        identity.device_id, identity.public_key
    );
    Ok(())
}

fn list(state: &RuntimeState, json: bool) -> Result<()> {
    let rows: Vec<_> = state
        .devices
        .values()
        .map(|device| {
            json!({
                "deviceId": device.device_id,
                "keyId": device.key_id,
                "status": if device.status == DeviceStatus::Active { "active" } else { "revoked" },
                "operator": state.operator_key_ids.contains(&device.key_id),
            })
        })
        .collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    for row in rows {
        println!(
            "{}  {}  {}{}",
            row["keyId"].as_str().unwrap_or_default(),
            row["deviceId"].as_str().unwrap_or_default(),
            row["status"].as_str().unwrap_or_default(),
            if row["operator"] == true { "  operator" } else { "" }
        );
    }
    Ok(())
}
