import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const specPath = path.join(siteRoot, 'static/openapi/surreal-memory-v2.openapi.json');
const spec = JSON.parse(fs.readFileSync(specPath, 'utf8'));
const failures = [];

const requireValue = (condition, message) => {
  if (!condition) failures.push(message);
};
const canonicalize = value => {
  if (Array.isArray(value)) return value.map(canonicalize);
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value).sort().map(key => [key, canonicalize(value[key])]),
    );
  }
  return value;
};
const payloadHash = payload =>
  crypto.createHash('sha256').update(JSON.stringify(canonicalize(payload))).digest('hex');

requireValue(spec.openapi === '3.1.0', 'OpenAPI version must be 3.1.0');
requireValue(spec.info?.version === '1.6.1', 'OpenAPI release version must be 1.6.1');
for (const route of [
  '/health',
  '/ready',
  '/api/v2/operations',
  '/api/v2/operations/{operation_id}',
  '/api/v2/operations/{operation_id}/events',
]) {
  requireValue(spec.paths?.[route], `missing OpenAPI route ${route}`);
}
for (const status of ['200', '202', '400', '409', '503']) {
  requireValue(
    spec.paths?.['/api/v2/operations']?.post?.responses?.[status],
    `POST /api/v2/operations must declare ${status}`,
  );
}
for (const status of ['200', '404', '503']) {
  requireValue(
    spec.paths?.['/api/v2/operations/{operation_id}']?.get?.responses?.[status],
    `GET operation must declare ${status}`,
  );
  requireValue(
    spec.paths?.['/api/v2/operations/{operation_id}/events']?.get?.responses?.[status],
    `GET operation events must declare ${status}`,
  );
}

const requestExamples = [
  spec.components?.examples?.AddMemoryRequest?.value,
  spec.paths?.['/api/v2/operations']?.post?.requestBody?.content?.['application/json']
    ?.examples?.longLogicalMemory?.value,
];
for (const [index, request] of requestExamples.entries()) {
  requireValue(request?.schema_version === 2, `request example ${index} must use schema 2`);
  requireValue(request?.operation_id?.length > 0, `request example ${index} needs operation_id`);
  requireValue(
    request?.payload_hash === payloadHash(request?.payload),
    `request example ${index} payload_hash does not match canonical payload bytes`,
  );
}

const receipt = spec.components?.examples?.CommittedReceipt?.value;
for (const field of spec.components?.schemas?.OperationReceipt?.required ?? []) {
  requireValue(Object.hasOwn(receipt ?? {}, field), `committed receipt example misses ${field}`);
}
requireValue(receipt?.state === 'committed', 'terminal replay example must be committed');
requireValue(
  spec.paths?.['/api/v2/operations/{operation_id}/events']?.get?.parameters?.some(
    parameter => parameter.name === 'after' && parameter.schema?.minimum === 0,
  ),
  'SSE operation must declare non-negative after cursor',
);

if (failures.length) {
  console.error(failures.map(failure => `OpenAPI: ${failure}`).join('\n'));
  process.exit(1);
}
console.log(`OpenAPI 3.1 contract valid: ${path.relative(siteRoot, specPath)}`);

