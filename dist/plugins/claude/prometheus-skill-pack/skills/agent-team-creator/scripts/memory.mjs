import { createHash } from 'node:crypto';
import { assertNoCredentials, endpoint, object, requestJson, RequestFailure, text } from './models-http.mjs';
function canonical(value) {
    if (Array.isArray(value))
        return `[${value.map(canonical).join(',')}]`;
    if (value && typeof value === 'object')
        return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
    return JSON.stringify(value);
}
const digest = (value) => createHash('sha256').update(canonical(value)).digest('hex');
function reference(state, provenance) {
    if (provenance.kbd === undefined)
        return;
    const kbd = object(provenance.kbd, 'provenance.kbd');
    for (const key of ['projectId', 'runId', 'phaseId', 'changeId', 'taskId'])
        text(kbd[key], `kbd.${key}`);
    const matching = state.tasks.some(task => task.kbd && canonical(task.kbd) === canonical(kbd));
    if (!matching)
        throw new Error('KBD provenance must exactly reference a linked team task; canonical validation is separate');
}
/**
 * Learning envelope (shared/schemas/learning-envelope.schema.json, design
 * docs/design/team-aware-learning-memory.md section 2).
 *
 * Entry scope -> envelope visibility -> surreal-memory agent_id:
 *   role:<r> | agent:<r>          -> visibility `agent`   -> `<team>/<r>`    (role-private; <r> must be a team role)
 *   lead                          -> visibility `lead`    -> `<team>/@lead`
 *   team | team:<anything>        -> visibility `team`    -> `<team>/@team`  (<team> is always this state's team id)
 *   project | project:<anything>  -> visibility `project` -> `@project`
 * Any other scope is rejected at queue time, so nothing unroutable reaches the outbox.
 *
 * Default kind when the caller supplies none: `progress` for `lead` (stage
 * summaries are addressed to the lead), `lesson` for every other scope.
 * user_id is the project id: scopeMapping.userId, else the entry's projectId
 * (queue input projectId or provenance.kbd.projectId). No project id -> throw.
 */
