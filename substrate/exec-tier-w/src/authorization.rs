use std::collections::HashSet;

#[cfg(feature = "estate")]
use std::{
    fs,
    path::{Component as PathComponent, Path, PathBuf},
};

#[cfg(feature = "estate")]
use base64::{engine::general_purpose::STANDARD, Engine as _};
#[cfg(feature = "estate")]
use ed25519_dalek::VerifyingKey;
use prometheus_exec_contracts::{ComponentAuthorization, ComponentAuthorizationMode, Digest};
use serde::Serialize;
#[cfg(feature = "estate")]
use serde_json::{Map, Value};

use crate::{TierWError, COMPONENT_WORLD};

#[cfg(feature = "estate")]
const SIGNATURE_NAMESPACE: &str = "prometheus-plugin-generation-v1";
#[cfg(feature = "estate")]
const MAX_MANIFEST_BYTES: u64 = 32 * 1024 * 1024;
#[cfg(feature = "estate")]
const MAX_SIGNATURE_BYTES: u64 = 16 * 1024;
#[cfg(feature = "estate")]
const MAX_TRUST_STORE_BYTES: u64 = 1024 * 1024;
#[cfg(feature = "estate")]
const MAX_COMPONENT_BYTES: u64 = 512 * 1024 * 1024;
#[cfg(feature = "estate")]
const ED25519_SPKI_PREFIX: &[u8] = &[
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

#[derive(Clone, Debug)]
pub enum ComponentTrustPolicy {
    ExactPins {
        mode: ComponentAuthorizationMode,
        digests: HashSet<Digest>,
    },
    #[cfg(feature = "estate")]
    Estate { plugin_root: PathBuf },
}

#[derive(Clone, Debug)]
pub struct ComponentAuthorizer {
    policy: ComponentTrustPolicy,
    verified_receipt: Option<VerifiedReceiptAuthorization>,
}

#[derive(Clone, Debug)]
struct VerifiedReceiptAuthorization {
    component_hash: Digest,
    authorization: ComponentAuthorization,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentTrustInspection {
    pub mode: ComponentAuthorizationMode,
    pub component_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_hash: Option<Digest>,
}

#[cfg(feature = "estate")]
struct VerifiedGeneration {
    root: PathBuf,
    manifest: Value,
    generation_id: String,
    manifest_hash: Digest,
}

pub(crate) struct AuthorizedComponent<'a> {
    bytes: &'a [u8],
    component_hash: Digest,
    authorization: ComponentAuthorization,
}

impl ComponentAuthorizer {
    pub fn hash_pins(digests: impl IntoIterator<Item = Digest>) -> Self {
        Self::exact_pins(ComponentAuthorizationMode::HashPin, digests)
    }

    pub fn bundled(digests: impl IntoIterator<Item = Digest>) -> Self {
        Self::exact_pins(ComponentAuthorizationMode::Bundled, digests)
    }

    #[cfg(feature = "estate")]
    pub fn estate(plugin_root: impl Into<PathBuf>) -> Self {
        Self {
            policy: ComponentTrustPolicy::Estate {
                plugin_root: plugin_root.into(),
            },
            verified_receipt: None,
        }
    }

    pub(crate) fn verified_receipt(
        component_hash: Digest,
        authorization: ComponentAuthorization,
    ) -> Self {
        Self {
            policy: ComponentTrustPolicy::ExactPins {
                mode: ComponentAuthorizationMode::HashPin,
                digests: HashSet::from([component_hash.clone()]),
            },
            verified_receipt: Some(VerifiedReceiptAuthorization {
                component_hash,
                authorization,
            }),
        }
    }

    /// Resolves authorization without compiling, linking, executing, or
    /// mutating the active plugin generation.
    pub fn authorization_for_bytes(
        &self,
        component_bytes: &[u8],
    ) -> Result<ComponentAuthorization, TierWError> {
        let component_hash = Digest::from_bytes(component_bytes);
        self.authorization_for(&component_hash, Some(component_bytes.len()))
    }

    pub fn from_policy(policy: ComponentTrustPolicy) -> Self {
        Self {
            policy,
            verified_receipt: None,
        }
    }

    pub fn policy(&self) -> &ComponentTrustPolicy {
        &self.policy
    }

    /// Verifies the configured trust roots without compiling or executing a
    /// component and without changing the active generation or trust store.
    pub fn inspect(&self) -> Result<ComponentTrustInspection, TierWError> {
        if let Some(verified) = &self.verified_receipt {
            verified
                .authorization
                .validate()
                .map_err(|error| unauthorized(error.to_string()))?;
            return Ok(ComponentTrustInspection {
                mode: verified.authorization.mode,
                component_count: 1,
                generation_id: verified.authorization.generation_id.clone(),
                manifest_hash: verified.authorization.manifest_hash.clone(),
            });
        }
        match &self.policy {
            ComponentTrustPolicy::ExactPins { mode, digests } => {
                if digests.is_empty() {
                    return Err(unauthorized("component trust has no exact pins"));
                }
                Ok(ComponentTrustInspection {
                    mode: *mode,
                    component_count: digests.len(),
                    generation_id: None,
                    manifest_hash: None,
                })
            }
            #[cfg(feature = "estate")]
            ComponentTrustPolicy::Estate { plugin_root } => {
                let generation = verify_generation(plugin_root)?;
                let component_count = verify_generation_components(&generation)?;
                if component_count == 0 {
                    return Err(unauthorized(
                        "active signed generation contains no Wasm components",
                    ));
                }
                Ok(ComponentTrustInspection {
                    mode: ComponentAuthorizationMode::SignedGeneration,
                    component_count,
                    generation_id: Some(generation.generation_id),
                    manifest_hash: Some(generation.manifest_hash),
                })
            }
        }
    }

    pub(crate) fn authorize<'a>(
        &self,
        component_bytes: &'a [u8],
    ) -> Result<AuthorizedComponent<'a>, TierWError> {
        let component_hash = Digest::from_bytes(component_bytes);
        let authorization = self.authorization_for(&component_hash, Some(component_bytes.len()))?;
        Ok(AuthorizedComponent {
            bytes: component_bytes,
            component_hash,
            authorization,
        })
    }

    pub(crate) fn reauthorize(
        &self,
        component_hash: &Digest,
        authorization: &ComponentAuthorization,
    ) -> Result<(), TierWError> {
        authorization
            .validate()
            .map_err(|error| unauthorized(error.to_string()))?;
        let current = self.authorization_for(component_hash, None)?;
        if &current != authorization {
            return Err(unauthorized(
                "component authorization no longer matches the active trust state",
            ));
        }
        Ok(())
    }

    fn exact_pins(
        mode: ComponentAuthorizationMode,
        digests: impl IntoIterator<Item = Digest>,
    ) -> Self {
        debug_assert!(matches!(
            mode,
            ComponentAuthorizationMode::HashPin | ComponentAuthorizationMode::Bundled
        ));
        Self {
            policy: ComponentTrustPolicy::ExactPins {
                mode,
                digests: digests.into_iter().collect(),
            },
            verified_receipt: None,
        }
    }

    fn authorization_for(
        &self,
        component_hash: &Digest,
        component_size: Option<usize>,
    ) -> Result<ComponentAuthorization, TierWError> {
        #[cfg(not(feature = "estate"))]
        let _ = component_size;
        if let Some(verified) = &self.verified_receipt {
            verified
                .authorization
                .validate()
                .map_err(|error| unauthorized(error.to_string()))?;
            if &verified.component_hash != component_hash {
                return Err(unauthorized(format!(
                    "component {} differs from verified receipt component {}",
                    component_hash, verified.component_hash
                )));
            }
            return Ok(verified.authorization.clone());
        }
        match &self.policy {
            ComponentTrustPolicy::ExactPins { mode, digests } => {
                if !matches!(
                    mode,
                    ComponentAuthorizationMode::HashPin | ComponentAuthorizationMode::Bundled
                ) {
                    return Err(unauthorized(
                        "exact component pins require hash-pin or bundled mode",
                    ));
                }
                if !digests.contains(component_hash) {
                    return Err(unauthorized(format!(
                        "component {} is absent from the configured exact pins",
                        component_hash
                    )));
                }
                Ok(ComponentAuthorization {
                    mode: *mode,
                    world: COMPONENT_WORLD.into(),
                    manifest_hash: None,
                    generation_id: None,
                })
            }
            #[cfg(feature = "estate")]
            ComponentTrustPolicy::Estate { plugin_root } => {
                verify_active_generation(plugin_root, component_hash, component_size)
            }
        }
    }
}

