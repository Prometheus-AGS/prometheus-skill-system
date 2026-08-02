//! Transitional single-writer compatibility policy for the KBD control plane.
//!
//! The public constructors and status shape are retained during stabilization.
//! Configurations naming more than one voter are rejected explicitly.

use std::{collections::BTreeSet, io};

use serde::{Deserialize, Serialize};

pub type KbdNodeId = u64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuorumStatus {
    pub configured_voters: usize,
    pub available_voters: usize,
    pub quorum_size: usize,
    pub writable: bool,
    pub standalone_non_ha: bool,
    pub automatic_takeover: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuorumPolicy {
    node_id: KbdNodeId,
    voters: BTreeSet<KbdNodeId>,
}

impl QuorumPolicy {
    pub fn new(
        node_id: KbdNodeId,
        voters: impl IntoIterator<Item = KbdNodeId>,
    ) -> io::Result<Self> {
        let voters = voters.into_iter().collect::<BTreeSet<_>>();
        if node_id == 0 || !voters.contains(&node_id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "KBD writer id must be nonzero and included in kbd.voters",
            ));
        }
        if voters.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "multi-voter KBD configuration is unsupported; configure exactly one local writer",
            ));
        }
        Ok(Self { node_id, voters })
    }

    pub fn node_id(&self) -> KbdNodeId {
        self.node_id
    }

    pub fn voters(&self) -> &BTreeSet<KbdNodeId> {
        &self.voters
    }

    pub fn status(&self, available: impl IntoIterator<Item = KbdNodeId>) -> QuorumStatus {
        let available_voters = available
            .into_iter()
            .filter(|node| self.voters.contains(node))
            .collect::<BTreeSet<_>>()
            .len();
        let writable = available_voters == 1;
        QuorumStatus {
            configured_voters: 1,
            available_voters,
            quorum_size: 1,
            writable,
            standalone_non_ha: true,
            automatic_takeover: false,
            reason: if writable {
                "single journal writer available; no automatic failover".into()
            } else {
                "read-only: the configured journal writer is unavailable".into()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_writer_and_rejects_multi_voter_configuration() {
        let policy = QuorumPolicy::new(7, [7]).unwrap();
        assert!(policy.status([7]).writable);
        assert!(!policy.status([]).writable);

        let error = QuorumPolicy::new(7, [7, 8]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}
