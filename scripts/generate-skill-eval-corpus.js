#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const definitions = [
  {
    skill: 'kbd-pause',
    command: 'kbd_pause',
    positives: [
      'Use /kbd-pause because I need to inspect an architectural deviation.',
      'Stop mutations, preserve the exact next task, and let me audit the run.',
      'Pause this phase before the agent makes another edit.',
      'Create a durable checkpoint so I can change course from another harness.',
    ],
    near: ['Pause the video at 01:20.', 'Explain what a paused operating-system process means.'],
  },
  {
    skill: 'kbd-resume',
    command: 'kbd_resume',
    positives: [
      'Use /kbd-resume at plan revision 3.',
      'Resume the paused KBD run from its committed next task.',
      'Continue execution only after validating the revised plan.',
      'Claim the writer lease and resume this suspended phase.',
    ],
    near: ['Resume the download after the network reconnects.', 'Improve this résumé summary.'],
  },
  {
    skill: 'kbd-cancel',
    command: 'kbd_cancel',
    positives: [
      'Use /kbd-cancel because this run should terminate gracefully.',
      'Cancel the KBD phase and preserve its audit history.',
      'Stop this workflow permanently; do not resume it.',
      'Record an operator cancellation before cleanup.',
    ],
    near: ['Cancel the restaurant reservation.', 'How does task cancellation work in Tokio?'],
  },
  {
    skill: 'kbd-audit',
    command: 'kbd_events',
    positives: [
      'Use /kbd-audit and show changes since revision 12.',
      'Audit the decisions and blockers before I revise the plan.',
      'Show exactly where the KBD run is without mutating it.',
      'Inspect the committed causal history for this phase.',
    ],
    near: ['Audit this CSS file for accessibility.', 'What does a financial auditor do?'],
  },
  {
    skill: 'kbd-handoff',
    command: 'kbd_handoff',
    positives: [
      'Use /kbd-handoff to transfer the writer lease to Claude Code.',
      'Hand this paused run from Codex to OpenCode at the same revision.',
      'Move execution ownership to Kimi without an unleased write window.',
      'Atomically release this harness and claim the next one.',
    ],
    near: ['Write a handoff note for the support team.', 'Explain a relay-race baton handoff.'],
  },
  {
    skill: 'kbd-process-orchestrator',
    command: 'kbd_status',
    positives: [
      'Use /kbd-process-orchestrator to continue the current phase safely.',
      'Determine the active KBD task and execute only its recorded next work.',
      'Coordinate this phase across assessment, plan, execution, and reflection.',
      'Reanchor after compaction and follow the committed lifecycle.',
    ],
    near: ['Orchestrate these Kubernetes containers.', 'Recommend an orchestral music recording.'],
  },
];

const cases = [];
for (const definition of definitions) {
  definition.positives.forEach((prompt, index) => {
    cases.push({
      id: `${definition.skill}-positive-${index + 1}`,
      skill: definition.skill,
      kind: index === 0 ? 'explicit' : 'implicit',
      prompt,
      expectedInvocation: true,
      expectedCommands: [definition.command],
      forbidDirectWrites: true,
    });
  });
  definition.near.forEach((prompt, index) => {
    cases.push({
      id: `${definition.skill}-near-miss-${index + 1}`,
      skill: definition.skill,
      kind: 'near_miss',
      prompt,
      expectedInvocation: false,
      expectedCommands: [],
      forbidDirectWrites: true,
    });
  });
}

const corpus = {
  schemaVersion: '1',
  trialsPerHarness: 3,
  criticalSkills: definitions.map(definition => definition.skill),
  cases,
};
if (cases.length !== 36) throw new Error(`expected 36 cases, got ${cases.length}`);
const output = path.join(root, 'evals/skill-activation');
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(path.join(output, 'critical-36.json'), `${JSON.stringify(corpus, null, 2)}\n`);
console.log(`Generated ${cases.length} critical skill evaluation prompts.`);
