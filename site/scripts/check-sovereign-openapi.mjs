#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const specPath = path.join(siteRoot, 'static/openapi/sovereign-sync-v2.openapi.json');
const spec = JSON.parse(fs.readFileSync(specPath, 'utf8'));
const failures = [];
const requireValue = (condition, message) => {
  if (!condition) failures.push(message);
};

requireValue(spec.openapi === '3.1.0', 'OpenAPI version must be 3.1.0');
requireValue(spec.info?.version === '1.7.0', 'release version must be 1.7.0');
for (const route of [
  '/health',
  '/ready',
  '/api/v2/sync/pushes',
  '/api/v2/sync/pushes/{push_id}',
  '/api/v2/sync/pushes/{push_id}/events',
]) {
  requireValue(spec.paths?.[route], `missing route ${route}`);
}
for (const status of ['200', '201', '400', '401', '403', '404', '409', '422', '503']) {
  requireValue(
    spec.paths?.['/api/v2/sync/pushes']?.post?.responses?.[status],
    `POST /api/v2/sync/pushes must declare ${status}`
  );
}
for (const schema of ['SignedSyncPushRequest', 'PushReceipt', 'PushReceiptEvent']) {
  requireValue(spec.components?.schemas?.[schema], `missing schema ${schema}`);
}

const request = spec.components?.examples?.SignedPushRequest?.value;
const accepted = spec.components?.examples?.AcceptedPushReceipt?.value;
const applied = spec.components?.examples?.AppliedPushReceipt?.value;
requireValue(request?.schemaVersion === '1.7', 'signed request example must use schema 1.7');
requireValue(request?.requestId === accepted?.pushId, 'request and receipt IDs must match');
requireValue(
  accepted?.canonicalPayloadHash === applied?.canonicalPayloadHash,
  'exact replay examples must use one canonical payload hash'
);
requireValue(accepted?.localState === 'accepted', 'accepted receipt state is incorrect');
requireValue(applied?.localState === 'broadcast', 'terminal example state is incorrect');
const sequences = (applied?.events ?? []).map(event => event.sequence);
requireValue(
  sequences.every((sequence, index) => index === 0 || sequence > sequences[index - 1]),
  'receipt example events must be strictly ordered'
);
requireValue(
  spec.paths?.['/api/v2/sync/pushes/{push_id}/events']?.get?.parameters?.some(
    parameter => parameter.name === 'after' && parameter.schema?.minimum === 0
  ),
  'SSE route must declare a non-negative after cursor'
);
for (const scenario of [
  'same-id-same-hash',
  'same-id-different-hash',
  'response-loss',
  'event-resume',
]) {
  requireValue(
    spec['x-correctness-scenarios']?.[scenario],
    `missing correctness scenario ${scenario}`
  );
}

if (failures.length > 0) {
  console.error(failures.map(failure => `Sovereign OpenAPI: ${failure}`).join('\n'));
  process.exit(1);
}
console.log(`Sovereign OpenAPI 3.1 contract valid: ${path.relative(siteRoot, specPath)}`);
