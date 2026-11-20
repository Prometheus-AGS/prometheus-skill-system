import fs from 'node:fs';
import path from 'node:path';
import { createHash, randomBytes } from 'node:crypto';
import { spawnSync } from 'node:child_process';

const names = ['AGENTS.md', 'CLAUDE.md'];
const heading = '## Goals-first development';
const axiom = 'Always think in terms of user goals and value—not in tests or assumptions.';
const policy = [
  heading, '', axiom, '',
  '- In KBD Assess → Analyze → Plan → Execute → Reflect, name the actual user or operator, goal, value, real inputs and conditions, and observable outcome. Trace the real workflow and source of truth first. Record the rule, expected outcome, evidence, exceptions and unresolved assumptions in the accepted specification or behavior contract.',
  '- Verify meaningful reachable outcomes through scoped phase-end VLBV/Gherkin in the existing approved process. Exercise the actual user-facing workflow, including the browser when relevant. Preserve established build, verification and release processes and signed gates. Do not invent unrelated checks or wait for a full suite unless required or warranted by material risk.',
  '- Done means the agreed user outcome demonstrably works, with relevant VLBV evidence, user access or guidance, honest limitations and reflection. Review-ready is not done, merged or deployed. Finish the authorized handoff at the approved boundary, without inventing a new gate or blocking on unrelated nits.', '',
].join('\n');

class Usage extends Error {}
const reject = (message) => { throw new Error(message); };
const sha = (text) => createHash('sha256').update(text).digest('hex');

function parse(args) {
  let root, mode, digest;
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--path' && root === undefined && args[i + 1] && !args[i + 1].startsWith('--')) root = args[++i];
    else if (!mode && ['--audit', '--preview', '--apply'].includes(arg)) {
      mode = arg.slice(2);
      if (mode === 'apply') digest = args[++i];
    } else throw new Usage('use --path <Git root> and exactly one of --audit, --preview, --apply <sha256>');
  }
  if (!root || !mode || (mode === 'apply' && !/^[a-f0-9]{64}$/.test(digest ?? ''))) {
    throw new Usage('use --path <Git root> and exactly one of --audit, --preview, --apply <sha256>');
  }
  return { root: fs.realpathSync(root), mode, digest };
}

function git(root, args, codes = [0]) {
  // Keep Git scoped to this working tree; caller environment cannot select another index/worktree.
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith('GIT_')));
  // check-ignore rejects pathspec magic; its inputs are the two fixed names above.
  const literal = args[0] === 'check-ignore' ? [] : ['--literal-pathspecs'];
  const r = spawnSync('git', ['--no-optional-locks', ...literal, '-C', root, ...args], {
    encoding: 'utf8', env, windowsHide: true,
  });
  if (r.error || !codes.includes(r.status)) reject('Git could not inspect the exact project root or named instructions');
  return r;
}