impl<'a> AuthorizedComponent<'a> {
    pub(crate) fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    pub(crate) fn component_hash(&self) -> &Digest {
        &self.component_hash
    }

    pub(crate) fn authorization(&self) -> &ComponentAuthorization {
        &self.authorization
    }
}

#[cfg(feature = "estate")]
fn verify_active_generation(
    plugin_root: &Path,
    component_hash: &Digest,
    component_size: Option<usize>,
) -> Result<ComponentAuthorization, TierWError> {
    let generation = verify_generation(plugin_root)?;
    verify_component_entry(
        &generation.root,
        &generation.manifest,
        component_hash,
        component_size,
    )?;

    Ok(ComponentAuthorization {
        mode: ComponentAuthorizationMode::SignedGeneration,
        world: COMPONENT_WORLD.into(),
        manifest_hash: Some(generation.manifest_hash),
        generation_id: Some(generation.generation_id),
    })
}

#[cfg(feature = "estate")]
fn verify_generation(plugin_root: &Path) -> Result<VerifiedGeneration, TierWError> {
    let plugin_root = plugin_root
        .canonicalize()
        .map_err(|error| unauthorized(format!("plugin root is unavailable: {error}")))?;
    let target = fs::read_link(plugin_root.join("current"))
        .map_err(|error| unauthorized(format!("active generation pointer is invalid: {error}")))?;
    let generation_id = generation_from_pointer(&target)?;
    let generation_root = plugin_root.join("generations").join(&generation_id);
    let canonical_generation = generation_root
        .canonicalize()
        .map_err(|error| unauthorized(format!("active generation is unavailable: {error}")))?;
    let canonical_generations = plugin_root
        .join("generations")
        .canonicalize()
        .map_err(|error| unauthorized(format!("generation store is unavailable: {error}")))?;
    if !canonical_generation.starts_with(&canonical_generations) {
        return Err(unauthorized("active generation escapes the plugin store"));
    }

    let manifest_bytes = read_regular_file(
        &canonical_generation.join("manifest.json"),
        MAX_MANIFEST_BYTES,
        "generation manifest",
        false,
    )?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| unauthorized(format!("generation manifest is invalid JSON: {error}")))?;
    let canonical_manifest = canonical_pretty_json(&manifest)?;
    verify_generation_identity(&manifest, &generation_id)?;
    let signer_key_id = required_string(&manifest, "signerKeyId")?;

    let signature_bytes = read_regular_file(
        &canonical_generation.join("manifest.sig.json"),
        MAX_SIGNATURE_BYTES,
        "generation signature",
        false,
    )?;
    let signature: Value = serde_json::from_slice(&signature_bytes)
        .map_err(|error| unauthorized(format!("generation signature is invalid JSON: {error}")))?;
    verify_signature_envelope(&plugin_root, &signature, signer_key_id, &canonical_manifest)?;
    Ok(VerifiedGeneration {
        root: canonical_generation,
        manifest,
        generation_id,
        manifest_hash: Digest::from_bytes(&canonical_manifest),
    })
}