export const ENVELOPE_TRAILER_PREFIX = '<!-- prometheus-envelope ';
const ENVELOPE_TRAILER_SUFFIX = ' -->';
const KINDS = ['lesson', 'gotcha', 'decision', 'progress', 'candidate'];
const AUTHOR_HARNESSES = ['claude-code', 'codex', 'opencode', 'kimi', 'other'];
const PORTABLE_ID = /^[a-z][a-z0-9-]{0,62}$/;
export function routeScope(state, scope) {
    const team = state.team.id;
    const role = /^(?:role|agent):(.+)$/.exec(scope);
    if (role) {
        const roleId = role[1];
        if (!state.team.roles.some(item => item.id === roleId))
            throw new Error(`memory scope names unknown role: ${roleId}`);
        return { visibility: 'agent', roleId, agentId: `${team}/${roleId}` };
    }
    if (scope === 'lead')
        return { visibility: 'lead', agentId: `${team}/@lead` };
    if (scope === 'team' || scope.startsWith('team:'))
        return { visibility: 'team', agentId: `${team}/@team` };
    if (scope === 'project' || scope.startsWith('project:'))
        return { visibility: 'project', agentId: '@project' };
    throw new Error('memory.scope must be role:<role>, agent:<role>, lead, team[:<id>] or project[:<id>]');
}
/** NFC, trimmed, internal whitespace collapsed to one space. contentHash = sha256 hex of this. */
export function normaliseText(value) {
    return value.normalize('NFC').trim().replace(/\s+/g, ' ');
}
export const contentHash = (value) => createHash('sha256').update(normaliseText(value)).digest('hex');
function authorHarness(value) {
    if (value === 'claude')
        return 'claude-code';
    return AUTHOR_HARNESSES.includes(value) ? value : 'other';
}
function suppliedProjectId(input, provenance) {
    if (input.projectId !== undefined)
        return text(input.projectId, 'memory.projectId');
    if (provenance.kbd !== undefined)
        return text(object(provenance.kbd, 'provenance.kbd').projectId, 'kbd.projectId');
    return undefined;
}
function optionalAuthor(value) {
    if (value === undefined)
        return undefined;
    const author = object(value, 'memory.author');
    const out = {};
    for (const key of Object.keys(author))
        if (!['harness', 'agentId', 'agentType', 'sessionId'].includes(key))
            throw new Error(`memory.author.${key} is not an envelope author field`);
    if (author.harness !== undefined)
        out.harness = authorHarness(text(author.harness, 'memory.author.harness'));
    for (const key of ['agentId', 'agentType', 'sessionId'])
        if (author[key] !== undefined)
            out[key] = text(author[key], `memory.author.${key}`);
    return out;
}
/** Builds the envelope for a queued entry; every field is derived from durable state so retries are byte-identical. */
export function buildEnvelope(state, entry, extra = {}) {
    const kbdProject = entry.provenance.kbd && typeof entry.provenance.kbd === 'object' ? entry.provenance.kbd.projectId : undefined;
    const projectId = entry.projectId ?? extra.projectId ?? (typeof kbdProject === 'string' ? kbdProject : undefined);
    if (projectId === undefined || !projectId.trim())
        throw new Error('memory requires a project id (queue projectId, provenance.kbd.projectId or scopeMapping.userId)');
    const route = routeScope(state, entry.scope);
    const roleId = route.roleId ?? entry.roleId;
    const stored = (entry.author ?? {});
    const author = { harness: stored.harness ?? authorHarness(state.team.harness) };
    const agentId = stored.agentId ?? extra.agentId;
    if (agentId !== undefined)
        author.agentId = agentId;
    if (stored.agentType !== undefined)
        author.agentType = stored.agentType;
    const sessionId = stored.sessionId ?? extra.sessionId;
    if (sessionId !== undefined)
        author.sessionId = sessionId;
    const envelope = { schemaVersion: 1, projectId, teamId: state.team.id };
    if (roleId !== undefined)
        envelope.roleId = roleId;
    Object.assign(envelope, {
        visibility: route.visibility,
        kind: entry.kind ?? (route.visibility === 'lead' ? 'progress' : 'lesson'),
        author,
        contentHash: contentHash(entry.content),
        ts: entry.ts ?? new Date(0).toISOString(),
    });
    return { envelope, route: { ...route, roleId } };
}
/** Record content = verbatim lesson text, a blank line, then a one-line envelope trailer. */
export function encodeEnvelopeContent(textValue, envelope) {
    return `${textValue}\n\n${ENVELOPE_TRAILER_PREFIX}${JSON.stringify(envelope)}${ENVELOPE_TRAILER_SUFFIX}`;
}
export function parseEnvelopeContent(content) {
    const at = content.lastIndexOf(`\n\n${ENVELOPE_TRAILER_PREFIX}`);
    if (at < 0 || !content.endsWith(ENVELOPE_TRAILER_SUFFIX))
        return null;
    const json = content.slice(at + 2 + ENVELOPE_TRAILER_PREFIX.length, content.length - ENVELOPE_TRAILER_SUFFIX.length);
    if (json.includes('\n'))
        return null;
    return { text: content.slice(0, at), envelope: JSON.parse(json) };
}
/** Caller commits this mutation atomically before offering the entry for publication. */
export function queueMemory(state, input) {
    assertNoCredentials(input);
    const content = text(input.content, 'memory.content');
    const scope = text(input.scope, 'memory.scope');
    const supplied = input.provenance === undefined ? {} : object(input.provenance, 'memory.provenance');
    reference(state, supplied);
    // Fail closed: every stored record is keyed to a project (user_id). No null user_id.
    const projectId = suppliedProjectId(input, supplied);
    if (projectId === undefined)
        throw new Error('memory.projectId is required (or provenance.kbd.projectId); resolve it with shared/scripts/lib/project-id.sh');
    const route = routeScope(state, scope);
    const kind = input.kind === undefined ? undefined : text(input.kind, 'memory.kind');
    if (kind !== undefined && !KINDS.includes(kind))
        throw new Error(`memory.kind must be one of ${KINDS.join(', ')}`);
    const roleId = input.roleId === undefined ? undefined : text(input.roleId, 'memory.roleId');
    if (roleId !== undefined && (!PORTABLE_ID.test(roleId) || !state.team.roles.some(item => item.id === roleId)))
        throw new Error(`memory.roleId names unknown role: ${roleId}`);
    if (roleId !== undefined && route.roleId !== undefined && roleId !== route.roleId)
        throw new Error('memory.roleId conflicts with the role named by memory.scope');
    const author = optionalAuthor(input.author);
    const provenance = { ...supplied, teamId: state.team.id, source: 'agent-team-runtime', authority: 'local-team-record; KBD references are unverified mirrors' };
    const identity = digest({ content, scope, provenance });
    const id = input.id === undefined ? `memory-${identity.slice(0, 48)}` : text(input.id, 'memory.id');
    if (!/^[a-z][a-z0-9-]{0,62}$/.test(id))
        throw new Error('memory.id must be a portable lowercase identifier');
    const existing = state.outbox.find(entry => entry.id === id);
    if (existing) {
        if (digest({ content: existing.content, scope: existing.scope, provenance: existing.provenance }) !== identity)
            throw new Error('memory id conflicts with different content, scope or provenance');
        if (existing.projectId !== undefined && existing.projectId !== projectId)
            throw new Error('memory id conflicts with a different project id');
        return existing;
    }
    const entry = { id, content, scope, provenance, status: 'queued', projectId, ts: new Date().toISOString() };
    if (kind !== undefined)
        entry.kind = kind;
    if (roleId !== undefined)
        entry.roleId = roleId;
    if (author !== undefined)
        entry.author = author;
    state.outbox.push(entry);
    return entry;
}
function field(value, label) {
    const key = text(value, label);
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(key) || ['__proto__', 'prototype', 'constructor'].includes(key))
        throw new Error('mapping fields must be safe top-level JSON property names');
    return key;
}
function publication(state, entry, input) {
    if (input.provider === 'surreal-memory') {
        const scope = object(input.scopeMapping, 'scopeMapping');
        if (scope.scope !== entry.scope)
            throw new Error('scopeMapping.scope must match the queued scope exactly');
        // scopeMapping.agentId is the writing harness agent (envelope author.agentId);
        // the stored agent_id is the design-table scope key, never caller-chosen.
        const writer = scope.agentId === undefined ? undefined : text(scope.agentId, 'scopeMapping.agentId');
        const sessionId = scope.sessionId === undefined ? undefined : text(scope.sessionId, 'scopeMapping.sessionId');
        const fallbackProject = input.projectId === undefined ? undefined : text(input.projectId, 'publication.projectId');
        const { envelope, route } = buildEnvelope(state, entry, { projectId: fallbackProject, agentId: writer, sessionId });
        const userId = scope.userId === undefined ? envelope.projectId : text(scope.userId, 'scopeMapping.userId');
        const hash = envelope.contentHash;
        const categories = ['env:1', `vis:${route.visibility}`, `kind:${envelope.kind}`, `h:${hash.slice(0, 16)}`];
        if (route.roleId !== undefined)
            categories.push(`author:${state.team.id}/${route.roleId}`);
        // The verified REST request has no metadata or idempotency fields, so the
        // envelope rides inside content as a trailer after the verbatim text.
        const content = encodeEnvelopeContent(entry.content, envelope);
        return {
            body: { content, agent_id: route.agentId, user_id: userId, session_id: sessionId ?? null, categories },
            headers: {}, remoteIdField: 'id', method: 'POST',
            contract: { provider: 'surreal-memory', source: 'https://github.com/Prometheus-AGS/surreal-memory-server/blob/dd7fdcd6d8974af4059d1d51401bd33ae29f65db/src/contracts.rs',
                route: 'POST /api/v1/memory', envelope: 'shared/schemas/learning-envelope.schema.json (content trailer)',
                scopeBinding: 'design-table agent_id and project user_id filters; not an authorization guarantee', remoteIdempotency: 'unsupported-by-verified-contract' },
        };
    }
    if (input.provider !== 'mapped-http')
        throw new Error('memory provider must be surreal-memory or mapped-http');
    const mapping = object(input.mapping, 'mapping');
    const source = text(mapping.source, 'mapping.source');
    const version = text(mapping.version, 'mapping.version');
    const method = mapping.method ?? 'POST';
    if (method !== 'POST' && method !== 'PUT')
        throw new Error('mapping.method must be POST or PUT');
    const body = mapping.constants === undefined ? {} : { ...object(mapping.constants, 'mapping.constants') };
    const fields = [
        [field(mapping.contentField, 'mapping.contentField'), entry.content],
        [field(mapping.scopeField, 'mapping.scopeField'), entry.scope],
        [field(mapping.provenanceField, 'mapping.provenanceField'), entry.provenance],
    ];
    if (mapping.idempotencyField !== undefined)
        fields.push([field(mapping.idempotencyField, 'mapping.idempotencyField'), entry.id]);
    const seen = new Set();
    for (const [key, value] of fields) {
        if (seen.has(key) || Object.hasOwn(body, key))
            throw new Error('memory mapping fields collide');
        seen.add(key);
        body[key] = value;
    }
    const headers = {};
    if (mapping.idempotencyHeader !== undefined) {
        const header = text(mapping.idempotencyHeader, 'mapping.idempotencyHeader');
        if (!/^(?:Idempotency-Key|X-Idempotency-Key)$/i.test(header))
            throw new Error('unsupported idempotency header mapping');
        headers[header] = entry.id;
    }
    return { body, headers, method, remoteIdField: field(mapping.responseIdField, 'mapping.responseIdField'),
        contract: { provider: 'mapped-http', source, version, scopeBinding: 'operator-configured mapping; server authorization unverified',
            remoteIdempotency: mapping.idempotencyHeader || mapping.idempotencyField ? 'operator-mapped; server guarantee unverified' : 'not-configured' } };
}
/** Only an already-queued entry is eligible. Caller persists success AND failure receipts. */
export async function publishMemory(state, input) {
    assertNoCredentials(input);
    const id = text(input.id, 'memory.id');
    const entry = state.outbox.find(item => item.id === id);
    if (!entry)
        throw new Error('queue and persist memory before publication');
    if (entry.status === 'published')
        return { id, status: 'published', receipt: entry.receipt ?? null, repeated: true };
    const previous = entry.receipt && typeof entry.receipt === 'object' && !Array.isArray(entry.receipt) ? entry.receipt : {};
    if (previous.uncertain === true && input.retryUncertain !== true) {
        return { id, status: 'queued', receipt: previous, reason: 'remote outcome uncertain; reconcile before explicitly setting retryUncertain' };
    }
    if (input.url === undefined) {
        entry.receipt = { at: new Date().toISOString(), outcome: 'unavailable', reason: 'no memory endpoint configured', uncertain: false };
        return { id, status: 'queued', receipt: entry.receipt };
    }
    const url = endpoint(input.url);
    // The live server (1.9.0) routes POST /api/v1/memory; the trailing-slash form 404s there.
    if (input.provider === 'surreal-memory' && !/\/api\/v1\/memory\/?$/.test(url.pathname))
        throw new Error('surreal-memory url must name the verified /api/v1/memory route');
    const request = publication(state, entry, input);
    assertNoCredentials(request.body);
    const target = { url: url.href, contract: request.contract };
    const publicationKey = digest({ id, content: entry.content, scope: entry.scope, provenance: entry.provenance, target, body: request.body });
    if (previous.publicationKey !== undefined && previous.publicationKey !== publicationKey)
        throw new Error('retry destination or mapping differs from recorded attempt');
    const receipt = { at: new Date().toISOString(), publicationKey, contentSha256: digest(entry.content), target, localIdempotencyKey: id,
        exactlyOnce: false, uncertaintyNote: 'A crash after remote commit and before local receipt can duplicate a retry; reconcile remotely.' };
    try {
        const response = await requestJson(url, input, request.method, request.body, request.headers);
        const payload = object(response.value, 'memory response');
        const remoteId = payload[request.remoteIdField];
        if (remoteId === undefined || remoteId === null)
            throw new RequestFailure('remote_response_missing_id', true, response.status);
        assertNoCredentials(remoteId);
        entry.receipt = { ...receipt, outcome: 'published', httpStatus: response.status, remoteId, uncertain: false };
        entry.status = 'published';
        return { id, status: 'published', receipt: entry.receipt };
    }
    catch (error) {
        // Invalid configuration fails before I/O; transport and response failures
        // remain durable retryable outbox records without logging remote content.
        if (!(error instanceof RequestFailure)) {
            entry.receipt = { ...receipt, outcome: 'unavailable', reason: 'unsafe_or_unsupported_remote_response', uncertain: true };
        }
        else {
            entry.receipt = { ...receipt, outcome: 'unavailable', reason: error.code, httpStatus: error.httpStatus, uncertain: error.uncertain };
        }
        return { id, status: 'queued', receipt: entry.receipt };
    }
}
