import { mkdir, writeFile, readFile, readdir } from 'node:fs/promises';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { normalizeUiuxFrontmatter, frontmatterAdaptation } from './uiux-frontmatter.mjs';

// Explicit maintenance-time importer. Installed skills never fetch their runtime.
const full = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const miniArg = process.argv.indexOf('--mini');
const mini = miniArg < 0 ? undefined : resolve(process.argv[miniArg + 1]);
const sha = data => createHash('sha256').update(data).digest('hex');
const sources = [
  ['Leonxlnx/taste-skill', 'c184364c58658b2f131b4ae8bd3d206cabb3deee', 'MIT', 'LICENSE', [
    'taste-skill', 'taste-skill-v1', 'gpt-tasteskill', 'redesign-skill', 'soft-skill', 'minimalist-skill', 'brutalist-skill', 'output-skill', 'stitch-skill'].map(x => `skills/${x}`)],
  ['jakubkrehel/skills', '267330e1adfc66a718fb65fa6918c1f06d0a689e', 'MIT', 'LICENSE', [
    'better-accessibility', 'better-colors', 'better-interface', 'better-layout', 'better-typography', 'better-ui', 'better-writing', 'break', 'explain-interface', 'interface-review', 'variant'].map(x => `skills/${x}`)],
  ['vercel-labs/agent-skills', '063bee94c3f4df8453406c830b0a7df0f2860278', 'MIT', 'README.md', [
    'react-best-practices', 'composition-patterns', 'react-native-skills', 'web-design-guidelines'].map(x => `skills/${x}`)],
  ['expo/skills', 'efa52f0a9d2176db75992736281c77da1b714fa3', 'MIT', 'LICENSE', [
    'expo-native-ui', 'expo-ui', 'expo-design-system', 'expo-router', 'expo-animation'].map(x => `plugins/expo/skills/${x}`)],
  ['flutter/agent-plugins', '3f58a5512a03c854b6c8b1128d6711e5e2d5055b', 'BSD-3-Clause', 'LICENSE', [
    'flutter-build-responsive-layout', 'flutter-fix-layout-issues', 'flutter-setup-declarative-routing', 'flutter-setup-localization'].map(x => `skills/${x}`)],
  ['twostraws/swiftui-agent-skill', 'be297ff80dddec529af1f9b1f1f114aab6c9d11c', 'MIT', 'LICENSE', ['swiftui-pro']],
  ['android/skills', '42dc2270e96032bd860bb94511e440aa00a43125', 'Apache-2.0', 'LICENSE.txt', ['jetpack-compose/adaptive', 'system/edge-to-edge']],
  ['pbakaus/impeccable', '9d715cc4f5564a990ca8345abfdd5df6dc9b41c8', 'Apache-2.0', 'LICENSE', ['.agents/skills/impeccable']],
];
async function remote(repo, revision, path) {
  const response = await fetch(`https://raw.githubusercontent.com/${repo}/${revision}/${path}`);
  if (!response.ok) throw new Error(`${response.status}: ${repo}/${path}`);
  return Buffer.from(await response.arrayBuffer());
}
async function put(root, path, data) {
  const target = join(root, path);
  await mkdir(dirname(target), { recursive: true });
  await writeFile(target, data);
}
function policy(id) {
  return `\n## Prometheus distribution contract\n\nThis pinned upstream guidance is subordinate to the project's approved scope, design authority, framework versions, tool permissions, and phase boundary. Read PRODUCT.md, DESIGN.md, existing tokens and the affected application manifest before choosing advice. Apply relevant rules only; do not migrate frameworks or replace incumbent design by default.\n\nComplete the implementation phase before a single consolidated verification pass; allow one batched correction and confirmation. Upstream per-edit tests, detector hooks and repetitive audit loops do not apply. Native platform execution requires its actual SDK/toolchain; portable guidance does not imply Windows can build or profile Apple applications. Never install a dependency, fetch a runtime, submit feedback, or message a third party merely because upstream suggests it.\n\nUse the installed prometheus-ui-ux protocol for automatic selection. Taste guidance is limited to new surfaces or authorized redesign, one implementation plus at most one explicit overlay. No taste guidance applies to refinement or review. Existing DESIGN.md remains authoritative; generated recommendations and DESIGN.stitch.md cannot overwrite it without explicit authorization. User-only skills remain user-only in every harness; reading their source is not an invocation workaround.\n\n${id === 'swiftui-pro' ? 'Use the project-pinned Swift and iOS versions; upstream newest-version defaults are conditional.\n' : ''}${id.startsWith('expo-') ? 'Inspect the actual Expo SDK version first. Guidance requiring SDK 56+ applies only when the project already satisfies it; preserve existing UI libraries unless migration is requested.\n' : ''}${id.startsWith('android-') ? 'Check the declared Compose and navigation dependencies before applying adaptive or edge-to-edge examples; do not silently adopt Navigation 3.\n' : ''}`;
}
function adaptMarkdown(text, id, adaptations) {
  if (id === 'gpt-taste') {
    text = text.replace('## 1. PYTHON-DRIVEN TRUE RANDOMIZATION (BREAKING THE LOOP)', '## 1. DETERMINISTIC LAYOUT SELECTION')
      .replace('LLMs are inherently lazy and always pick the first layout option. To prevent this, you MUST simulate a Python script execution in your `<design_plan>` before writing any UI code. ', 'Before writing UI code, record deterministic layout choices in your `<design_plan>`. Use transparent arithmetic or an actually executed Node.js script; do not invent execution output. ')
      .replace('Use a deterministic seed (e.g., character count of the user prompt modulo math) to simulate `random.choice()` and strictly select:', 'Use a deterministic seed such as the prompt character count. For each candidate list, compute `seed % candidates.length` using JavaScript arithmetic or explain the same calculation, then select:')
      .replace('**Python RNG Execution:** Write a 3-line mock Python output showing the deterministic selection of your Hero Layout, Component Arsenal, GSAP animations, and Fonts based on the prompt\'s character count.', '**Deterministic Selection Record:** Record the seed, candidate-list indices, and selected Hero Layout, Component Arsenal, animations, and Fonts. Label this as a planning calculation; claim Node.js execution only if it actually ran.');
    adaptations.add('Replaced simulated Python/random-choice instructions and fabricated execution output with deterministic JavaScript arithmetic or an explicitly identified planning calculation.');
  }
  if (text.includes('npx chrome-devtools-mcp@latest')) {
    text = text.replace(/```(?:bash|sh)?\nclaude mcp add chrome-devtools[^`]+```/g, '> Use an already configured browser capability. Installing a browser MCP server is a separate explicit setup task; this skill does not download one.');
    adaptations.add('Removed automatic browser-MCP installation; use only an existing harness browser capability.');
  }
  if (text.includes('shadcn@latest')) {
    text = text.replace(/`npx shadcn@latest add \.\.\.`/g, 'project-pinned component tooling').replace(/npx shadcn@latest init\n(?:npx shadcn@latest add[^\n]+)?/g, '# Use the project-pinned component tooling only when dependency changes are authorized.');
    adaptations.add('Replaced latest-version component-generator commands with project-pinned, explicitly authorized tooling.');
  }
  if (id === 'stitch-design-taste') {
    text = text.replaceAll('DESIGN.md', 'DESIGN.stitch.md');
    adaptations.add('Renamed Stitch output and bundled example to DESIGN.stitch.md so project DESIGN.md stays authoritative.');
  }
  if (/\bgrep\b/.test(text)) {
    text = text.replace(/grep it there/g, 'search for it there with the harness file tools').replace(/grep its class name/g, 'search for its class name with the harness file tools').replace(/candidate grep checks/g, 'candidate source-search checks').replace(/no grep precise enough/g, 'no source search precise enough');
    adaptations.add('Reworded Unix-specific prose discovery instructions to harness file searches.');
  }
  if (id.startsWith('expo-')) {
    const index = text.indexOf('## Submitting Feedback');
    if (index >= 0) { text = text.slice(0, index); adaptations.add('Removed unsolicited upstream feedback submission workflow.'); }
    text = text.replace(/^allowed-tools:.*\n/gm, '');
  }
  if (id === 'web-design-guidelines') {
    text = text.replace(/Fetch the latest guidelines from the source URL below/g, 'Read the bundled references/web-interface-guidelines.md')
      .replace(/Fetch fresh guidelines before each review:/g, 'The pinned offline guidelines are in references/web-interface-guidelines.md. Upstream source:')
      .replace(/Use WebFetch[^\n]+/g, 'Read the bundled reference; do not fetch at runtime.')
      .replace(/Fetch guidelines from the source URL above/g, 'Read references/web-interface-guidelines.md');
    adaptations.add('Pinned external Web Interface Guidelines locally; removed required runtime fetch.');
  }
  // Shell-oriented discovery examples are documentation, not a portable dependency.
  text = text.replace(/```(?:bash|sh|shell|zsh)?\n([\s\S]*?)```/g, (block, body) => {
    if (/\b(?:grep|sed|awk|curl|wget|mktemp|chmod|jq|python3?|bash)\b|\bfind\s|\bhead\s|\btail\s/.test(body)) {
      adaptations.add('Replaced Unix/Python command examples with portable file-inspection instructions.');
      return '> Inspect the indicated project files using the harness file tools or Node.js filesystem APIs. This upstream shell example is omitted in the portable distribution.';
    }
    return block;
  });
  return text;
}
const catalog = { schemaVersion: 1, sources: [], skills: [], deferredPorts: [] };
for (const [repo, revision, license, licensePath, selections] of sources) {
  const treeResponse = await fetch(`https://api.github.com/repos/${repo}/git/trees/${revision}?recursive=1`);
  if (!treeResponse.ok) throw new Error(`Tree lookup failed: ${repo}`);
  const tree = (await treeResponse.json()).tree;
  const notice = await remote(repo, revision, licensePath);
  catalog.sources.push({ repository: `https://github.com/${repo}`, revision, license, evidence: licensePath });
  for (const selection of selections) {
    const skill = (await remote(repo, revision, `${selection}/SKILL.md`)).toString();
    const upstreamName = skill.match(/^name:\s*(.+)$/m)[1].trim().replace(/^['"]|['"]$/g, '');
    const id = repo === 'android/skills' ? (upstreamName === 'adaptive' ? 'android-compose-adaptive' : 'android-edge-to-edge') : upstreamName;
    const root = join(full, 'skills/ui-ux', id);
    const miniRoot = mini && id !== 'impeccable' ? join(mini, 'skills', id) : undefined;
    const adaptations = new Set(['Prepended explicit project authority, scope, version, phase-boundary and invocation contract.']);
    if (id !== upstreamName) adaptations.add(`Aliased upstream ${upstreamName} to ${id} to avoid ambiguous skill identity.`);
    const assets = [];
    const files = tree.filter(x => x.type === 'blob' && x.path.startsWith(`${selection}/`) && !x.path.slice(selection.length + 1).startsWith('skills/'));
    for (const file of files) {
      let path = file.path.slice(selection.length + 1);
      if (path.startsWith('.claude-plugin/')) {
        adaptations.add('Omitted repository plugin wrapper pointing to a nested skills tree; distribute the standalone skill and its referenced assets.');
        continue;
      }
      if (id === 'stitch-design-taste' && path === 'DESIGN.md') path = 'DESIGN.stitch.md';
      if (id === 'impeccable' && ['scripts/impeccable', 'scripts/impeccable.cmd'].includes(path)) continue;
      if (/\.(?:sh|py|exe|dll|dylib|so)$/.test(path)) {
        adaptations.add(`Omitted optional non-portable helper ${path}; no equivalent runtime advertised.`); continue;
      }
      let bytes = await remote(repo, revision, file.path);
      const upstreamSha256 = sha(bytes);
      if (/\.md$/.test(path)) {
        let text = adaptMarkdown(bytes.toString(), id, adaptations);
        if (path === 'SKILL.md') {
          text = normalizeUiuxFrontmatter(text, license, id);
          adaptations.add(frontmatterAdaptation);
          text = text.replace(/^name:.*$/m, `name: ${id}`);
          const end = text.indexOf('\n---', 4) + 4;
          text = text.slice(0, end) + '\n' + policy(id) + '\n' + text.slice(end);
          if (id === 'design-taste-frontend') text = text.replace('## Prometheus distribution contract', 'Upstream identity: design-taste-frontend v2 (experimental).\n\n## Prometheus distribution contract');
        }
        if (id === 'impeccable') {
          text = text.replace(/(?:<skill-base-dir>|\.agents\/skills\/impeccable)\/scripts\/impeccable(?:\.cmd)?/g, 'node <skill-base-dir>/scripts/engine.mjs');
          text = text.replace(/The launcher runs a self-contained binary[^\n]+/g, 'The local Node launcher requires an explicitly configured preinstalled native engine (IMPECCABLE_BIN); it never downloads one. If absent, load existing project context directly and use the permitted guidance.');
        }
        bytes = Buffer.from(text);
      }
      if (id === 'expo-ui' && path === 'scripts/list-components.js') {
        path = 'scripts/list-components.mjs';
        bytes = Buffer.from(bytes.toString().replace(/^#!.*\n/, '').replace("const fs = require('fs');", "import fs from 'node:fs';").replace("const path = require('path');", "import path from 'node:path';"));
        adaptations.add('Converted the optional local component inspector from CommonJS .js to explicit Node ESM .mjs; no new dependencies.');
      }
      if (/\.md$/.test(path)) bytes = Buffer.from(bytes.toString().replaceAll('list-components.js', 'list-components.mjs'));
      await put(root, path, bytes);
      if (miniRoot) await put(miniRoot, path, bytes);
      assets.push({ path, sha256: sha(bytes), upstreamSha256 });
    }
    const preferredNotice = licensePath === 'README.md' ? 'UPSTREAM-LICENSE-EVIDENCE.md' : licensePath;
    const noticeName = assets.some(asset => asset.path === preferredNotice) ? `${preferredNotice}.repository` : preferredNotice;
    if (noticeName !== preferredNotice) adaptations.add(`Preserved the skill-specific ${preferredNotice}; repository-wide notice is separately retained as ${noticeName}.`);
    await put(root, noticeName, notice);
    if (miniRoot) await put(miniRoot, noticeName, notice);
    assets.push({ path: noticeName, sha256: sha(notice) });
    const record = { id, upstreamName, source: `https://github.com/${repo}/tree/${revision}/${selection}`, revision, license,
      full: { path: `skills/ui-ux/${id}`, status: id === 'impeccable' ? 'native-engine-optional' : 'bundled' },
      mini: { path: miniRoot ? `skills/${id}` : null, status: miniRoot ? 'portable-guidance' : 'excluded-native-engine' },
      invocation: /disable-model-invocation:\s*true/.test(skill) ? 'user-only' : 'automatic',
      assets, adaptations: [...adaptations] };
    if (id === 'design-taste-frontend') record.experimental = true;
    catalog.skills.push(record);
    console.log(`Bundled ${id}: ${assets.length} assets`);
  }
}
const guidelinesRevision = 'e3d624baaf29dc1fc645aff3e38f03e564d2d6b1';
for (const [source, target] of [['command.md', 'references/web-interface-guidelines.md'], ['LICENSE', 'references/LICENSE.web-interface-guidelines']]) {
  const bytes = await remote('vercel-labs/web-interface-guidelines', guidelinesRevision, source);
  await put(join(full, 'skills/ui-ux/web-design-guidelines'), target, bytes);
  if (mini) await put(join(mini, 'skills/web-design-guidelines'), target, bytes);
  catalog.skills.find(x => x.id === 'web-design-guidelines').assets.push({ path: target, sha256: sha(bytes) });
}
catalog.sources.push({ repository: 'https://github.com/vercel-labs/web-interface-guidelines', revision: guidelinesRevision, license: 'MIT', evidence: 'LICENSE' });
catalog.deferredPorts.push({ capability: 'Impeccable native detector and live browser engine', dependency: 'Upstream native engine, platform binary, browser instrumentation and post-edit hooks', miniStatus: 'excluded; prometheus-impeccable-core is a bounded adaptation only', proposedNodeInterface: 'node scripts/impeccable-engine.mjs detect --input recorded-dom-style.json --design DESIGN.md; live integration delegates to an existing harness browser capability', retainedBehavior: ['Context and incumbent surface resolution', 'Mode selection', 'Selective skill routing', 'Completed-phase review hook'], omittedBehavior: ['Native detector execution', 'Native engine-managed browser session and injection', 'Per-edit detector hooks', 'Runtime binary downloads'], parityAcceptance: ['Pinned recorded DOM/style fixture parity for supported detector rules', 'Stable rule IDs, ignore behavior and actionable evidence', 'Existing harness browser permission preservation', 'Offline Windows Node execution with no native engine or extra daemon', 'One phase-boundary inspection and at most one confirmation'] });
const engine = `import { spawnSync } from 'node:child_process';\nimport { dirname, resolve } from 'node:path';\nimport { fileURLToPath } from 'node:url';\nconst binary = process.env.IMPECCABLE_BIN;\nif (!binary) { console.error('Impeccable native engine is not configured. Read project PRODUCT.md and DESIGN.md directly. Set IMPECCABLE_BIN only to an explicitly installed engine; this launcher never downloads a runtime.'); process.exit(127); }\nconst args = process.argv.slice(2);\nif (args[0] === 'hooks' && args[1] === 'on') { console.error('Prometheus runs verification only at the complete phase boundary; per-edit native hooks are not installed.'); process.exit(2); }\nconst result = spawnSync(binary, args, { stdio: 'inherit', shell: false, env: { ...process.env, IMPECCABLE_SKILL_DIR: resolve(dirname(fileURLToPath(import.meta.url)), '..'), IMPECCABLE_SELF: 'node <skill-base-dir>/scripts/engine.mjs' } });\nif (result.error) console.error(result.error.message);\nprocess.exit(result.status ?? 1);\n`;
await put(join(full, 'skills/ui-ux/impeccable'), 'scripts/engine.mjs', engine);
catalog.skills.find(x => x.id === 'impeccable').assets.push({ path: 'scripts/engine.mjs', sha256: sha(engine) });
catalog.skills.find(x => x.id === 'impeccable').adaptations.push('Replaced platform shell launchers with local-only Node native-engine invocation; no automatic download, install, or per-edit hook. Native engine is an explicit external prerequisite.');
for (const record of catalog.skills) {
  const provenance = `# Provenance: ${record.id}\n\nSource: ${record.source}\n\nImmutable revision: ${record.revision}\n\nLicense: ${record.license}. Preserve the bundled upstream notice.\n\nInvocation: ${record.invocation}.\n\n## Adaptations\n\n${record.adaptations.map(x => `- ${x}`).join('\n')}\n\nFull: ${record.full.status}. Mini: ${record.mini.status}.\n\nSee the collection catalog.lock.json for source and bundled asset hashes. This records provenance, not execution certification.\n`;
  await put(join(full, record.full.path), 'PROVENANCE.md', provenance);
  if (mini && record.mini.path) await put(join(mini, record.mini.path), 'PROVENANCE.md', provenance);
}
await put(full, 'skills/ui-ux/catalog.lock.json', JSON.stringify(catalog, null, 2) + '\n');
if (mini) await put(mini, 'references/ui-ux/catalog.lock.json', JSON.stringify(catalog, null, 2) + '\n');