#[cfg(feature = "estate")]
fn verify_generation_components(generation: &VerifiedGeneration) -> Result<usize, TierWError> {
    let files = generation
        .manifest
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| unauthorized("generation manifest has no files array"))?;
    let mut component_count = 0usize;
    let mut observed = HashSet::new();
    for entry in files {
        let Some(path) = entry.get("path").and_then(Value::as_str) else {
            continue;
        };
        if !path.ends_with(".wasm") {
            continue;
        }
        let raw_hash = required_string(entry, "sha256")?;
        let component_hash = Digest::parse(format!("sha256:{raw_hash}"))
            .map_err(|error| unauthorized(format!("component digest is invalid: {error}")))?;
        if !observed.insert(component_hash.clone()) {
            return Err(unauthorized(
                "component digest is duplicated in the active generation manifest",
            ));
        }
        let size = entry
            .get("size")
            .and_then(Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| unauthorized("component manifest entry has no valid byte size"))?;
        verify_component_entry(
            &generation.root,
            &generation.manifest,
            &component_hash,
            Some(size),
        )?;
        component_count += 1;
    }
    Ok(component_count)
}

#[cfg(feature = "estate")]
fn generation_from_pointer(target: &Path) -> Result<String, TierWError> {
    if target.is_absolute() {
        return Err(unauthorized("active generation pointer must be relative"));
    }
    let components: Vec<_> = target.components().collect();
    let [PathComponent::Normal(parent), PathComponent::Normal(generation)] = components.as_slice()
    else {
        return Err(unauthorized(
            "active generation pointer must target generations/<sha256>",
        ));
    };
    if parent != &std::ffi::OsStr::new("generations") {
        return Err(unauthorized(
            "active generation pointer must target the generations directory",
        ));
    }
    let generation = generation
        .to_str()
        .ok_or_else(|| unauthorized("generation identity is not UTF-8"))?;
    if generation.len() != 64
        || !generation
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(unauthorized("generation identity is not a SHA-256 digest"));
    }
    Ok(generation.into())
}

