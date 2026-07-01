use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Privacy classification for a syncable domain.
///
/// `DomainConfig` requires a `PrivacyClass` at construction time — there is
/// no default — so a domain cannot be registered into a `SyncManifest`
/// without an explicit privacy decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyClass {
    /// Safe to replicate to any paired peer (e.g. skill-index, public KB).
    Public,
    /// Replicates only to peers explicitly trusted by the local device owner.
    Trusted,
    /// Never leaves the local device — structurally excluded from P2P sync.
    Local,
}

/// Identifies a syncable data domain: a CRDT document tree rooted at a
/// storage key prefix, e.g. `"learner-model"`, `"skill-index"`, or a custom
/// knowledge base (`"kb:<name>"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SyncDomain(pub String);

impl SyncDomain {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl std::fmt::Display for SyncDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Per-domain sync configuration: its privacy class and the storage key
/// prefix it owns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub privacy: PrivacyClass,
    /// Storage key prefix this domain owns (e.g. `"learner/"`, `"kb/clinical/"`).
    pub key_prefix: String,
}

impl DomainConfig {
    pub fn new(privacy: PrivacyClass, key_prefix: impl Into<String>) -> Self {
        Self {
            privacy,
            key_prefix: key_prefix.into(),
        }
    }
}

/// Registry of which domains are eligible for P2P sync and at what privacy
/// class.
///
/// A sync transport (e.g. `sovereign-sync`) is expected to consult this
/// manifest before gossiping any CRDT delta for a domain — a domain absent
/// from the manifest, or classed [`PrivacyClass::Local`], must never be put
/// on the wire. This module only defines the data model and the
/// `is_syncable` predicate; enforcing it at the transport layer is the
/// caller's responsibility.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncManifest {
    domains: HashMap<String, DomainConfig>,
}

impl SyncManifest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, domain: SyncDomain, config: DomainConfig) {
        self.domains.insert(domain.0, config);
    }

    pub fn config_for(&self, domain: &SyncDomain) -> Option<&DomainConfig> {
        self.domains.get(&domain.0)
    }

    /// Whether a domain is permitted to leave the local device at all.
    /// `false` for unregistered domains and for [`PrivacyClass::Local`].
    pub fn is_syncable(&self, domain: &SyncDomain) -> bool {
        matches!(
            self.config_for(domain).map(|c| c.privacy),
            Some(PrivacyClass::Public) | Some(PrivacyClass::Trusted)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unregistered_domain_is_not_syncable() {
        let manifest = SyncManifest::new();
        assert!(!manifest.is_syncable(&SyncDomain::new("learner-model")));
    }

    #[test]
    fn local_domain_is_not_syncable() {
        let mut manifest = SyncManifest::new();
        let domain = SyncDomain::new("kb:clinical-protocols");
        manifest.register(
            domain.clone(),
            DomainConfig::new(PrivacyClass::Local, "kb/clinical-protocols/"),
        );
        assert!(!manifest.is_syncable(&domain));
    }

    #[test]
    fn public_and_trusted_domains_are_syncable() {
        let mut manifest = SyncManifest::new();
        let pub_domain = SyncDomain::new("skill-index");
        let trusted_domain = SyncDomain::new("learner-model");
        manifest.register(
            pub_domain.clone(),
            DomainConfig::new(PrivacyClass::Public, "skills/"),
        );
        manifest.register(
            trusted_domain.clone(),
            DomainConfig::new(PrivacyClass::Trusted, "learner/"),
        );
        assert!(manifest.is_syncable(&pub_domain));
        assert!(manifest.is_syncable(&trusted_domain));
    }
}
