use super::{
    validate_device_record, Actor, ActorKind, DeviceRecord, DeviceStatus, Event, EventKind,
    KbdStateV2, Result, RuntimeError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignerRequirement {
    Bootstrap,
    ActiveDevice,
    Operator,
}

impl SignerRequirement {
    pub(crate) fn for_actor(state: &KbdStateV2, actor: &Actor) -> Self {
        if state.revision == 0 {
            Self::Bootstrap
        } else if actor.kind == ActorKind::Operator {
            Self::Operator
        } else {
            Self::ActiveDevice
        }
    }

    pub(crate) fn accepts(self, state: &KbdStateV2, key_id: &str) -> bool {
        match self {
            Self::Bootstrap => true,
            Self::ActiveDevice => state
                .devices
                .get(key_id)
                .is_some_and(|device| device.status == DeviceStatus::Active),
            Self::Operator => {
                state.operator_key_ids.contains(key_id)
                    && state
                        .devices
                        .get(key_id)
                        .is_some_and(|device| device.status == DeviceStatus::Active)
            }
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Bootstrap => "bootstrap",
            Self::ActiveDevice => "active enrolled device",
            Self::Operator => "active enrolled operator",
        }
    }
}

pub(crate) fn verify_and_apply(state: &mut KbdStateV2, event: &Event) -> Result<()> {
    let bootstrap = state.revision == 0;
    if bootstrap
        && (!matches!(event.kind, EventKind::RunInitialized { .. })
            || event.actor.kind != ActorKind::Operator)
    {
        return Err(RuntimeError::InvalidState(
            "the first signed event must initialize the run under operator authority".into(),
        ));
    }
    event.verify_signature(&state.devices, bootstrap)?;
    if event.integrity_hash != event.calculate_hash()? {
        return Err(RuntimeError::Integrity {
            revision: event.revision,
        });
    }

    if event.schema_version != "1" && !bootstrap && event.actor.kind == ActorKind::Operator {
        let signer_key_id =
            event
                .signer_key_id
                .as_deref()
                .ok_or_else(|| RuntimeError::Signature {
                    revision: event.revision,
                    reason: "operator event is missing signerKeyId".into(),
                })?;
        if !SignerRequirement::Operator.accepts(state, signer_key_id) {
            return Err(RuntimeError::InvalidState(
                "operator event requires an active operator signing key".into(),
            ));
        }
    }

    if event.schema_version != "1" && bootstrap {
        let key_id = event
            .signer_key_id
            .clone()
            .ok_or_else(|| RuntimeError::Signature {
                revision: event.revision,
                reason: "missing bootstrap signerKeyId".into(),
            })?;
        let public_key =
            event
                .signer_public_key
                .clone()
                .ok_or_else(|| RuntimeError::Signature {
                    revision: event.revision,
                    reason: "missing bootstrap signerPublicKey".into(),
                })?;
        state.devices.insert(
            key_id.clone(),
            DeviceRecord {
                device_id: event.actor.device.clone(),
                key_id: key_id.clone(),
                public_key,
                status: DeviceStatus::Active,
                enrolled_at_revision: event.revision,
                revoked_at_revision: None,
            },
        );
        state.operator_key_ids.insert(key_id);
    }

    match &event.kind {
        EventKind::DeviceEnrolled { device } => {
            require_operator(event, "enroll")?;
            if state.devices.contains_key(&device.key_id) {
                return Err(RuntimeError::WorkItemExists {
                    kind: "device key",
                    id: device.key_id.clone(),
                });
            }
            validate_device_record(device, event.revision)?;
            state.devices.insert(device.key_id.clone(), device.clone());
        }
        EventKind::DeviceRevoked { key_id, .. } => {
            require_operator(event, "revoke")?;
            let device =
                state
                    .devices
                    .get_mut(key_id)
                    .ok_or_else(|| RuntimeError::WorkItemNotFound {
                        kind: "device key",
                        id: key_id.clone(),
                    })?;
            device.status = DeviceStatus::Revoked;
            device.revoked_at_revision = Some(event.revision);
            state.operator_key_ids.remove(key_id);
        }
        EventKind::DeviceKeyRotated {
            previous_key_id,
            replacement,
        } => {
            require_operator(event, "rotate a device key")?;
            validate_device_record(replacement, event.revision)?;
            let previous = state.devices.get_mut(previous_key_id).ok_or_else(|| {
                RuntimeError::WorkItemNotFound {
                    kind: "device key",
                    id: previous_key_id.clone(),
                }
            })?;
            previous.status = DeviceStatus::Revoked;
            previous.revoked_at_revision = Some(event.revision);
            state
                .devices
                .insert(replacement.key_id.clone(), replacement.clone());
            if state.operator_key_ids.remove(previous_key_id) {
                state.operator_key_ids.insert(replacement.key_id.clone());
            }
        }
        _ => {}
    }
    Ok(())
}

fn require_operator(event: &Event, action: &str) -> Result<()> {
    if event.actor.kind != ActorKind::Operator {
        return Err(RuntimeError::InvalidState(format!(
            "only an operator may {action}"
        )));
    }
    Ok(())
}