function readEntry(root, name) {
  const file = path.join(root, name);
  const index = git(root, ['ls-files', '--stage', '--', name]).stdout.trim();
  if (index.includes('\n') || (index && !/^(100644|100755|120000) [0-9a-f]+ 0\t/.test(index))) {
    reject(name + ': unresolved index or unsupported Git entry');
  }
  const ignored = !index && git(root, ['check-ignore', '--quiet', '--', name], [0, 1]).status === 0;
  if (ignored) reject(name + ': ignored local instruction; resolve portability before automatic update');
  let stat;
  try { stat = fs.lstatSync(file); } catch (error) {
    if (error.code !== 'ENOENT') throw error;
    // An index deletion can hide a HEAD-tracked entry. Never undo either form of deliberate removal.
    const head = git(root, ['rev-parse', '--verify', 'HEAD'], [0, 128]).stdout.trim();
    const inHead = head && git(root, ['ls-tree', '--full-tree', '--name-only', '-z', head]).stdout.split('\0').includes(name);
    if (index || inHead) reject(name + ': tracked instruction absent from the working tree; resolve the deliberate deletion manually');
    return { name, file, kind: 'missing', index };
  }
  const flags = git(root, ['ls-files', '-v', '--', name]).stdout.trim();
  if (flags && !/^H /.test(flags)) reject(name + ': assume-unchanged or sparse/index flag; review manually');
  const identity = { dev: stat.dev, ino: stat.ino, mode: stat.mode & 0o777 };
  if (stat.isSymbolicLink()) {
    if (index && !index.startsWith('120000 ')) reject(name + ': tracked regular file and working symlink differ; review manually');
    const target = fs.readlinkSync(file);
    const peer = names.find((other) => other !== name);
    if (target !== peer && target !== './' + peer && !(process.platform === 'win32' && target === '.\\' + peer)) {
      reject(name + ': only a relative link to the other named root entry is supported');
    }
    return { name, file, kind: 'link', peer, target, index, identity };
  }
  if (!stat.isFile() || stat.nlink !== 1) reject(name + ': expected a regular, single-link instruction file');
  const fd = fs.openSync(file, fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW ?? 0));
  let bytes;
  try {
    const current = fs.fstatSync(fd);
    if (!current.isFile() || current.nlink !== 1 || current.dev !== stat.dev || current.ino !== stat.ino) {
      reject(name + ': entry changed while reading; retry audit without concurrent writers');
    }
    bytes = fs.readFileSync(fd);
  } finally { fs.closeSync(fd); }
  let text;
  try { text = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(bytes); } catch {
    reject(name + ': non-UTF-8 instructions, review manually');
  }
  if (text.includes('\0')) reject(name + ': binary instructions, review manually');
  return { name, file, kind: 'file', text, index, identity };
}

function inspectMarkdown(text, name) {
  const active = [];
  const markerContext = [];
  const routed = /^\s*(?:Project-wide agent rules:\s*)?@.*(?:AGENTS|CLAUDE)\.md\s*$/i;
  const owned = /(?:Goals[- ]first development|User goals, value, and business rules|Canonical instructions live in|The full project instructions live in|@(?:[^\s]*\/)?(?:AGENTS|CLAUDE)\.md)/i;
  const dangerous = (s) => s.includes(axiom) || routed.test(s) || owned.test(s);
  let fence = null;
  let comment = false;
  let excluded = false;
  for (const line of text.split(/\r?\n/)) {
    if (fence) {
      const closer = /^ {0,3}([\x60~]+)[\t ]*$/.exec(line);
      if (closer && [...closer[1]].every(c => c === fence[0]) && closer[1].length >= fence.length) fence = null;
      else if (dangerous(line)) excluded = true;
      active.push(''); markerContext.push('');
      continue;
    }
    if (!comment) {
      const opening = /^ {0,3}(\x60{3,}|~{3,})(.*)$/.exec(line);
      if (opening && (opening[1][0] === '~' || !opening[2].includes('\x60'))) {
        fence = opening[1];
        active.push(''); markerContext.push('');
        continue;
      }
    }
    let visible = '', cursor = 0;
    const beganInComment = comment;
    for (;;) {
      if (comment) {
        const end = line.indexOf('-->', cursor);
        if (dangerous(line.slice(cursor, end < 0 ? line.length : end))) excluded = true;
        if (end < 0) break;
        comment = false; cursor = end + 3; visible += ' ';
      } else {
        const start = line.indexOf('<!--', cursor);
        visible += start < 0 ? line.slice(cursor) : line.slice(cursor, start);
        if (start < 0) break;
        comment = true; cursor = start + 4;
      }
    }
    // Only exact own marker lines outside a continued HTML comment are syntax.
    const ownMarker = /^ {0,3}<!--\s*prometheus-(?:base|mini-context):(?:start[^>]*|end\s*)-->\s*$/.test(line);
    markerContext.push(!beganInComment && !comment && ownMarker ? line : visible);
    const contextualExample = /^(?: {0,3}>| {4,}|\t)/.test(visible);
    if (contextualExample && dangerous(visible)) {
      excluded = true;
      active.push('');
    } else active.push(visible);
  }
  if (fence || comment) reject(name + ': unclosed Markdown fence or HTML comment; review manually');
  if (excluded) reject(name + ': policy or instruction reference appears in a non-active Markdown example; review manually');
  return { text: active.join('\n'), markerContext: markerContext.join('\n') };
}

