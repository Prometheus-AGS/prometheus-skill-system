# Derived method context

Method: VLBV (pack-default); phase: vlbv-r4-portability; revision: 2527; status: RESOLVED_REFERENCES.

This is a recovery index, not execution permission. Resolve current canonical decisions and contract before dependent action.

```json
{
  "schemaVersion": 1,
  "kind": "kbd.derived-method-context",
  "generatedAt": "2026-11-16T08:36:11.034Z",
  "authority": "derived-read-only-index; canonical signed decisions, phase binding, contract and governing host policy remain authority",
  "projectId": "6ac090a4-3656-4d83-8eb6-2891508196d5",
  "runId": "sovereign-sync-service-reliability-20260829",
  "phaseId": "vlbv-r4-portability",
  "sourceRevision": 2527,
  "phaseDirectory": ".kbd-orchestrator/phases/vlbv-r4-portability",
  "contract": {
    "path": ".kbd-orchestrator/phases/vlbv-r4-portability/adopted-contract.json",
    "observation": "observed",
    "sha256": "f6d60e92e1f5f048cc1bab6443ad39fa8c920fc272372af1e5e9ccb59867f5f7",
    "contractDigest": "baa987b2956f8973c8e5047015d0b488e97411e0c42d46ee534f3cc8081def68",
    "binding": {
      "contractPath": ".kbd-orchestrator/phases/vlbv-r4-portability/adopted-contract.json",
      "contractDigest": "baa987b2956f8973c8e5047015d0b488e97411e0c42d46ee534f3cc8081def68",
      "scenarioDigests": {
        "AUTH-ISO-1": "ccd227b5f49cd11022bd4e9dd3fc1556e845767ffd584de9caf841c648640139",
        "CAP-UPDATE-1": "b164d041a89aeb5b3df287a25e47edac56142c79bb1edebe5e8b5b19d6e6c888",
        "EVAL-CLAUDE-1": "90849ed8b2064a54bc665514c0c976cd7f85c40a0583d30198c04b803c802e58",
        "EVAL-CODEX-1": "9e447bd45cbc8e72c59163c2e6003d0c0c3dc0ec2415afd285ff33d0fa54e280",
        "EVAL-KIMI-1": "076a8cbcd25981e3b33803319a17e6ccf3059eec1df53dd55e7a5ac737690d4a",
        "EVAL-MACH-1": "44167fd35fda9c8ad7e7672412279837ab00e0e187edf10e8fbe601f19de21fe",
        "EVAL-NOTRUN-1": "fee1852f756befb88b75aaaa7fe5f756bb72db4ce4e26a7248829dc01563e1a2",
        "EVAL-OPENCODE-1": "62cde8fa9490d8c8fa2284ec8386f9a61ce9e94255c6703ead94c48c594f691d",
        "EVAL-QWEN-1": "cae3dbb76a74717e7991880df0c68555939d33fadb7040c2e5126bde1c549d22",
        "SCAN-PRE-1": "576649173dfb7b1c43eaf60987eb25f073cd41b699f5eb7bcf19e1cdfba6f9ca"
      },
      "planRevision": 17,
      "adoptionDecisionId": "contract-adoption-vlbv-r4-portability-baa987b2",
      "bindingRevision": 1
    },
    "identity": {
      "projectId": "6ac090a4-3656-4d83-8eb6-2891508196d5",
      "runId": "sovereign-sync-service-reliability-20260829",
      "phaseId": "vlbv-r4-portability",
      "planRevision": 17,
      "contractDigest": "baa987b2956f8973c8e5047015d0b488e97411e0c42d46ee534f3cc8081def68"
    }
  },
  "selectedMethod": "VLBV",
  "methodSource": "pack-default",
  "deviation": null,
  "methodDecision": null,
  "retiredMethods": [],
  "references": [],
  "includedValue": [
    "Per-harness protocol eval reports",
    "Machinery cases executed once and counted in every report",
    "Capability profiles derived from reports",
    "Passive pre-authorization artifact scan",
    "Isolated runs with temporary auth material",
    {
      "item": "B4 authorization event record",
      "nonObservable": true,
      "reason": "Observed only through SCAN-PRE-1"
    }
  ],
  "exclusions": [
    "Executing evals on zed, minimax-code or mimo-code",
    "CI or hosted runners",
    "Tool guards or PreToolUse hooks (CF-36)",
    "Running evals against real harness homes",
    "Submodule changes"
  ],
  "stage": "assess",
  "stageAuthority": {
    "operations": [
      "discover/define/plan within governing policy"
    ],
    "implementationComplete": true,
    "verificationAuthority": "separately authorized whole-value-phase boundary required; this index grants none",
    "executableTestAuthoring": "governed by host policy; where that policy is production-first, defer until the complete agreed production work"
  },
  "governingInstructions": [
    {
      "path": "CLAUDE.md",
      "observation": "observed",
      "sha256": "245990f7291de1bed30a6171d4785a1c300122ef3ae0eaafc408f611babdd17a"
    },
    {
      "path": ".kbd-orchestrator/constraints.md",
      "observation": "observed",
      "sha256": "13b4a478549c11e69aa3c2fdba583a54654ff4b0cd0756de1a97dbc3aead5a44"
    }
  ],
  "effectiveRules": {
    "harness": "claude-code",
    "profile": "found",
    "files": [
      "CLAUDE.md",
      ".kbd-orchestrator/constraints.md"
    ],
    "shadowing": null
  },
  "policyConflicts": [],
  "candidateAndEvidence": {
    "freshness": "Per-scenario invalidationKeys; only affected evidence is invalidated.",
    "paths": [
      "boundary-results.json",
      "evals/reports/"
    ]
  },
  "budgets": {
    "adopted": {
      "timeMinutes": 600,
      "toolCalls": 4000,
      "repairAttempts": 1,
      "noProgressLimit": 3,
      "tokenBudget": "UNKNOWN",
      "tokenFallback": "toolCalls ceiling",
      "exhaustion": "checkpoint + BLOCKED; never silent PASS"
    },
    "consumption": "UNKNOWN; recover original canonical/evidence receipts before execution"
  },
  "unresolved": {
    "boundaryObligations": {
      "change:change-ldd-03-published-memory-pins": {
        "key": "change:change-ldd-03-published-memory-pins",
        "receiptId": "e59df779a9057902c443f750f79e5381cdd879062f2ec7fd7e30053519c291ce",
        "boundary": "change",
        "subject": "change-ldd-03-published-memory-pins",
        "sourceRevision": 2344,
        "exactSignal": "Starting change 14 out of 18: Freeze and apply owner-approved final release pins across full mini and services"
      },
      "change:change-ldd-12-integration-rollout": {
        "key": "change:change-ldd-12-integration-rollout",
        "receiptId": "1d7519762413ad00163b06ce5ff9d0d4acbffea19718d55f4e24e6b7d6f2d5bb",
        "boundary": "change",
        "subject": "change-ldd-12-integration-rollout",
        "sourceRevision": 2371,
        "exactSignal": "Starting change 18 out of 18: Reconcile generated outputs certify locally and hand off approved rollout"
      },
      "change:change-tli-a5b-mini-pin-operator-request": {
        "key": "change:change-tli-a5b-mini-pin-operator-request",
        "receiptId": "da33294902c20e8b7dea01813dec4a98813e203a324eaa4faf2474b7deb85775",
        "boundary": "change",
        "subject": "change-tli-a5b-mini-pin-operator-request",
        "sourceRevision": 1683,
        "exactSignal": "Starting change 9 out of 25: Operator pin request: mini gitlinks and versions.toml to v1.10.0 (operator-authored)"
      },
      "change:change-tli-b5-subagentstart-delivery-alpha": {
        "key": "change:change-tli-b5-subagentstart-delivery-alpha",
        "receiptId": "9a5b8477c27db80037ea8cbf63a62f5a9c5e26dec9a843956869aded43eaaccc",
        "boundary": "change",
        "subject": "change-tli-b5-subagentstart-delivery-alpha",
        "sourceRevision": 1832,
        "exactSignal": "Starting change 15 out of 25: SubagentStart delivery hook in both harnesses — alpha gate"
      },
      "change:change-tli-b6-file-tier-reduction": {
        "key": "change:change-tli-b6-file-tier-reduction",
        "receiptId": "051e2a2989069142ce545bfb7650191f75ad673187622a8f031e2ba1735bdeb6",
        "boundary": "change",
        "subject": "change-tli-b6-file-tier-reduction",
        "sourceRevision": 1756,
        "exactSignal": "Starting change 17 out of 25: Reduce the file-memory tier: MEMORY.md index ≤ 4 KB, per-role local memory, Codex names and memory controls"
      },
      "change:change-tli-b7-cross-agent-awareness-beta": {
        "key": "change:change-tli-b7-cross-agent-awareness-beta",
        "receiptId": "e394f95985fb6ee94b6d83a002ff35caf5e4c32a304da821c45fbe7f7fe7de20",
        "boundary": "change",
        "subject": "change-tli-b7-cross-agent-awareness-beta",
        "sourceRevision": 1872,
        "exactSignal": "Starting change 18 out of 25: Path-overlap routing, team digest and lead view — beta gate"
      },
      "change:change-tli-c1b-promotion-routing": {
        "key": "change:change-tli-c1b-promotion-routing",
        "receiptId": "466834bf45b539e775996fda799415d1f45c899779f9d6a44c7a49a09722e9cd",
        "boundary": "change",
        "subject": "change-tli-c1b-promotion-routing",
        "sourceRevision": 1885,
        "exactSignal": "Starting change 19 out of 25: [GLOBAL]/[USER] routing, user/global recall quotas, and candidate review at reflect"
      },
      "change:change-tli-c2-feynman-gap-routing": {
        "key": "change:change-tli-c2-feynman-gap-routing",
        "receiptId": "dc2b333b8b7a38148a9c9e190a56c87e8053d837af40e66543cb4be863bb1b79",
        "boundary": "change",
        "subject": "change-tli-c2-feynman-gap-routing",
        "sourceRevision": 1747,
        "exactSignal": "Starting change 20 out of 25: Knowledge-gap detection on prompts with /learn-goal suggestion and gap resolution"
      },
      "change:change-tli-c3b-skill-candidate-surfacing": {
        "key": "change:change-tli-c3b-skill-candidate-surfacing",
        "receiptId": "c6166b03a0a913487fb51a8b936496db1b75357ef42f31487bbd8fde78827a2f",
        "boundary": "change",
        "subject": "change-tli-c3b-skill-candidate-surfacing",
        "sourceRevision": 1942,
        "exactSignal": "Starting change 24 out of 25: Surface skill candidates in kbd-open and reflect"
      },
      "change:change-tli-c4-cross-repo-team-requests": {
        "key": "change:change-tli-c4-cross-repo-team-requests",
        "receiptId": "7f6506f9e366f94d62ff5b5c74fd4bfbfcdd7b4c222b8f2ede18c48a7b14f598",
        "boundary": "change",
        "subject": "change-tli-c4-cross-repo-team-requests",
        "sourceRevision": 1767,
        "exactSignal": "Starting change 21 out of 25: Team cards, discovery, and same-repo handoff vs cross-repo GitHub issue requests"
      },
      "change:change-tli-d1a-mini-file-tier-port": {
        "key": "change:change-tli-d1a-mini-file-tier-port",
        "receiptId": "08f957515d7a4ed150bee2d087f13b47e1116180d8f2d6438b6a1c3f5c932f9a",
        "boundary": "change",
        "subject": "change-tli-d1a-mini-file-tier-port",
        "sourceRevision": 1916,
        "exactSignal": "Starting change 22 out of 25: Mini: envelope on Karpathy records, identity resolver port, file-tier SubagentStart hook (Claude)"
      },
      "change:change-tli-d1b-mini-pk-tags": {
        "key": "change:change-tli-d1b-mini-pk-tags",
        "receiptId": "147de7a1506b5bb43a2ad21c547e5a1cec630a54ba956978bfbd02553cce0508",
        "boundary": "change",
        "subject": "change-tli-d1b-mini-pk-tags",
        "sourceRevision": 1955,
        "exactSignal": "Starting change 25 out of 25: Mini: use pk --type/--tag when delivering Karpathy records"
      },
      "change:change-tli-d3-claude-md-memory-chain-cortex": {
        "key": "change:change-tli-d3-claude-md-memory-chain-cortex",
        "receiptId": "373dd4e2343155d237ebb53d950fec1bbfa22cdb37588a10d05aa13978f6feba",
        "boundary": "change",
        "subject": "change-tli-d3-claude-md-memory-chain-cortex",
        "sourceRevision": 1929,
        "exactSignal": "Starting change 23 out of 25: CLAUDE.md memory chain describes per-role delivery; optional Cortex mirror"
      },
      "change:change-tli-e1-pin-v1-11-0": {
        "key": "change:change-tli-e1-pin-v1-11-0",
        "receiptId": "966044661d110368a43d61a0bd94e8f2dd8d266393e5fee177239bb4772358fe",
        "boundary": "change",
        "subject": "change-tli-e1-pin-v1-11-0",
        "sourceRevision": 1849,
        "exactSignal": "Starting change 16 out of 25: Tag pk v1.11.0 (promotion + skill discovery) and bump the skill-pack pin; operator request for mini"
      },
      "phase:control-plane-to-companion": {
        "key": "phase:control-plane-to-companion",
        "receiptId": "6b623f80c1b11b83b9f02c983b3fa8c07459d98f1f089f9b75ff26b892914cc1",
        "boundary": "phase",
        "subject": "control-plane-to-companion",
        "sourceRevision": 385,
        "exactSignal": "Starting phase 3 out of 3: control-plane-to-companion"
      },
      "phase:deep-research-onyx-parity": {
        "key": "phase:deep-research-onyx-parity",
        "receiptId": "c84904edcec054b380cb083745af2928d4c408ac22967485d7a9968c14a4119d",
        "boundary": "phase",
        "subject": "deep-research-onyx-parity",
        "sourceRevision": 916,
        "exactSignal": "Starting phase 5 out of 5: deep-research-onyx-parity"
      },
      "phase:deep-research-onyx-parity::stage-10-hang-investigation": {
        "key": "phase:deep-research-onyx-parity::stage-10-hang-investigation",
        "receiptId": "522f4364cdba756d39d95e7182db10c2e7152862b20f5988c3080a5f69458333",
        "boundary": "phase",
        "subject": "deep-research-onyx-parity::stage-10-hang-investigation",
        "sourceRevision": 1102,
        "exactSignal": "Starting phase 1 out of 1: stage-10-hang-investigation"
      },
      "phase:issue-195-closeout": {
        "key": "phase:issue-195-closeout",
        "receiptId": "6388876f6c9f7d842a9f1067f34c4dfe6f2b90a20345a914cc742028e9f28026",
        "boundary": "phase",
        "subject": "issue-195-closeout",
        "sourceRevision": 2425,
        "exactSignal": "Starting phase 11 out of 11: issue-195-closeout"
      },
      "phase:phase-learning-deploy-and-debt": {
        "key": "phase:phase-learning-deploy-and-debt",
        "receiptId": "56c3998ff5aa9ab39b85591b5ff78bf78ab8b610fa882a3b421279832111c4dd",
        "boundary": "phase",
        "subject": "phase-learning-deploy-and-debt",
        "sourceRevision": 2134,
        "exactSignal": "Starting phase 10 out of 10: phase-learning-deploy-and-debt"
      },
      "phase:research-agent-hardening": {
        "key": "phase:research-agent-hardening",
        "receiptId": "8be7b4584cf2ee95249a7386c3e2bb7a645d67da0998b214aaff20a7b7ccf8a4",
        "boundary": "phase",
        "subject": "research-agent-hardening",
        "sourceRevision": 612,
        "exactSignal": "Starting phase 4 out of 4: research-agent-hardening"
      },
      "phase:team-aware-learning-memory": {
        "key": "phase:team-aware-learning-memory",
        "receiptId": "e663e02d28f663917ab6fe4b28debfec7a5fefc8b8226249e89d45f7695fd765",
        "boundary": "phase",
        "subject": "team-aware-learning-memory",
        "sourceRevision": 1365,
        "exactSignal": "Starting phase 7 out of 7: team-aware-learning-memory"
      },
      "task:change-ldd-03-published-memory-pins/2": {
        "key": "task:change-ldd-03-published-memory-pins/2",
        "receiptId": "0e2af5d2f469ecb112e611249adaad7fbd004e75e02a504d183d5b0a47c3e5d8",
        "boundary": "task",
        "subject": "change-ldd-03-published-memory-pins/2",
        "sourceRevision": 2349,
        "exactSignal": "Starting task 2 out of 3: Freeze exact compatible release identities and obtain protected-edit and tag approvals"
      },
      "task:change-ldd-03-published-memory-pins/3": {
        "key": "task:change-ldd-03-published-memory-pins/3",
        "receiptId": "3b415c7531edfd822ef3ebac9bd384ba92ee73e4550e2f1cc241d898520fb1de",
        "boundary": "task",
        "subject": "change-ldd-03-published-memory-pins/3",
        "sourceRevision": 2369,
        "exactSignal": "Starting task 3 out of 3: Apply approved final gitlinks and coherent version declarations"
      },
      "task:change-ldd-12-integration-rollout/2": {
        "key": "task:change-ldd-12-integration-rollout/2",
        "receiptId": "258d1f9844419f911d4305242b2ed9f9306ac5a4a57dff30c4d4764bd90c9caa",
        "boundary": "task",
        "subject": "change-ldd-12-integration-rollout/2",
        "sourceRevision": 2376,
        "exactSignal": "Starting task 2 out of 6: Run the consolidated local product services sites and fresh-clone release gates"
      },
      "task:create companion skills/sync-status|sync-peers|sync-push and skill-package.json; rewrite endpoint references": {
        "key": "task:create companion skills/sync-status|sync-peers|sync-push and skill-package.json; rewrite endpoint references",
        "receiptId": "48d1e26a2d1575c0d23a3d5eabe26e996615b0b000e428077d7aeb73c41fb4b2",
        "boundary": "task",
        "subject": "Create Companion skills/sync-status|sync-peers|sync-push and skill-package.json; rewrite endpoint references",
        "sourceRevision": 608,
        "exactSignal": "Starting task 1 out of 1: Create Companion skills/sync-status|sync-peers|sync-push and skill-package.json; rewrite endpoint references"
      },
      "task:define the label enum, graph.json schema with claim labels, and the package derivation rule in okf-research-format.md and research-package-spec.md": {
        "key": "task:define the label enum, graph.json schema with claim labels, and the package derivation rule in okf-research-format.md and research-package-spec.md",
        "receiptId": "212075ce31b87ecf2e6c1843a500c85d5bee7fdbad7e224a0890e0e5634da086",
        "boundary": "task",
        "subject": "Define the label enum, graph.json schema with claim labels, and the package derivation rule in okf-research-format.md and research-package-spec.md",
        "sourceRevision": 758,
        "exactSignal": "Starting task 1 out of 1: Define the label enum, graph.json schema with claim labels, and the package derivation rule in okf-research-format.md and research-package-spec.md"
      },
      "task:implement scripts/merge-threads.sh: union sources by canonical url into the existing stage 02/04 shapes, chunks into stage 03, claims by content-addressed id, and write citation-map.json": {
        "key": "task:implement scripts/merge-threads.sh: union sources by canonical url into the existing stage 02/04 shapes, chunks into stage 03, claims by content-addressed id, and write citation-map.json",
        "receiptId": "c894f0329b2fcf31dbed255a7b2379cdf8abe3aa41e740031d0cf46a3343ff02",
        "boundary": "task",
        "subject": "Implement scripts/merge-threads.sh: union sources by canonical URL into the existing stage 02/04 shapes, chunks into stage 03, claims by content-addressed id, and write citation-map.json",
        "sourceRevision": 1010,
        "exactSignal": "Starting task 1 out of 1: Implement scripts/merge-threads.sh: union sources by canonical URL into the existing stage 02/04 shapes, chunks into stage 03, claims by content-addressed id, and write citation-map.json"
      },
      "task:re-derive and record the footprint; delete substrate/sovereign-sync, sovereign-client, kbd-mobile, storage-provider iroh_docs.rs and its feature; retire the four sovereign-sync openspec main specs with a decision record; delete docs/sovereign_sync_testing.md; cut the estate-feature dependency in prometheus-exec/exec-remote": {
        "key": "task:re-derive and record the footprint; delete substrate/sovereign-sync, sovereign-client, kbd-mobile, storage-provider iroh_docs.rs and its feature; retire the four sovereign-sync openspec main specs with a decision record; delete docs/sovereign_sync_testing.md; cut the estate-feature dependency in prometheus-exec/exec-remote",
        "receiptId": "9cf4d5ab1c96960eab82a1ead9b854e2f84ce43354c3f0b6c909332039d7fe03",
        "boundary": "task",
        "subject": "Re-derive and record the footprint; delete substrate/sovereign-sync, sovereign-client, kbd-mobile, storage-provider iroh_docs.rs and its feature; retire the four sovereign-sync OpenSpec main specs with a decision record; delete docs/SOVEREIGN_SYNC_TESTING.md; cut the estate-feature dependency in prometheus-exec/exec-remote",
        "sourceRevision": 650,
        "exactSignal": "Starting task 1 out of 1: Re-derive and record the footprint; delete substrate/sovereign-sync, sovereign-client, kbd-mobile, storage-provider iroh_docs.rs and its feature; retire the four sovereign-sync OpenSpec main specs with a decision record; delete docs/SOVEREIGN_SYNC_TESTING.md; cut the estate-feature dependency in prometheus-exec/exec-remote"
      },
      "task:release-skill-pack-1-10-recovery/recovery-4": {
        "key": "task:release-skill-pack-1-10-recovery/recovery-4",
        "receiptId": "9e26f18f080e8f9d830288e4b4656dab9f12e486d5fd031e662c03c35132276e",
        "boundary": "task",
        "subject": "release-skill-pack-1-10-recovery/recovery-4",
        "sourceRevision": 1186,
        "exactSignal": "Starting task 4 out of 6: Integrate unique recovered work onto current origin main"
      },
      "task:repoint probes and cli (doctor.rs, setup.rs, control_transport.rs) to optional contract discovery with no warning when absent; add the doctor test case; update shell tests": {
        "key": "task:repoint probes and cli (doctor.rs, setup.rs, control_transport.rs) to optional contract discovery with no warning when absent; add the doctor test case; update shell tests",
        "receiptId": "ece5b58210a47767658a46552930de5bd3ad697809ad422d457b24ea281e17f0",
        "boundary": "task",
        "subject": "Repoint probes and CLI (doctor.rs, setup.rs, control_transport.rs) to optional contract discovery with no warning when absent; add the doctor test case; update shell tests",
        "sourceRevision": 662,
        "exactSignal": "Starting task 2 out of 2: Repoint probes and CLI (doctor.rs, setup.rs, control_transport.rs) to optional contract discovery with no warning when absent; add the doctor test case; update shell tests"
      },
      "task:update site/docs, docs/codex-plugin.md, and claude.md; regenerate plugin surfaces; run validate:codex and the footprint check": {
        "key": "task:update site/docs, docs/codex-plugin.md, and claude.md; regenerate plugin surfaces; run validate:codex and the footprint check",
        "receiptId": "044fdc5154c4ac030906a3e511cd0e4c82933be4abd80b09fff2bd4ca45ac391",
        "boundary": "task",
        "subject": "Update site/docs, docs/codex-plugin.md, and CLAUDE.md; regenerate plugin surfaces; run validate:codex and the footprint check",
        "sourceRevision": 691,
        "exactSignal": "Starting task 3 out of 3: Update site/docs, docs/codex-plugin.md, and CLAUDE.md; regenerate plugin surfaces; run validate:codex and the footprint check"
      },
      "task:vendor the adopted criteria, english criteria prompt, and a 10-task query subset pinned at upstream commit 469cce54; write attribution.md per apache-2.0": {
        "key": "task:vendor the adopted criteria, english criteria prompt, and a 10-task query subset pinned at upstream commit 469cce54; write attribution.md per apache-2.0",
        "receiptId": "26cd0e8b68314439cc2bafd3585917c64b9ef62290c97fa86c9a733f8ca74bea",
        "boundary": "task",
        "subject": "Vendor the adopted criteria, English criteria prompt, and a 10-task query subset pinned at upstream commit 469cce54; write ATTRIBUTION.md per Apache-2.0",
        "sourceRevision": 1098,
        "exactSignal": "Starting task 1 out of 1: Vendor the adopted criteria, English criteria prompt, and a 10-task query subset pinned at upstream commit 469cce54; write ATTRIBUTION.md per Apache-2.0"
      },
      "task:write research-manifest.schema.json from the spec; make the spec normative (no .research suffix, plan.md, package dir naming); delete the template": {
        "key": "task:write research-manifest.schema.json from the spec; make the spec normative (no .research suffix, plan.md, package dir naming); delete the template",
        "receiptId": "4d58a2fd94f51787bb94306cd24143653a48e32a004e8027135cf9f1ada52c28",
        "boundary": "task",
        "subject": "Write research-manifest.schema.json from the spec; make the spec normative (no .research suffix, plan.md, package dir naming); delete the template",
        "sourceRevision": 702,
        "exactSignal": "Starting task 1 out of 1: Write research-manifest.schema.json from the spec; make the spec normative (no .research suffix, plan.md, package dir naming); delete the template"
      }
    },
    "blockers": [
      {
        "id": "bottleneck-881456002a7dae3690e5e8e9ee250cf091d0500cea8308f55882220b23efcec4",
        "summary": "Task Before boundary i195-03-delivery-qualification/2 blocked: boundary i195-03-delivery-qualification/2 already has an outstanding start receipt",
        "resolved": false,
        "resolution": null
      },
      {
        "id": "issue-195-closeout.scratch-claude-auth",
        "summary": "Mandatory POL-001/002/003 and VER-001/003 remain BLOCKED: authenticate /Users/gqadonis/.prometheus-scratch/kbd194/claude. Preserve the adopted scope, original attempts and budget consumption. #184 and mini implementation have not started.",
        "resolved": false,
        "resolution": null
      }
    ],
    "instruction": "Unknown side effects require reconciliation before retry, not a fresh attempt assumption."
  },
  "preserve": [
    "latest non-superseded method decision and retiredMethods; old commands/memory are context only",
    "original contract snapshots, evidence and source/build/configuration/fixture/assertion identity",
    "all budget consumption/high-water marks, including UNKNOWN prior usage; never reset on recovery",
    "unresolved effects and cleanup/receipt obligations; reconcile uncertain effects before retry"
  ],
  "hostCapabilities": {
    "platform": "darwin",
    "nodeVersion": "v24.11.1",
    "observedInterfaces": [
      "canonical status",
      "typed phase disposition",
      "project-owned file identity"
    ],
    "executionInterception": "not provided by this helper",
    "commandInventoryIsPermission": false
  },
  "control": {
    "mode": "instruction-only",
    "observed": [
      "canonical references and file identities"
    ],
    "enforced": [
      "only named existing signed runtime mutations; no generic shell/tool interception claimed"
    ],
    "unsupported": [
      "universal model obedience or arbitrary tool interception"
    ]
  },
  "status": "RESOLVED_REFERENCES",
  "blockers": []
}
```
