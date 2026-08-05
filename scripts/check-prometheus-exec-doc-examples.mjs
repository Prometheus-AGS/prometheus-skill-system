import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const exampleRoot = path.join(repoRoot, 'examples', 'prometheus-exec', 'tier-p');
const docsRoot = path.join(repoRoot, 'site', 'docs', 'execution');
const failures = [];

const read = file => fs.readFileSync(file, 'utf8');
const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, { encoding: 'utf8', ...options });
  if (result.status !== 0) {
    failures.push(`${command} ${args.join(' ')} failed: ${result.stderr || result.stdout}`);
  }
  return result;
};

const requiredDocs = [
  'overview-and-use-cases.md',
  'choosing-the-right-capability.md',
  'closed-loop-architecture.md',
  'generating-programs.md',
  'architecture-and-tiers.md',
  'tier-p-native-processes.md',
  'tier-w-portable-components.md',
  'local-api-cli-and-mcp.md',
  'remote-dispatch-and-reconciliation.md',
  'receipts-verification-and-certification.md',
  'use-case-cookbook.md',
  'security-and-trust.md',
  'installation-doctor-and-recovery.md',
  'platform-and-evidence-status.md',
];

for (const name of requiredDocs) {
  if (!fs.existsSync(path.join(docsRoot, name)))
    failures.push(`missing Dynamic Operations doc ${name}`);
}

const corpus = requiredDocs
  .filter(name => fs.existsSync(path.join(docsRoot, name)))
  .map(name => read(path.join(docsRoot, name)))
  .join('\n');

for (const required of [
  'Dynamic Operations',
  'native-agent',
  'explicit adapter',
  'prometheus:component@0.1.0',
  'wasm32-unknown-unknown',
  'Python, Node, or Bash',
]) {
  if (!corpus.includes(required))
    failures.push(`Dynamic Operations docs miss required boundary: ${required}`);
}
if (!/does \*\*not generate code\*\*/.test(corpus)) {
  failures.push('Dynamic Operations docs do not state that Exec does not generate code');
}
if (!/does \*\*not run arbitrary native executables\*\*/.test(corpus)) {
  failures.push('Dynamic Operations docs do not state the native executable boundary');
}

for (const [pattern, message] of [
  [
    /Prometheus Exec (?:generates|writes|creates) (?:the )?(?:code|program)/i,
    'claims Exec generates code',
  ],
  [/Tier P (?:runs|accepts) arbitrary native/i, 'claims Tier P runs arbitrary native binaries'],
  [/LibreFang[^\n]{0,100}(?:is|as) (?:a )?Tier W/i, 'conflates LibreFang and Tier W ABIs'],
  [/Windows Tier P[^\n]{0,80}\b(?:supported|certified|available)\b/i, 'overstates Windows Tier P'],
  [/Linux Tier P[^\n]{0,80}runtime-certified/i, 'overstates Linux Tier P runtime evidence'],
  [
    /native-agent[^\n]{0,100}(?:automatically|out of the box)[^\n]{0,60}(?:Exec|execution)/i,
    'claims automatic native-agent integration',
  ],
]) {
  if (pattern.test(corpus)) failures.push(message);
}

const mermaidCount = [...corpus.matchAll(/^```mermaid\s*$/gm)].length;
if (mermaidCount < 7)
  failures.push(`expected at least 7 Dynamic Operations diagrams, found ${mermaidCount}`);

const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'prometheus-exec-docs-'));
try {
  const inputDir = path.join(temporary, 'inputs');
  const outputDir = path.join(temporary, 'outputs');
  fs.mkdirSync(inputDir);
  fs.mkdirSync(outputDir);
  fs.copyFileSync(path.join(exampleRoot, 'records.json'), path.join(inputDir, 'records'));

  const exampleEnvironment = {
    ...process.env,
    PROMETHEUS_INPUT_DIR: inputDir,
    PROMETHEUS_OUTPUT_DIR: outputDir,
  };
  const python = run('python3', [path.join(exampleRoot, 'transform.py')], {
    env: exampleEnvironment,
  });
  if (python.status === 0) {
    assert.deepEqual(
      JSON.parse(python.stdout),
      JSON.parse(read(path.join(exampleRoot, 'expected-summary.json')))
    );
    assert.deepEqual(
      JSON.parse(read(path.join(outputDir, 'summary.json'))),
      JSON.parse(read(path.join(exampleRoot, 'expected-summary.json')))
    );
  }

  fs.rmSync(outputDir, { recursive: true });
  fs.mkdirSync(outputDir);
  const node = run('node', [path.join(exampleRoot, 'transform.mjs')], { env: exampleEnvironment });
  if (node.status === 0) {
    assert.deepEqual(
      JSON.parse(node.stdout),
      JSON.parse(read(path.join(exampleRoot, 'expected-summary.json')))
    );
  }

  fs.copyFileSync(path.join(exampleRoot, 'numbers.txt'), path.join(inputDir, 'numbers'));
  fs.rmSync(outputDir, { recursive: true });
  fs.mkdirSync(outputDir);
  const bash = run('bash', [path.join(exampleRoot, 'transform.sh')], { env: exampleEnvironment });
  if (bash.status === 0) {
    assert.equal(bash.stdout, read(path.join(exampleRoot, 'expected-total.txt')));
    assert.equal(
      read(path.join(outputDir, 'total.txt')),
      read(path.join(exampleRoot, 'expected-total.txt'))
    );
  }
} catch (error) {
  failures.push(`example output validation failed: ${error.message}`);
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}

const homepage = read(path.join(repoRoot, 'site', 'src', 'pages', 'index.js'));
const config = read(path.join(repoRoot, 'site', 'docusaurus.config.js'));
if (!homepage.includes('Turn generated code into verifiable work.')) {
  failures.push('homepage misses the featured Dynamic Operations treatment');
}
if (!config.includes("label: 'Dynamic Operations'")) {
  failures.push('navbar/footer config misses Dynamic Operations');
}

if (failures.length) {
  console.error(failures.map(failure => `Dynamic Operations docs: ${failure}`).join('\n'));
  process.exit(1);
}

console.log(
  `Dynamic Operations documentation and ${mermaidCount} relationship diagrams are valid.`
);