#[cfg(feature = "estate")]
fn verify_generation_identity(manifest: &Value, generation_id: &str) -> Result<(), TierWError> {
    let declared = required_string(manifest, "generation")?;
    if declared != generation_id {
        return Err(unauthorized(
            "active pointer and manifest generation identities differ",
        ));
    }
    let object = manifest
        .as_object()
        .ok_or_else(|| unauthorized("generation manifest must be an object"))?;
    let mut identity = Map::new();
    for key in [
        "schemaVersion",
        "sourceVersion",
        "signerKeyId",
        "bundleId",
        "hookRuntime",
        "sourceProvenance",
        "skillIndex",
        "files",
        "targetPayloads",
    ] {
        let value = object
            .get(key)
            .ok_or_else(|| unauthorized(format!("generation manifest omits {key}")))?;
        identity.insert(key.into(), value.clone());
    }
    let computed = raw_sha256(&canonical_pretty_json(&Value::Object(identity))?);
    if computed != generation_id {
        return Err(unauthorized("generation identity hash is invalid"));
    }
    Ok(())
}

#[cfg(feature = "estate")]
fn verify_signature_envelope(
    plugin_root: &Path,
    envelope: &Value,
    manifest_signer: &str,
    canonical_manifest: &[u8],
) -> Result<(), TierWError> {
    if envelope.get("schemaVersion").and_then(Value::as_u64) != Some(1)
        || envelope.get("namespace").and_then(Value::as_str) != Some(SIGNATURE_NAMESPACE)
        || envelope.get("algorithm").and_then(Value::as_str) != Some("Ed25519")
    {
        return Err(unauthorized("generation signature envelope is invalid"));
    }
    let envelope_signer = required_string(envelope, "signerKeyId")?;
    if envelope_signer != manifest_signer {
        return Err(unauthorized(
            "manifest signer does not match the signature envelope",
        ));
    }
    let trust_bytes = read_regular_file(
        &plugin_root.join("trust/allowed-signers.json"),
        MAX_TRUST_STORE_BYTES,
        "plugin trust store",
        true,
    )?;
    let trust: Value = serde_json::from_slice(&trust_bytes)
        .map_err(|error| unauthorized(format!("plugin trust store is invalid JSON: {error}")))?;
    if trust.get("schemaVersion").and_then(Value::as_u64) != Some(1) {
        return Err(unauthorized("plugin trust-store schema is unsupported"));
    }
    let signers = trust
        .get("signers")
        .and_then(Value::as_array)
        .ok_or_else(|| unauthorized("plugin trust store has no signers array"))?;
    let matches: Vec<_> = signers
        .iter()
        .filter(|entry| entry.get("keyId").and_then(Value::as_str) == Some(manifest_signer))
        .collect();
    if matches.len() != 1 {
        return Err(unauthorized(
            "plugin signer is missing or duplicated in the trust store",
        ));
    }
    let signer = matches[0];
    if signer.get("algorithm").and_then(Value::as_str) != Some("Ed25519") {
        return Err(unauthorized("plugin signer algorithm is unsupported"));
    }
    let public_key = parse_ed25519_spki(required_string(signer, "publicKey")?)?;
    let key_id = raw_sha256(&public_key.0);
    if key_id != manifest_signer {
        return Err(unauthorized("plugin trust-store key fingerprint mismatch"));
    }
    let signature = STANDARD
        .decode(required_string(envelope, "signature")?)
        .map_err(|error| unauthorized(format!("plugin signature is not base64: {error}")))?;
    let signature = ed25519_dalek::Signature::from_slice(&signature)
        .map_err(|error| unauthorized(format!("plugin signature has an invalid size: {error}")))?;
    let mut payload = Vec::with_capacity(
        SIGNATURE_NAMESPACE.len() + 1usize.saturating_add(canonical_manifest.len()),
    );
    payload.extend_from_slice(SIGNATURE_NAMESPACE.as_bytes());
    payload.push(b'\n');
    payload.extend_from_slice(canonical_manifest);
    public_key
        .1
        .verify_strict(&payload, &signature)
        .map_err(|_| unauthorized("plugin generation signature verification failed"))
}

