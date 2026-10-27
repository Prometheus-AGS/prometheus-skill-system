# PKG-001 source and ownership manifest

Captured 2026-10-27T11:15:27.093185Z at base HEAD `bfe572ff8a97a21c7610c44ea6b39d5f97a857b7` (working tree, nothing committed).
Static inventory of authored production source only; no build, test or generation has run. SHA-256 is of the working-tree bytes.

## Registration decision

- New skills `value-scenario-discovery` and `value-scenario-verify` live under the recursive `skills/` root; `generate-skill-system-distribution.js` and `generate:skills-index` discover them. No generated inventory (`skill-index.json`, `SKILLS.md`, `dist/`, `.agents/plugins/marketplace.json`) is hand-edited.
- Observation, not changed: `skills/testing/.claude-plugin/plugin.json` lists only `bdd-testing` and `bdd-video-proof`; it already omitted `bdd-cucumber-*` and `bdd-lifecycle-loop`, so it is not an authoritative roster and needs no new entry for PKG-001.
- Generated Claude/Codex payloads are owned by `kbd-194-196-delivery-qualification` task 2 (regeneration twice); compiled team scripts are rebuilt there from the `.mts` sources below.

## kbd-194-change-identity-repair

| File | SHA-256 |
|---|---|
| `substrate/kbd-runtime/src/change_identity.rs` | `ab03dbb829bc5bc90deeb78022f43e905172170e14d3c95a72d2d7ecf2a5f25d` |
| `substrate/kbd-runtime/src/lib.rs` | `c195e7e572c96b05a8c39833ed2f4234883765fc99d8f8b7859d202ba077f273` |
| `substrate/kbd-runtime/src/project_document.rs` | `9125b6be9bf980ecf619838d280c9a8f9159dd5b5575d75d35f41cf3f440e4b6` |
| `tools/prometheus-cli/crates/prometheus-cli/src/main.rs` | `db3fc509b00e7b9d266c4c84446cfb663245abf8651da3b7d995a5f1c3ff545c` |
| `tools/prometheus-cli/crates/prometheus-cli/src/commands/kbd.rs` | `3c9737ffe78eaf7fd19adb94a1b437ba0170a76f4d4c377a9b38a152a2c936e6` |
| `skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh` | `63b791f2d9103b24887e6ed00c5cceedd092a00d36b10f0e557aba837c57840b` |
| `skills/process/kbd-process-orchestrator/skills/kbd-apply/reconcile.mjs` | `51d9622f950bfbaeeda02bbf375d1835b1efc592ac92751ad3ffcd9a3ea862bb` |
| `skills/process/kbd-process-orchestrator/skills/kbd-apply/reconcile-command.mjs` | `6fac7bdb9e9c186b9c2276969b029f9495ceba7d99c3509be29f2a34cf7fc53b` |
| `docs/guide/kbd-change-identity-repair.md` | `4944e7c4738713f35a5fcfb54ee54d2fa1ce15b501a197f0dbff2dac4c75ea5e` |

## kbd-195-vlbv-policy-propagation

