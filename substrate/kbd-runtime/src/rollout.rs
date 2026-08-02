use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

/// Rollout evidence is operational metadata only. It can gate promotion, but it
/// cannot append an authoritative event or mutate KbdStateV2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RolloutStage {
    Shadow,
    CanaryLocal,
    CanaryHarnesses,
    CanaryQuorum,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RolloutObservation {
    pub observation_id: String,
    pub observed_at: DateTime<Utc>,
    pub real_mutations: u64,
    pub synthetic_replay_mutations: u64,
    pub unexplained_projection_mismatches: u64,
    #[serde(default)]
    pub projection_mismatches: Vec<PathBuf>,
    pub harness: Option<String>,
    pub device: Option<String>,
    pub voters: u64,
    pub successful: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RolloutEvidence {
    pub schema_version: String,
    pub stage: RolloutStage,
    pub stage_started_at: DateTime<Utc>,
    pub observations: BTreeMap<String, RolloutObservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromotionGate {
    pub from: RolloutStage,
    pub to: RolloutStage,
    pub passed: bool,
    pub consecutive_successful_days: u64,
    pub real_mutations: u64,
    pub synthetic_replay_mutations: u64,
    pub unexplained_projection_mismatches: u64,
    pub harnesses: BTreeSet<String>,
    pub devices: BTreeSet<String>,
    pub max_voters: u64,
    pub failures: Vec<String>,
}

impl RolloutEvidence {
    fn new(now: DateTime<Utc>) -> Self {
        Self {
            schema_version: "1".into(),
            stage: RolloutStage::Shadow,
            stage_started_at: now,
            observations: BTreeMap::new(),
        }
    }

    pub fn gate(&self) -> PromotionGate {
        let target = match self.stage {
            RolloutStage::Shadow => RolloutStage::CanaryLocal,
            RolloutStage::CanaryLocal => RolloutStage::CanaryHarnesses,
            RolloutStage::CanaryHarnesses => RolloutStage::CanaryQuorum,
            RolloutStage::CanaryQuorum => RolloutStage::Production,
            RolloutStage::Production => RolloutStage::Production,
        };
        let observations = self
            .observations
            .values()
            .filter(|observation| observation.observed_at >= self.stage_started_at)
            .collect::<Vec<_>>();
        let real_mutations = observations
            .iter()
            .map(|observation| observation.real_mutations)
            .sum();
        let synthetic_replay_mutations = observations
            .iter()
            .map(|observation| observation.synthetic_replay_mutations)
            .sum();
        let unexplained_projection_mismatches = observations
            .iter()
            .map(|observation| observation.unexplained_projection_mismatches)
            .sum();
        let harnesses = observations
            .iter()
            .filter_map(|observation| observation.harness.clone())
            .collect::<BTreeSet<_>>();
        let devices = observations
            .iter()
            .filter_map(|observation| observation.device.clone())
            .collect::<BTreeSet<_>>();
        let max_voters = observations
            .iter()
            .map(|observation| observation.voters)
            .max()
            .unwrap_or_default();
        let days = observations
            .iter()
            .filter(|observation| observation.successful)
            .map(|observation| observation.observed_at.date_naive())
            .collect::<BTreeSet<_>>();
        let consecutive_successful_days = consecutive_days_ending_at_latest(&days);
        let mut failures = Vec::new();
        match self.stage {
            RolloutStage::Shadow => {
                require(
                    consecutive_successful_days >= 7,
                    "shadow mode requires seven consecutive successful days",
                    &mut failures,
                );
                require(
                    real_mutations >= 100,
                    "shadow mode requires at least 100 real mutations",
                    &mut failures,
                );
                require(
                    synthetic_replay_mutations >= 10_000,
                    "shadow mode requires at least 10,000 synthetic replay mutations",
                    &mut failures,
                );
                require(
                    unexplained_projection_mismatches == 0,
                    "shadow mode has unexplained projection mismatches",
                    &mut failures,
                );
            }
            RolloutStage::CanaryLocal => {
                require(
                    consecutive_successful_days >= 3,
                    "local canary requires three consecutive successful days",
                    &mut failures,
                );
                require(
                    unexplained_projection_mismatches == 0,
                    "local canary has unexplained projection mismatches",
                    &mut failures,
                );
            }
            RolloutStage::CanaryHarnesses => {
                require(
                    consecutive_successful_days >= 3,
                    "all-harness canary requires three consecutive successful days",
                    &mut failures,
                );
                for harness in ["claude-code", "codex", "opencode", "kimi"] {
                    require(
                        harnesses.contains(harness),
                        &format!("all-harness canary has no successful {harness} observation"),
                        &mut failures,
                    );
                }
            }
            RolloutStage::CanaryQuorum => {
                require(
                    consecutive_successful_days >= 7,
                    "quorum canary requires seven consecutive successful days",
                    &mut failures,
                );
                require(
                    devices.len() >= 2,
                    "quorum canary requires at least two devices",
                    &mut failures,
                );
                require(
                    max_voters >= 3,
                    "quorum canary requires at least three voters",
                    &mut failures,
                );
                require(
                    unexplained_projection_mismatches == 0,
                    "quorum canary has unexplained projection mismatches",
                    &mut failures,
                );
            }
            RolloutStage::Production => {
                failures.push("runtime is already in production stage".into());
            }
        }
        PromotionGate {
            from: self.stage.clone(),
            to: target,
            passed: failures.is_empty(),
            consecutive_successful_days,
            real_mutations,
            synthetic_replay_mutations,
            unexplained_projection_mismatches,
            harnesses,
            devices,
            max_voters,
            failures,
        }
    }
}

pub struct RolloutTracker {
    path: PathBuf,
}

impl RolloutTracker {
    pub fn open(runtime_root: impl AsRef<Path>) -> Self {
        Self {
            path: runtime_root.as_ref().join("rollout-evidence.json"),
        }
    }

    pub fn load(&self) -> io::Result<RolloutEvidence> {
        if !self.path.exists() {
            return Ok(RolloutEvidence::new(Utc::now()));
        }
        let file = File::open(&self.path)?;
        let evidence: RolloutEvidence = serde_json::from_reader(file)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if evidence.schema_version != "1" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported rollout evidence schema",
            ));
        }
        Ok(evidence)
    }

    pub fn record(&self, observation: RolloutObservation) -> io::Result<RolloutEvidence> {
        let mut evidence = self.load()?;
        if let Some(existing) = evidence.observations.get(&observation.observation_id) {
            if existing == &observation {
                return Ok(evidence);
            }
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "observationId was reused with different evidence",
            ));
        }
        if evidence.observations.is_empty() {
            evidence.stage_started_at = observation.observed_at;
        }
        evidence
            .observations
            .insert(observation.observation_id.clone(), observation);
        self.store(&evidence)?;
        Ok(evidence)
    }

    pub fn promote(&self) -> io::Result<RolloutEvidence> {
        let mut evidence = self.load()?;
        let gate = evidence.gate();
        if !gate.passed {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                gate.failures.join("; "),
            ));
        }
        evidence.stage = gate.to;
        evidence.stage_started_at = Utc::now();
        self.store(&evidence)?;
        Ok(evidence)
    }

    pub fn reset_to_shadow(&self) -> io::Result<RolloutEvidence> {
        let evidence = RolloutEvidence::new(Utc::now());
        self.store(&evidence)?;
        Ok(evidence)
    }

    fn store(&self, evidence: &RolloutEvidence) -> io::Result<()> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid rollout path"))?;
        fs::create_dir_all(parent)?;
        let temporary = parent.join(format!(".rollout-{}.tmp", Uuid::new_v4()));
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        serde_json::to_writer_pretty(&mut file, evidence).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&temporary, &self.path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    }
}