function classify(entry, other) {
  if (entry.kind === 'link') return 'pointer';
  if (entry.kind !== 'file') return 'missing';
  const text = entry.text.replace(/^\uFEFF/, '');
  // With core.symlinks=false Git can materialize a tracked link as a text stub.
  if (entry.index.startsWith('120000 ')) {
    if (text === other.name || text === './' + other.name) return 'pointer';
    reject(entry.name + ': tracked symlink and working entry differ; resolve manually');
  }
  const markdown = inspectMarkdown(text, entry.name);
  const visible = markdown.text;
  const imports = visible.split(/\r?\n/).filter((line) => /^[\t ]*(?:Project-wide agent rules:\s*)?@.*(?:AGENTS|CLAUDE)\.md\s*$/i.test(line));
  const expected = [other.name, './' + other.name];
  const explicitImport = imports.length === 1
    && expected.includes(imports[0].trim().replace(/^(?:Project-wide agent rules:\s*)?@/, ''));
  const routeIntro = /^(?:\*\*)?(?:Canonical instructions live in|The full project instructions live in)\s+/m;
  const documentedRoute = entry.name === 'AGENTS.md' && routeIntro.test(visible)
    && /^(?:\*\*)?(?:Canonical instructions live in|The full project instructions live in)\s+(?:\*\*)?\[[\x60]?CLAUDE\.md[\x60]?\]\((?:\.\/)?CLAUDE\.md\)/m.test(visible);
  if (entry.name === 'AGENTS.md' && routeIntro.test(visible) && !documentedRoute)
    reject(entry.name + ': reported canonical instructions have an unsupported destination');
  const bareLegacyPointer = entry.name === 'AGENTS.md' && visible.trim() === 'CLAUDE.md';
  const opposite = /^(?:\s*[-*]\s+)?(?:\*\*)?(?:Do not|Never) (?:think in terms of|prioriti[sz]e|consider) (?:the )?user goals(?: and value)?\b|^(?:\s*[-*]\s+)?(?:\*\*)?Tests (?:must |shall )?define (?:the )?(?:user value|business requirements|user goals)\b/im;
  if (opposite.test(visible)) reject(entry.name + ': explicit conflicting user-goal instruction; review manually');
  if (imports.length > 1 || (imports.length && !explicitImport)) reject(entry.name + ': unclear instruction import');
  if (explicitImport || bareLegacyPointer || documentedRoute) {
    if (visible.includes(axiom) || /^\s*##\s+Goals.first development\s*$/im.test(visible)) {
      reject(entry.name + ': imported and independent goals policy both present; resolve authority');
    }
    return entry.name === 'AGENTS.md' ? 'documented-route' : 'pointer';
  }
  // Recognize ownership markers rather than splice a generated or broken region.
  const header = visible.split(/\r?\n/).slice(0, 8).join('\n');
  const managed = /\b(?:generated by|auto.generated|agent constitution \(Layer 0\)|Edit `rules\/src\/`, never this file)\b/i.test(header)
    || header.includes('agent constitution (Layer 0)');
  if (managed && !visible.includes(axiom)) reject(entry.name + ': generated instructions must be updated at their owning source');
  for (const marker of ['prometheus-base', 'prometheus-mini-context']) {
    const starts = [...markdown.markerContext.matchAll(new RegExp('<!--\\s*' + marker + ':start[^>]*-->', 'g'))];
    const ends = [...markdown.markerContext.matchAll(new RegExp('<!--\\s*' + marker + ':end\\s*-->', 'g'))];
    if (starts.length !== ends.length || starts.length > 1 || (starts.length && starts[0].index >= ends[0].index)) {
      reject(entry.name + ': malformed managed instructions');
    }
  }
  const axiomCount = visible.split(axiom).length - 1;
  const activeAxiom = visible.split('\n').some(line => (
    line.trim() === axiom || line.startsWith('- **User goals and business rules.** ' + axiom)
  ));
  const ownHeadings = [...visible.matchAll(/^\s*##\s+Goals.first development\s*$/gim)].length;
  const canonicalCoverage = axiomCount === 1 && activeAxiom && /\bVLBV\b/.test(visible)
    && /accepted spec|adopted (?:phase )?contract/i.test(visible)
    && /(?:preserve|existing|established).{0,85}(?:gates|release|verification)/i.test(visible);
  if (axiomCount > 1 || ownHeadings > 1 || (managed && !canonicalCoverage)) reject(entry.name + ': duplicate or generated goals authority');
  if (canonicalCoverage) return 'covered';
  if (axiomCount || ownHeadings || /^\s*#+\s+(?:User goals, value, and business rules|Goals.first)\s*$/im.test(visible)) {
    reject(entry.name + ': partial or conflicting goals section; review manually');
  }
  return 'append';
}

function projectIdentity(root) {
  const top = git(root, ['rev-parse', '--show-toplevel']).stdout.replace(/\r?\n$/, '');
  if (!top || fs.realpathSync(top) !== root) reject('--path must be the exact existing Git top-level');
  const head = git(root, ['rev-parse', '--verify', 'HEAD'], [0, 128]).stdout.trim();
  const ref = git(root, ['symbolic-ref', '--quiet', 'HEAD'], [0, 1]).stdout.trim();
  if (!head && !ref) reject('Git HEAD identity is unavailable');
  const common = fs.realpathSync(git(root, ['rev-parse', '--path-format=absolute', '--git-common-dir']).stdout.replace(/\r?\n$/, ''));
  const { dev, ino } = fs.statSync(root);
  const commonStat = fs.statSync(common);
  return { root, dev, ino, head, ref, common, commonDev: commonStat.dev, commonIno: commonStat.ino };
}

function plan(root) {
  const identity = projectIdentity(root);
  const entries = names.map((name) => readEntry(root, name));
  const modes = entries.map((entry, i) => classify(entry, entries[1 - i]));
  if (modes.every((mode) => mode === 'missing')) reject('both root instruction files are missing; use an explicitly authorized initializer');
  const isPointer = (mode) => mode === 'pointer' || mode === 'documented-route';
  if (modes.some((mode, i) => isPointer(mode) && entries[1 - i].kind !== 'file') || modes.every(isPointer)) {
    reject('broken or circular canonical instruction link');
  }
  const writes = entries.flatMap((entry, index) => {
    if (!['append', 'missing'].includes(modes[index])) return [];
    const existing = entry.text ?? '';
    const eol = existing.includes('\r\n') ? '\r\n' : '\n';
    const prefix = existing ? (existing.endsWith('\n') ? eol : eol + eol) : '';
    return [{ entry, after: existing + prefix + policy.replaceAll('\n', eol) }];
  });
  const digest = sha(JSON.stringify([2, identity, entries, writes]));
  return { identity, entries, modes, writes, digest };
}

function displayDiff({ entry, after }) {
  const split = (text) => text.match(/[^\n]*\n|[^\n]+$/g) ?? [];
  const before = split(entry.text ?? '');
  const next = split(after);
  let same = 0;
  while (same < before.length && before[same] === next[same]) same++;
  const context = Math.max(0, same - 2);
  const line = (prefix, value) => {
    process.stdout.write(prefix + value);
    if (!value.endsWith('\n')) process.stdout.write('\n\\ No newline at end of file\n');
  };
  console.log('--- ' + (entry.kind === 'missing' ? '/dev/null' : 'a/' + entry.name));
  console.log('+++ b/' + entry.name);
  console.log('@@ -' + (before.length ? context + 1 : 0) + ',' + (before.length - context) + ' +' + (context + 1) + ',' + (next.length - context) + ' @@');
  for (const value of before.slice(context, same)) line(' ', value);
  for (const value of before.slice(same)) line('-', value);
  for (const value of next.slice(same)) line('+', value);
}

function apply(root, snapshot) {
  const staged = [];
  let written = 0;
  try {
    for (const { entry, after } of snapshot.writes) {
      const temp = path.join(root, '.' + entry.name + '.prometheus-' + randomBytes(12).toString('hex'));
      const file = fs.openSync(temp, 'wx', entry.identity?.mode ?? 0o644);
      staged.push({ entry, temp });
      try {
        fs.writeFileSync(file, after, 'utf8');
        if (entry.identity && process.platform !== 'win32') fs.fchmodSync(file, entry.identity.mode);
        fs.fsyncSync(file);
      } finally { fs.closeSync(file); }
    }
    if (plan(root).digest !== snapshot.digest) reject('preview became stale before writes; run --preview again');
    const expectedEntries = snapshot.entries.slice();
    for (const { entry, temp } of staged) {
      if (JSON.stringify(projectIdentity(root)) !== JSON.stringify(snapshot.identity)
        || expectedEntries.some((expected) => JSON.stringify(readEntry(root, expected.name)) !== JSON.stringify(expected))) {
        reject('project identity or named instructions changed during apply');
      }
      // Link a completely written file without ever overwriting a concurrently created entry.
      if (entry.kind === 'missing') fs.linkSync(temp, entry.file);
      else fs.renameSync(temp, entry.file);
      written++;
      if (entry.kind === 'missing') fs.unlinkSync(temp);
      const updated = readEntry(root, entry.name);
      const expectedBytes = snapshot.writes.find((write) => write.entry === entry).after;
      if (updated.kind !== 'file' || updated.text !== expectedBytes || updated.index !== entry.index
        || (entry.identity && updated.identity.mode !== entry.identity.mode)) reject(entry.name + ': apply readback disagrees');
      expectedEntries[names.indexOf(entry.name)] = updated;
    }
  } catch (error) {
    if (written) reject('partial apply: ' + written + ' named file(s) changed; audit and reconcile before retrying (' + error.message + ')');
    throw error;
  } finally {
    for (const { temp } of staged) fs.rmSync(temp, { force: true });
  }
  return written;
}

function runExistingProjectMode(args) {
  try {
    const { root, mode, digest } = parse(args);
    const snapshot = plan(root);
    if (mode === 'apply' && digest !== snapshot.digest) reject('preview digest is stale or belongs to another root; run --preview again');
    for (const [i, entry] of snapshot.entries.entries()) {
      const change = snapshot.writes.some((write) => write.entry === entry);
      console.log((change ? 'CHANGE' : 'COVERED') + ' ' + entry.name + ' (' + snapshot.modes[i] + '; ' + (entry.index ? 'tracked' : 'untracked') + ')');
    }
    if (mode === 'preview') {
      for (const write of snapshot.writes) displayDiff(write);
      console.log('PLAN_SHA256=' + snapshot.digest);
    } else if (mode === 'apply') console.log('Applied ' + apply(root, snapshot) + ' named instruction change(s).');
    if (mode !== 'apply') {
      console.log('Review operator prose for semantic conflicts; only recognized patterns are checked.');
      if (snapshot.modes.includes('documented-route')) console.log('Documented AGENTS.md route: follow the existing CLAUDE.md reference explicitly; automatic harness import is not established.');
    }
    return 0;
  } catch (error) {
    console.error((error instanceof Usage ? 'USAGE ' : 'BLOCKED ') + error.message);
    return error instanceof Usage ? 1 : 2;
  }
}

process.exitCode = runExistingProjectMode(process.argv.slice(2));
