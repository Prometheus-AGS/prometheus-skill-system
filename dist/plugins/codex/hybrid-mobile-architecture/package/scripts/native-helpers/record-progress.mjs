import { randomUUID } from 'node:crypto';
import { appendFileSync, existsSync, mkdirSync, realpathSync, writeFileSync } from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { basename, isAbsolute, join, relative, resolve } from 'node:path';
import { assert, main, repoRoot } from './common.mjs';
await main(() => {
    const args = process.argv.slice(2), values = { status: 'in-progress' };
    const fields = ['phase', 'title', 'summary', 'evidence', 'next', 'status'];
    for (let i = 0; i < args.length; i += 2) {
        const field = args[i].replace(/^--/, '');
        assert(args[i].startsWith('--') && fields.includes(field) && args[i + 1], 'usage: node record-progress.mjs --phase SLUG --title TEXT --summary TEXT --evidence TEXT --next TEXT [--status STATUS]', 2);
        values[field] = args[i + 1];
    }
    for (const field of fields)
        assert(values[field], `missing --${field}`, 2);
    assert(/^[a-z0-9][a-z0-9-]*$/.test(values.phase), 'phase must be a lowercase slug', 2);
    const combined = [values.title, values.summary, values.evidence, values.next].join(' ');
    assert(!/(api[_ -]?key|token|password|secret|private[_ -]?key)\s*[:=]\s*[^$<{\[]/i.test(combined), 'refusing to record text that appears to contain a secret value', 3);
    const root = repoRoot(), slug = basename(root).replace(/[^a-zA-Z0-9-]+/g, '-').replace(/-$/, '');
    const utc = new Date().toISOString().replace(/\.\d{3}Z$/, 'Z'), stamp = utc.replace(/[-:]/g, '');
    const id = `karpathy-progress-${stamp}-${values.phase}-${randomUUID().slice(0, 8)}`;
    const roots = new Set([root, root.replaceAll('\\', '/')]);
    // git canonicalizes /var -> /private/var on macOS; users commonly paste the
    // public spelling. Preserve redaction across that system alias and shell PWD.
    if (process.platform === 'darwin' && /^\/private\/(?:var|tmp)\//.test(root))
        roots.add(root.slice('/private'.length));
    const pwd = process.env.PWD;
    if (pwd && existsSync(pwd) && realpathSync(pwd) === realpathSync(root))
        roots.add(pwd);
    const cwd = resolve();
    if (existsSync(cwd) && realpathSync(cwd) === realpathSync(root))
        roots.add(cwd);
    const canonicalRoot = realpathSync(root);
    for (const base of [tmpdir(), homedir(), process.env.TEMP, process.env.TMP, process.env.USERPROFILE]) {
        if (!base || !existsSync(base))
            continue;
        const suffix = relative(realpathSync(base), canonicalRoot);
        if (!isAbsolute(suffix) && suffix !== '..' && !suffix.startsWith(`..${process.platform === 'win32' ? '\\' : '/'}`))
            roots.add(join(base, suffix));
    }
    for (const path of [...roots]) {
        roots.add(path.replaceAll('\\', '/'));
        roots.add(path.replaceAll('/', '\\'));
    }
    for (const field of ['title', 'summary', 'evidence', 'next'])
        for (const path of [...roots].sort((a, b) => b.length - a.length))
            values[field] = values[field].replaceAll(path, '$REPO_ROOT');
    const projectWiki = join(root, '.prometheus/knowledge/wiki'), privateProject = join(process.env.PROMETHEUS_PRIVATE_ROOT ?? join(homedir(), '.prometheus'), 'knowledge/private', slug), privateWiki = join(privateProject, 'wiki');
    mkdirSync(projectWiki, { recursive: true });
    mkdirSync(privateWiki, { recursive: true });
    const content = `---\ntype: Reference\nid: ${id}\ntitle: ${JSON.stringify(values.title)}\ntags:\n- karpathy-progress\n- ${values.phase}\n- ${JSON.stringify(values.status)}\nsources:\n- conversation:operator-agent\ntimestamp: ${utc}\ncreated_at: ${utc}\nupdated_at: ${utc}\nrevision: 1\n---\n\n## Intent\n\n${values.summary}\n\n## Observed state and verification\n\n${values.evidence}\n\n## Decision and lesson\n\nStatus: ${values.status}. Preserve evidence, distinguish compile proof from runtime proof, and do not narrow the active goal.\n\n## Next experiment\n\n${values.next}\n`;
    const event = JSON.stringify({ id: randomUUID(), kind: 'compiled', session_id: 'karpathy-progress-memory', project_root: '$REPO_ROOT', scope: 'project', timestamp: utc, payload: { entry_id: id, tags: ['karpathy-progress', values.phase, values.status], title: values.title, ts: utc, type: 'compiled' }, affects: [id] }) + '\n';
    for (const path of [join(projectWiki, `${id}.md`), join(privateWiki, `${id}.md`)]) {
        writeFileSync(path, content, { flag: 'wx' });
        console.log(path);
    }
    appendFileSync(join(root, '.prometheus/events.jsonl'), event);
    appendFileSync(join(privateProject, 'events.jsonl'), event);
});