#[cfg(feature = "estate")]
fn verify_component_entry(
    generation_root: &Path,
    manifest: &Value,
    component_hash: &Digest,
    component_size: Option<usize>,
) -> Result<(), TierWError> {
    let raw_hash = component_hash
        .as_str()
        .strip_prefix("sha256:")
        .expect("Digest always carries a sha256 prefix");
    let files = manifest
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| unauthorized("generation manifest has no files array"))?;
    let mut matches = files.iter().filter(|entry| {
        entry.get("sha256").and_then(Value::as_str) == Some(raw_hash)
            && entry
                .get("path")
                .and_then(Value::as_str)
                .is_some_and(|path| path.ends_with(".wasm"))
    });
    let Some(entry) = matches.next() else {
        return Err(unauthorized(format!(
            "component {component_hash} is absent from the active signed generation"
        )));
    };
    if matches.next().is_some() {
        return Err(unauthorized(
            "component digest is duplicated in the active generation manifest",
        ));
    }
    if entry.get("type").and_then(Value::as_str) != Some("file") {
        return Err(unauthorized(
            "component manifest entry is not a regular file",
        ));
    }
    let path = required_string(entry, "path")?;
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed
            .components()
            .any(|component| !matches!(component, PathComponent::Normal(_)))
    {
        return Err(unauthorized("component manifest path is unsafe"));
    }
    if let Some(component_size) = component_size {
        let declared_size = entry
            .get("size")
            .and_then(Value::as_u64)
            .ok_or_else(|| unauthorized("component manifest entry has no byte size"))?;
        if declared_size != component_size as u64 {
            return Err(unauthorized(
                "component byte size differs from the signed manifest entry",
            ));
        }
    }
    let installed_bytes = read_regular_file(
        &generation_root.join(parsed),
        MAX_COMPONENT_BYTES,
        "generation component",
        false,
    )?;
    if Digest::from_bytes(&installed_bytes) != *component_hash {
        return Err(unauthorized(
            "installed component bytes differ from the signed manifest entry",
        ));
    }
    Ok(())
}

#[cfg(feature = "estate")]
fn parse_ed25519_spki(pem: &str) -> Result<(Vec<u8>, VerifyingKey), TierWError> {
    let body = pem
        .strip_prefix("-----BEGIN PUBLIC KEY-----")
        .and_then(|value| value.strip_suffix("-----END PUBLIC KEY-----\n"))
        .or_else(|| {
            pem.strip_prefix("-----BEGIN PUBLIC KEY-----")
                .and_then(|value| value.strip_suffix("-----END PUBLIC KEY-----"))
        })
        .ok_or_else(|| unauthorized("plugin signer public key is not SPKI PEM"))?;
    let encoded: String = body
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let der = STANDARD
        .decode(encoded)
        .map_err(|error| unauthorized(format!("plugin signer PEM is invalid: {error}")))?;
    let key_bytes = der
        .strip_prefix(ED25519_SPKI_PREFIX)
        .ok_or_else(|| unauthorized("plugin signer SPKI algorithm is not Ed25519"))?;
    let key_bytes: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| unauthorized("plugin signer SPKI key has an invalid length"))?;
    let key = VerifyingKey::from_bytes(&key_bytes)
        .map_err(|error| unauthorized(format!("plugin signer key is invalid: {error}")))?;
    Ok((der, key))
}

#[cfg(feature = "estate")]
fn read_regular_file(
    path: &Path,
    max_bytes: u64,
    label: &str,
    private: bool,
) -> Result<Vec<u8>, TierWError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| unauthorized(format!("{label} is unavailable: {error}")))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(unauthorized(format!("{label} is not a regular file")));
    }
    if metadata.len() > max_bytes {
        return Err(unauthorized(format!("{label} exceeds its size limit")));
    }
    #[cfg(unix)]
    if private {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o777 != 0o600 {
            return Err(unauthorized(format!("{label} must have mode 0600")));
        }
    }
    fs::read(path).map_err(|error| unauthorized(format!("failed to read {label}: {error}")))
}

#[cfg(feature = "estate")]
fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, TierWError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| unauthorized(format!("signed plugin value omits {field}")))
}