| File | SHA-256 |
|---|---|
| `skills/process/kbd-process-orchestrator/shared/method-context.mjs` | `dae3754ae06f4a1b685535cb3be3a1a25d03c4946355034accfae34b479b7968` |
| `skills/process/kbd-process-orchestrator/references/method-context.md` | `fa267bafc905749054aa354825990d612fd97a0b0fe3187cb34b7929eb7dc9a4` |
| `skills/process/agent-team-creator/runtime/src/phase-policy.mts` | `c83051d9dad073dc98766e0645b632f24723bd26f8362aa64049e917103d995d` |
| `skills/process/agent-team-creator/runtime/src/adapters-codecs.mts` | `4d2d70feb507f18374ea192dc0f298f978299acd4d7fc6ddf4bf10921fb91d56` |
| `skills/process/agent-team-creator/runtime/src/handoff-provenance.mts` | `d4f04e61275c3c82904a60b619cb3436c8092da54606d3c9a0510b0b5aa0e2fe` |
| `skills/process/agent-team-creator/runtime/src/handoff.mts` | `c369506d654ac33173b67b2b5493c3991d04b8ca66f8701dd170c56b50c8ea71` |
| `skills/process/prometheus-context-bootstrap/SKILL.md` | `91f2564efd63c1c092ca9fde1c5267dbe5c3f95e06916d7b4b95098eb4edb36c` |
| `skills/process/prometheus-context-bootstrap/assets/hooks/reanchor.sh` | `de98a12efd60eae402d896745bb8d7109d286e94b7125d5f18f25ab433c86a53` |
| `skills/process/prometheus-context-bootstrap/references/AGENTS.base.md` | `d5516c113a8af4e06acc61f88164a714f610855b94379b35f3a5f3ac60f6aea3` |
| `skills/process/prometheus-context-bootstrap/references/rules-flutter.md` | `844a8d8fff7270522050406380d06633541c1b893235fabd3aa4a25620706ad0` |
| `skills/process/prometheus-context-bootstrap/references/rules-go.md` | `67a89cc6760a61052e672afdd02f8f499acd7e0696984dddcc892853e437c525` |
| `skills/process/prometheus-context-bootstrap/references/rules-python.md` | `3f10da91800f5ee68a2a6325d51a21d7f94e1a29aad4ae9ff30d0a8bb5fccbf8` |
| `skills/process/prometheus-context-bootstrap/references/rules-rust.md` | `fa9ef9d1887026dd7b3659a2ff3c86247fbf6c6093c78199af1c1fb2e57f0f91` |
| `skills/process/prometheus-context-bootstrap/references/rules-typescript.md` | `4f1a5a9cacb4cfdc4ae89d2d3ce04cc55642178b741978a2a2ffb196d0b1d4ac` |
| `skills/process/kbd-process-orchestrator/hooks/hooks.json` | `ed5b8f2d3e1f17945a88d0b32375898c2424977d362792ce64ceba40a1c80245` |
| `skills/process/kbd-process-orchestrator/skills/kbd-handoff/SKILL.md` | `9d1a14029c0884477e43bded16efb7a68be830b577957a0c6b13d29167b9c775` |
| `skills/process/kbd-process-orchestrator/skills/kbd-handoff/kbd-handoff.sh` | `37c604c8f9b7bc311a27fab6c91826f733f0696e41e47ef3e3e30b97f37431a9` |
| `skills/process/kbd-process-orchestrator/skills/kbd-child-exit/SKILL.md` | `db32d6333b126c3d170cda94301f1caca9d681577ac1561a0d3868f22d8b648c` |
| `skills/process/kbd-process-orchestrator/skills/kbd-new-child/SKILL.md` | `0b2c1f742a09b38ee8aba53e5bcf693eefd760b57dfad87f94b472d77feb81ba` |
| `skills/process/kbd-process-orchestrator/skills/kbd-next-child/SKILL.md` | `302cbd9ae9597f9e81c0709fac8f6ddc1b86ee0020be84ede84475e9eb0310fc` |
| `skills/process/kbd-process-orchestrator/skills/kbd-pause/SKILL.md` | `8c17ff24f2ce8623881188421aa2b9e25174eb5b5c6d2e77762a66b721b82621` |
| `skills/process/kbd-process-orchestrator/skills/kbd-resume/SKILL.md` | `7e1f3e5d00ce5fcad81c6f499cd5ae007efae0ec102f3360503370405311ece2` |
| `skills/process/kbd-process-orchestrator/skills/kbd-execute/SKILL.md` | `db4a33f356807fbe05be80e5c93a8f3b12470c93d176c3b56542231ccad0b635` |
| `skills/process/kbd-process-orchestrator/skills/kbd-init/SKILL.md` | `1c17e32d0f69a079b257031021e3ee4d4ecad7aa3d14ace044157895827caa76` |
| `skills/process/delivery-cadence/SKILL.md` | `11dfd5aea74da8a63d34079233988466e5e5dce9a360a66d11aab138e3599539` |
| `docs/guide/kbd-selected-method-recovery.md` | `f9aae49fca870117e835f994a1cd8d1839b30291ec37c2920d9d872d0b5d07e7` |

## kbd-196-value-scenario-workflow

| File | SHA-256 |
|---|---|
| `skills/testing/value-scenario-discovery/SKILL.md` | `0de0f67f7b0a485849718dc530fcd65e8b4fccea6148b94711f48512066ef050` |
| `skills/testing/value-scenario-verify/SKILL.md` | `311359ffa341c64031afbdaa229e94cddeaacf70f5c00f458ad4e62460c3e161` |
| `skills/testing/bdd-cucumber-js/SKILL.md` | `f57d541196fa3d434562f9e0c7fcfff80ea7d87a5604e1023bbe75b052530fe7` |
| `skills/testing/bdd-cucumber-js/references/author-only.md` | `1dcdb9291b2137b41773f14fcf09b450a281aa69f56aee03aedb6ba146f8fa13` |
| `skills/testing/bdd-cucumber-rs/SKILL.md` | `4cb9bd167656546ab15645521e55efa2da5f06dd861081241cb00767a704e9c2` |
| `skills/testing/bdd-cucumber-rs/references/author-only.md` | `1dcdb9291b2137b41773f14fcf09b450a281aa69f56aee03aedb6ba146f8fa13` |
| `skills/testing/bdd-lifecycle-loop/SKILL.md` | `960ec2462aabc4eb4b681636358d00463162d9c25e1e85622e97858d4587a4cf` |
| `skills/process/kbd-process-orchestrator/references/integrations/bdd-testing.md` | `d96028ccb8c4669c531540536669b23be4e842e9988414f822c5d64ec3a1dd88` |
| `skills/process/kbd-process-orchestrator/references/architecture.md` | `acdd0eed9b7cb7e8a1afd14274bf57d69933e6b98c3a5c995fd97b96a39cea56` |
| `skills/process/kbd-process-orchestrator/skills/kbd-assess/SKILL.md` | `0797482bfecbf0c5c4da8a0e98d8405af4d3441bd2b1f648f8c5eeb1faccc883` |
| `skills/process/kbd-process-orchestrator/skills/kbd-spec/SKILL.md` | `86665648b598d3e7dc2dd8c698a1d88893ebd75c1d65e7762002ffabd04a08bf` |
| `skills/process/kbd-process-orchestrator/skills/kbd-plan/SKILL.md` | `919bd482c82a602b1f758d18351fdefa48725d0e2ab1934c94e68b26fec7fad3` |
| `docs/guide/value-scenario-workflow.md` | `3fbfe864b3e070b3b4e007ea25290199d9add08ecb8d0fb90c538e47dabd4144` |
