//! Wire compatibility between the embedded mobile replica and sovereign-sync.
//!
//! This test lives with `kbd-mobile` rather than with sovereign-sync's
//! integration suite because it is *this crate's* obligation: `kbd-mobile`
//! encodes a delta that a sovereign-sync peer must be able to decode and
//! verify byte-for-byte. It is a single-process encode/decode assertion, not
//! two-node evidence — no iroh endpoint is started and nothing crosses a
//! network boundary. Moved here from
//! `sovereign-sync/tests/domain_sync.rs` by change-cpc-002 so it travels with
//! the crate when kbd-mobile relocates to the Companion.

use kbd_mobile::{MobilePeer, MobileProject};
use kbd_runtime::{Actor, DeviceSigner, Runtime};
use sovereign_sync::domains::SyncEnvelope;
use sovereign_sync::kbd_sync::KbdAuthorityPayload;
use sovereign_sync::p2p::P2PNode;
use uuid::Uuid;

#[test]
fn mobile_wire_is_byte_compatible_with_sovereign_sync() {
    let fixture = tempfile::tempdir().unwrap();
    let project_id = Uuid::new_v4().to_string();
    let runtime = Runtime::open(fixture.path());
    runtime
        .initialize(
            &project_id,
            "mobile-wire-run",
            Actor::operator("operator", "mobile-wire-test"),
        )
        .unwrap();
    let signer: DeviceSigner = runtime.device_signer().unwrap();
    let mobile =
        MobileProject::from_events(&project_id, "mobile-replica", &runtime.events().unwrap())
            .unwrap();
    let mut prepared = mobile.prepare_signed_delta(signer.key_id()).unwrap();
    assert_eq!(
        prepared.delta.signable_bytes_for_host(),
        prepared.signing_payload
    );
    let signature = signer.sign_base64(&prepared.signing_payload);
    prepared
        .delta
        .attach_host_signature(signer.public_key(), signature)
        .unwrap();

    let wire = prepared.delta.encode().unwrap();
    let daemon_envelope: SyncEnvelope = serde_json::from_slice(&wire).unwrap();
    assert_eq!(daemon_envelope.signable_bytes(), prepared.signing_payload);
    assert!(daemon_envelope.verify(signer.public_key()));
    let authority = KbdAuthorityPayload::decode(&daemon_envelope.payload).unwrap();
    assert_eq!(authority.project_id, project_id);
    assert_eq!(authority.project_updates, mobile.export_updates().unwrap());

    let group_secret = [73; 32];
    assert_eq!(
        MobilePeer::derive_topic(&group_secret),
        P2PNode::derive_topic(&group_secret)
    );
}