fn require(condition: bool, message: &str, failures: &mut Vec<String>) {
    if !condition {
        failures.push(message.into());
    }
}

fn consecutive_days_ending_at_latest(days: &BTreeSet<NaiveDate>) -> u64 {
    let Some(latest) = days.last().copied() else {
        return 0;
    };
    let mut count = 0;
    let mut day = latest;
    while days.contains(&day) {
        count += 1;
        let Some(previous) = day.pred_opt() else {
            break;
        };
        day = previous;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tempfile::tempdir;

    fn observation(day: DateTime<Utc>, sequence: u64) -> RolloutObservation {
        RolloutObservation {
            observation_id: format!("observation-{sequence}"),
            observed_at: day,
            real_mutations: 15,
            synthetic_replay_mutations: 1_500,
            unexplained_projection_mismatches: 0,
            projection_mismatches: Vec::new(),
            harness: Some("codex".into()),
            device: Some("device-a".into()),
            voters: 1,
            successful: true,
        }
    }

    #[test]
    fn shadow_evidence_is_idempotent_and_cannot_promote_early() {
        let directory = tempdir().unwrap();
        let tracker = RolloutTracker::open(directory.path());
        let now = Utc::now();
        let first = observation(now, 1);
        tracker.record(first.clone()).unwrap();
        tracker.record(first).unwrap();
        assert!(tracker.promote().is_err());
        assert_eq!(tracker.load().unwrap().observations.len(), 1);
    }

    #[test]
    fn shadow_promotes_only_after_all_acceptance_thresholds() {
        let directory = tempdir().unwrap();
        let tracker = RolloutTracker::open(directory.path());
        let now = Utc::now();
        for offset in 0..7 {
            tracker
                .record(observation(now - Duration::days(6 - offset), offset as u64))
                .unwrap();
        }
        let gate = tracker.load().unwrap().gate();
        assert!(gate.passed, "{:?}", gate.failures);
        assert!(gate.real_mutations >= 100);
        assert!(gate.synthetic_replay_mutations >= 10_000);
        assert_eq!(tracker.promote().unwrap().stage, RolloutStage::CanaryLocal);
    }
}
