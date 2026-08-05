use landlock::{LandlockStatus, RestrictionStatus, RulesetStatus};

use crate::{LandlockClassification, LandlockCompatibility, LandlockProbe, LandlockRulesetStatus};

/// Converts the safe rust-landlock restriction result into the portable,
/// receipt-facing classification. This is called only inside the future
/// single-threaded helper after `restrict_self`; it never probes by mutating
/// the long-lived sidecar process.
pub fn classify_landlock_restriction(status: &RestrictionStatus) -> LandlockClassification {
    let (effective_abi, kernel_abi) = match status.landlock {
        LandlockStatus::Available {
            effective_abi,
            kernel_abi,
        } => (Some(effective_abi as u32), kernel_abi),
        LandlockStatus::NotEnabled | LandlockStatus::NotImplemented => (None, None),
    };
    let ruleset = match status.ruleset {
        RulesetStatus::FullyEnforced => LandlockRulesetStatus::FullyEnforced,
        RulesetStatus::PartiallyEnforced => LandlockRulesetStatus::PartiallyEnforced,
        RulesetStatus::NotEnforced => LandlockRulesetStatus::NotEnforced,
    };
    LandlockClassification::classify(&LandlockProbe {
        compatibility: LandlockCompatibility::BestEffort,
        ruleset,
        no_new_privs: status.no_new_privs,
        effective_abi,
        kernel_abi,
    })
}
