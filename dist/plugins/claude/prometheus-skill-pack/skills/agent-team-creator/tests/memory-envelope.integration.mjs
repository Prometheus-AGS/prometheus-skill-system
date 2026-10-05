// Live integration: the packaged CLI queues and publishes through the real
// surreal-memory REST server, then the stored record is read back over HTTP.
// No fakes. When the server is unset or unreachable the suite exits 2 (BLOCKED), never 0.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createHash, randomUUID } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { fixture, skillRoot, team } from './fixture.mjs';
// No default: the live service must never be the target. Run through tests/run-memory-envelope.sh,
// which starts a scratch surreal-memory-server and exports its URL.
if (!process.env.SURREAL_MEMORY_URL) {
    process.stderr.write('BLOCKED: SURREAL_MEMORY_URL is not set (run tests/run-memory-envelope.sh, which starts a scratch server)\n');
    process.exit(2);
}
const base = process.env.SURREAL_MEMORY_URL.replace(/\/+$/, '');
try {
    const health = await fetch(`${base}/health`, { signal: AbortSignal.timeout(3000) });
    if (!health.ok)
        throw new Error(`HTTP ${health.status}`);
}
catch (error) {
    process.stderr.write(`BLOCKED: surreal-memory not reachable at ${base}: ${error instanceof Error ? error.message : String(error)}\n`);
    process.exit(2);
}
const repoRoot = path.resolve(skillRoot, '..', '..', '..');
const schema = JSON.parse(fs.readFileSync(path.join(repoRoot, 'shared', 'schemas', 'learning-envelope.schema.json'), 'utf8'));
/** Minimal JSON Schema 2020-12 subset validator covering every keyword the envelope schema uses. */
function validate(value, node, at = '$') {
    const errors = [];
    const type = node.type;
    const isObj = value !== null && typeof value === 'object' && !Array.isArray(value);
    if (type === 'object' && !isObj)
        return [`${at}: expected object`];
    if (type === 'array' && !Array.isArray(value))
        return [`${at}: expected array`];
    if (type === 'string' && typeof value !== 'string')
        return [`${at}: expected string`];
    if (type === 'number' && typeof value !== 'number')
        return [`${at}: expected number`];
    if ('const' in node && value !== node.const)
        errors.push(`${at}: expected const ${JSON.stringify(node.const)}`);
    if (Array.isArray(node.enum) && !node.enum.includes(value))
        errors.push(`${at}: not in enum`);
    if (typeof value === 'string') {
        if (typeof node.minLength === 'number' && value.length < node.minLength)
            errors.push(`${at}: shorter than minLength`);
        if (typeof node.pattern === 'string' && !new RegExp(node.pattern, 'u').test(value))
            errors.push(`${at}: does not match ${node.pattern}`);
        if (node.format === 'date-time' && (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$/.test(value) || Number.isNaN(Date.parse(value))))
            errors.push(`${at}: not a date-time`);
    }
    if (typeof value === 'number') {
        if (typeof node.minimum === 'number' && value < node.minimum)
            errors.push(`${at}: below minimum`);
        if (typeof node.maximum === 'number' && value > node.maximum)
            errors.push(`${at}: above maximum`);
    }
    if (Array.isArray(value)) {
        if (node.uniqueItems === true && new Set(value.map(item => JSON.stringify(item))).size !== value.length)
            errors.push(`${at}: items not unique`);
        if (node.items)
            value.forEach((item, index) => errors.push(...validate(item, node.items, `${at}[${index}]`)));
    }
    if (isObj) {
        const record = value;
        const properties = (node.properties ?? {});
        for (const key of (node.required ?? []))
            if (!(key in record))
                errors.push(`${at}: missing ${key}`);
        for (const [key, child] of Object.entries(record)) {
            if (properties[key])
                errors.push(...validate(child, properties[key], `${at}.${key}`));
            else if (node.additionalProperties === false)
                errors.push(`${at}: unexpected property ${key}`);
        }
    }
    for (const clause of (node.allOf ?? [])) {
        if (clause.if && validate(value, clause.if, at).length === 0 && clause.then)
            errors.push(...validate(value, clause.then, at));
    }
    return errors;
}
const TRAILER = '<!-- prometheus-envelope ';
function splitContent(content) {
    const at = content.lastIndexOf(`\n\n${TRAILER}`);
    assert.ok(at > 0 && content.endsWith(' -->'), 'stored content must end with the envelope trailer');
    return { text: content.slice(0, at), envelope: JSON.parse(content.slice(at + 2 + TRAILER.length, -4)) };
}
const normalisedHash = (value) => createHash('sha256').update(value.normalize('NFC').trim().replace(/\s+/g, ' ')).digest('hex');
const remoteKey = (remoteId) => {
    if (typeof remoteId === 'string')
        return remoteId.replace(/^memory:/, '');
    const key = remoteId.key;
    const value = key && (key.String ?? Object.values(key)[0]);
    assert.equal(typeof value, 'string', 'remote id must carry a record key');
    return value;
};
async function readBack(userId, key) {
    const direct = await fetch(`${base}/api/v1/memory/${encodeURIComponent(key)}`, { signal: AbortSignal.timeout(10000) });
    if (direct.ok)
        return await direct.json();
    const listed = await fetch(`${base}/api/v1/memory?user_id=${encodeURIComponent(userId)}`, { signal: AbortSignal.timeout(10000) });
    assert.ok(listed.ok, `list memories: HTTP ${listed.status}`);
    const rows = await listed.json();
    const row = rows.find(item => remoteKey(item.id) === key);
    assert.ok(row, 'published record must be readable from surreal-memory');
    return row;
}
test('memory-publish stores a schema-valid envelope under the design-table agent_id with a non-null project user_id', async (t) => {
    const f = fixture();
    t.after(f.close);
    const projectId = `b3b-it-${randomUUID()}`;
    const env = { ...process.env, PROMETHEUS_PROJECT_ID: projectId };
    delete env.CLAUDE_PLUGIN_ROOT;
    delete env.PLUGIN_ROOT;
    const written = [];
    t.after(async () => {
        for (const key of written)
            await fetch(`${base}/api/v1/memory/${encodeURIComponent(key)}`, { method: 'DELETE', signal: AbortSignal.timeout(10000) }).catch(() => undefined);
    });
    f.call('init', { state: f.state, team: team() }, 0, env);
    const cases = [
        { scope: 'role:implementer', agentId: 'example/implementer', visibility: 'agent', role: 'implementer' },
        { scope: 'team:example', agentId: 'example/@team', visibility: 'team', role: undefined },
    ];
    let revision = 0;
    for (const item of cases) {
        const content = `B3b envelope integration ${randomUUID()}:   keep  review evidence separate.`;
        const queued = f.call('memory-queue', { state: f.state, expectedRevision: revision, entry: { content, scope: item.scope } }, 0, env);
        revision = queued.revision;
        const entry = queued.outbox.find((row) => row.content === content);
        assert.equal(entry.projectId, projectId, 'CLI resolves the project id through project-id.sh');
        const sessionId = `b3b-session-${randomUUID()}`;
        const result = f.call('memory-publish', { state: f.state, expectedRevision: revision, publication: {
                id: entry.id, provider: 'surreal-memory', url: `${base}/api/v1/memory`, timeoutMs: 15000,
                scopeMapping: { scope: item.scope, agentId: 'fixture-subagent', sessionId }
            } }, 0, env);
        revision = result.state.revision;
        assert.equal(result.publication.status, 'published', JSON.stringify(result.publication));
        const key = remoteKey(result.publication.receipt.remoteId);
        written.push(key);
        const record = await readBack(projectId, key);
        assert.equal(record.agent_id, item.agentId);
        assert.notEqual(record.user_id, null);
        assert.equal(record.user_id, projectId);
        assert.equal(record.session_id, sessionId);
        const { text, envelope } = splitContent(String(record.content));
        assert.equal(text, content, 'lesson text is stored verbatim ahead of the trailer');
        assert.deepEqual(validate(envelope, schema), [], 'stored envelope validates against learning-envelope.schema.json');
        assert.equal(envelope.projectId, projectId);
        assert.equal(envelope.teamId, 'example');
        assert.equal(envelope.roleId, item.role);
        assert.equal(envelope.visibility, item.visibility);
        assert.equal(envelope.kind, 'lesson');
        assert.deepEqual(envelope.author, { harness: 'codex', agentId: 'fixture-subagent', sessionId });
        assert.equal(envelope.contentHash, normalisedHash(content));
        const categories = record.categories;
        for (const category of ['env:1', `vis:${item.visibility}`, 'kind:lesson', `h:${normalisedHash(content).slice(0, 16)}`])
            assert.ok(categories.includes(category), category);
        assert.equal(categories.includes('author:example/implementer'), item.role === 'implementer');
        // The validator is not vacuous: an extra field and a missing roleId both fail.
        assert.notDeepEqual(validate({ ...envelope, extra: true }, schema), []);
        if (item.visibility === 'agent') {
            const { roleId: _drop, ...noRole } = envelope;
            assert.notDeepEqual(validate(noRole, schema), []);
        }
    }
});
test('queueMemory without a project id throws and leaves the outbox unchanged', async (t) => {
    const f = fixture();
    t.after(f.close);
    f.call('init', { state: f.state, team: team() });
    const memory = await import(pathToFileURL(path.join(skillRoot, 'scripts', 'memory.mjs')).href);
    const state = f.read();
    assert.throws(() => memory.queueMemory(state, { content: 'No project id given', scope: 'team:example' }), /projectId is required/);
    assert.equal(state.outbox.length, 0);
    const queued = memory.queueMemory(state, { content: 'Project id given', scope: 'team:example', projectId: 'b3b-explicit' });
    assert.equal(queued.projectId, 'b3b-explicit');
});