#[cfg(feature = "estate")]
fn canonical_pretty_json(value: &Value) -> Result<Vec<u8>, TierWError> {
    let sorted = sort_json(value);
    let mut bytes = serde_json::to_vec_pretty(&sorted)
        .map_err(|error| unauthorized(format!("plugin value cannot be canonicalized: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(feature = "estate")]
fn sort_json(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(sort_json).collect()),
        Value::Object(values) => {
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort_unstable();
            let mut sorted = Map::new();
            for key in keys {
                sorted.insert(key.clone(), sort_json(&values[key]));
            }
            Value::Object(sorted)
        }
        scalar => scalar.clone(),
    }
}

#[cfg(feature = "estate")]
fn raw_sha256(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let digest = sha2::Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unauthorized(message: impl Into<String>) -> TierWError {
    TierWError::ComponentUnauthorized(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "estate")]
    use base64::engine::general_purpose::STANDARD;
    #[cfg(feature = "estate")]
    use ed25519_dalek::{Signer as _, SigningKey};
    #[cfg(feature = "estate")]
    use serde_json::json;

    #[cfg(feature = "cranelift")]
    use crate::{EngineProfile, TierWEngine};
    #[cfg(all(feature = "cranelift", feature = "estate"))]
    use crate::{TierWExecutionOutcome, TierWLimits};

    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );

    #[cfg(feature = "cranelift")]
    #[test]
    fn exact_pins_authorize_before_validation_and_bind_cache_identity() {
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let reference_hash = Digest::from_bytes(REFERENCE_COMPONENT);
        let hash_pin = ComponentAuthorizer::hash_pins([reference_hash.clone()]);
        let bundled = ComponentAuthorizer::bundled([reference_hash]);

        let unauthorized_bytes = b"not a component";
        let error = engine
            .load_component(unauthorized_bytes, &hash_pin)
            .unwrap_err();
        assert!(matches!(error, TierWError::ComponentUnauthorized(_)));

        let pinned = engine
            .load_component(REFERENCE_COMPONENT, &hash_pin)
            .unwrap();
        let bundled_component = engine
            .load_component(REFERENCE_COMPONENT, &bundled)
            .unwrap();
        assert_eq!(
            pinned.authorization().mode,
            ComponentAuthorizationMode::HashPin
        );
        assert_eq!(
            bundled_component.authorization().mode,
            ComponentAuthorizationMode::Bundled
        );
        assert_ne!(pinned.cache_key(), bundled_component.cache_key());
    }

    #[cfg(all(unix, feature = "cranelift", feature = "estate"))]
    #[test]
    fn signed_generation_rollback_reauthorizes_cached_components() {
        let fixture = tempfile::tempdir().unwrap();
        let plugin_root = fixture.path().join("plugin");
        fs::create_dir_all(plugin_root.join("generations")).unwrap();
        let signing_key = SigningKey::from_bytes(&[41; 32]);
        write_trust_store(&plugin_root, &signing_key);

        let old_bytes = REFERENCE_COMPONENT.to_vec();
        let new_bytes = component_with_custom_section(REFERENCE_COMPONENT, b'n');
        let old_generation = write_generation(&plugin_root, &signing_key, &old_bytes, "old");
        let new_generation = write_generation(&plugin_root, &signing_key, &new_bytes, "new");
        activate(&plugin_root, &new_generation);

        let authorizer = ComponentAuthorizer::estate(&plugin_root);
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();
        let cached_new = engine.load_component(&new_bytes, &authorizer).unwrap();
        assert_eq!(
            cached_new.authorization().generation_id.as_deref(),
            Some(new_generation.as_str())
        );
        let old_error = engine.load_component(&old_bytes, &authorizer).unwrap_err();
        assert!(matches!(old_error, TierWError::ComponentUnauthorized(_)));

        activate(&plugin_root, &old_generation);
        let stale = engine.execute_component(
            &cached_new,
            &authorizer,
            crate::CapabilityGrant::empty(),
            &TierWLimits::default(),
            "{}",
        );
        assert!(matches!(
            stale,
            TierWExecutionOutcome::Failed(ref failure)
                if failure.failure.kind
                    == prometheus_exec_contracts::ExecutionFailureKind::ComponentUnauthorized
        ));
        let active_old = engine.load_component(&old_bytes, &authorizer).unwrap();
        assert_eq!(
            active_old.authorization().generation_id.as_deref(),
            Some(old_generation.as_str())
        );
        assert_eq!(
            cached_new.authorization().generation_id.as_deref(),
            Some(new_generation.as_str())
        );
    }

    #[cfg(all(unix, feature = "estate"))]
    #[test]
    fn trust_inspection_verifies_active_component_bytes_without_mutation() {
        let fixture = tempfile::tempdir().unwrap();
        let plugin_root = fixture.path().join("plugin");
        fs::create_dir_all(plugin_root.join("generations")).unwrap();
        let signing_key = SigningKey::from_bytes(&[67; 32]);
        write_trust_store(&plugin_root, &signing_key);
        let generation =
            write_generation(&plugin_root, &signing_key, REFERENCE_COMPONENT, "inspect");
        activate(&plugin_root, &generation);
        let authorizer = ComponentAuthorizer::estate(&plugin_root);

        let before = tree_snapshot(&plugin_root);
        let inspection = authorizer.inspect().unwrap();
        let after = tree_snapshot(&plugin_root);

        assert_eq!(
            inspection.mode,
            ComponentAuthorizationMode::SignedGeneration
        );
        assert_eq!(inspection.component_count, 1);
        assert_eq!(
            inspection.generation_id.as_deref(),
            Some(generation.as_str())
        );
        assert!(inspection.manifest_hash.is_some());
        assert_eq!(before, after);

        let component = plugin_root
            .join("generations")
            .join(generation)
            .join("skills/fixture/skill.wasm");
        fs::write(component, b"tampered").unwrap();
        assert!(matches!(
            authorizer.inspect(),
            Err(TierWError::ComponentUnauthorized(_))
        ));
    }

    #[test]
    fn exact_pin_inspection_rejects_an_empty_trust_policy() {
        let empty = ComponentAuthorizer::hash_pins([]);
        assert!(matches!(
            empty.inspect(),
            Err(TierWError::ComponentUnauthorized(_))
        ));

        let digest = Digest::from_bytes(b"component");
        let ready = ComponentAuthorizer::bundled([digest]).inspect().unwrap();
        assert_eq!(ready.mode, ComponentAuthorizationMode::Bundled);
        assert_eq!(ready.component_count, 1);
    }

    #[cfg(all(unix, feature = "cranelift", feature = "estate"))]
    #[test]
    fn signed_generation_rejects_tampering_before_engine_validation() {
        let fixture = tempfile::tempdir().unwrap();
        let plugin_root = fixture.path().join("plugin");
        fs::create_dir_all(plugin_root.join("generations")).unwrap();
        let signing_key = SigningKey::from_bytes(&[73; 32]);
        write_trust_store(&plugin_root, &signing_key);
        let generation =
            write_generation(&plugin_root, &signing_key, REFERENCE_COMPONENT, "tamper");
        activate(&plugin_root, &generation);
        let authorizer = ComponentAuthorizer::estate(&plugin_root);
        let engine = TierWEngine::new(EngineProfile::desktop()).unwrap();

        let modified = component_with_custom_section(REFERENCE_COMPONENT, b'x');
        assert!(matches!(
            engine.load_component(&modified, &authorizer),
            Err(TierWError::ComponentUnauthorized(_))
        ));

        let signature_path = plugin_root
            .join("generations")
            .join(generation)
            .join("manifest.sig.json");
        let mut signature: Value =
            serde_json::from_slice(&fs::read(&signature_path).unwrap()).unwrap();
        signature["signature"] = Value::String(STANDARD.encode([0u8; 64]));
        fs::write(&signature_path, canonical_pretty_json(&signature).unwrap()).unwrap();
        assert!(matches!(
            engine.load_component(REFERENCE_COMPONENT, &authorizer),
            Err(TierWError::ComponentUnauthorized(_))
        ));
    }

    #[cfg(all(unix, feature = "estate"))]
    fn write_trust_store(plugin_root: &Path, signing_key: &SigningKey) {
        use std::os::unix::fs::PermissionsExt as _;

        let public_der = public_der(signing_key);
        let signer_key_id = raw_sha256(&public_der);
        let trust = json!({
            "schemaVersion": 1,
            "signers": [{
                "algorithm": "Ed25519",
                "keyId": signer_key_id,
                "publicKey": public_pem(&public_der),
            }],
        });
        let trust_dir = plugin_root.join("trust");
        fs::create_dir_all(&trust_dir).unwrap();
        let trust_path = trust_dir.join("allowed-signers.json");
        fs::write(&trust_path, canonical_pretty_json(&trust).unwrap()).unwrap();
        fs::set_permissions(&trust_path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    #[cfg(all(unix, feature = "estate"))]
    fn write_generation(
        plugin_root: &Path,
        signing_key: &SigningKey,
        component_bytes: &[u8],
        marker: &str,
    ) -> String {
        let signer_key_id = raw_sha256(&public_der(signing_key));
        let component_hash = Digest::from_bytes(component_bytes);
        let raw_component_hash = component_hash.as_str().strip_prefix("sha256:").unwrap();
        let mut manifest = json!({
            "bundleId": "fixture-bundle",
            "files": [{
                "mode": "0644",
                "path": "skills/fixture/skill.wasm",
                "sha256": raw_component_hash,
                "size": component_bytes.len(),
                "type": "file",
            }],
            "hookRuntime": {"abi": "hook-runtime-v1"},
            "schemaVersion": 1,
            "signerKeyId": signer_key_id,
            "skillIndex": {"entryCount": 1, "sha256": "fixture"},
            "sourceProvenance": {"fixture": marker},
            "sourceVersion": "1.7.0",
            "targetPayloads": [],
        });
        let object = manifest.as_object().unwrap();
        let mut identity = Map::new();
        for key in [
            "schemaVersion",
            "sourceVersion",
            "signerKeyId",
            "bundleId",
            "hookRuntime",
            "sourceProvenance",
            "skillIndex",
            "files",
            "targetPayloads",
        ] {
            identity.insert(key.into(), object[key].clone());
        }
        let generation = raw_sha256(&canonical_pretty_json(&Value::Object(identity)).unwrap());
        manifest["generation"] = Value::String(generation.clone());
        let canonical_manifest = canonical_pretty_json(&manifest).unwrap();
        let mut payload = format!("{SIGNATURE_NAMESPACE}\n").into_bytes();
        payload.extend_from_slice(&canonical_manifest);
        let signature = signing_key.sign(&payload);
        let envelope = json!({
            "algorithm": "Ed25519",
            "namespace": SIGNATURE_NAMESPACE,
            "schemaVersion": 1,
            "signature": STANDARD.encode(signature.to_bytes()),
            "signerKeyId": signer_key_id,
        });

        let generation_root = plugin_root.join("generations").join(&generation);
        let component_path = generation_root.join("skills/fixture/skill.wasm");
        fs::create_dir_all(component_path.parent().unwrap()).unwrap();
        fs::write(component_path, component_bytes).unwrap();
        fs::write(generation_root.join("manifest.json"), canonical_manifest).unwrap();
        fs::write(
            generation_root.join("manifest.sig.json"),
            canonical_pretty_json(&envelope).unwrap(),
        )
        .unwrap();
        generation
    }

    #[cfg(all(unix, feature = "estate"))]
    fn activate(plugin_root: &Path, generation: &str) {
        use std::os::unix::fs::symlink;

        let current = plugin_root.join("current");
        if current.symlink_metadata().is_ok() {
            fs::remove_file(&current).unwrap();
        }
        symlink(Path::new("generations").join(generation), current).unwrap();
    }

    #[cfg(feature = "estate")]
    fn public_der(signing_key: &SigningKey) -> Vec<u8> {
        let mut der = ED25519_SPKI_PREFIX.to_vec();
        der.extend_from_slice(signing_key.verifying_key().as_bytes());
        der
    }

    #[cfg(feature = "estate")]
    fn public_pem(der: &[u8]) -> String {
        format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
            STANDARD.encode(der)
        )
    }

    #[cfg(all(unix, feature = "estate"))]
    fn tree_snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        fn collect(root: &Path, current: &Path, snapshot: &mut Vec<(PathBuf, Vec<u8>)>) {
            let mut entries: Vec<_> = fs::read_dir(current)
                .unwrap()
                .map(|entry| entry.unwrap())
                .collect();
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                let path = entry.path();
                let relative = path.strip_prefix(root).unwrap().to_path_buf();
                let metadata = fs::symlink_metadata(&path).unwrap();
                if metadata.file_type().is_symlink() {
                    snapshot.push((
                        relative,
                        fs::read_link(path)
                            .unwrap()
                            .into_os_string()
                            .into_encoded_bytes(),
                    ));
                } else if metadata.is_dir() {
                    collect(root, &path, snapshot);
                } else {
                    snapshot.push((relative, fs::read(path).unwrap()));
                }
            }
        }

        let mut snapshot = Vec::new();
        collect(root, root, &mut snapshot);
        snapshot
    }

    #[cfg(feature = "estate")]
    fn component_with_custom_section(component: &[u8], marker: u8) -> Vec<u8> {
        let mut bytes = component.to_vec();
        bytes.extend_from_slice(&[0, 3, 2, b'p', marker]);
        bytes
    }
}
