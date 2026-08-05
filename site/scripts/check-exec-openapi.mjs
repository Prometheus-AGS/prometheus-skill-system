import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(siteRoot, '..');
const spec = JSON.parse(
  fs.readFileSync(path.join(siteRoot, 'static/openapi/prometheus-exec.openapi.json'), 'utf8')
);
const failures = [];
const requireValue = (condition, message) => {
  if (!condition) failures.push(message);
};

requireValue(spec.openapi === '3.1.0', 'Prometheus Exec must publish OpenAPI 3.1');
requireValue(spec.info?.version === '1.7.0', 'Prometheus Exec OpenAPI version must be 1.7.0');
for (const route of [
  '/health',
  '/ready',
  '/api/v2/exec/runs',
  '/api/v2/exec/runs/{run_id}',
  '/api/v2/exec/runs/{run_id}/events',
  '/api/v2/exec/receipts/{run_id}',
  '/api/v2/exec/artifacts/{digest}',
]) {
  requireValue(spec.paths?.[route], `Prometheus Exec OpenAPI misses ${route}`);
}
const routeSource = fs.readFileSync(
  path.join(repoRoot, 'substrate/exec-service/src/http.rs'),
  'utf8'
);
const codeRoutes = [
  ...routeSource.matchAll(/\.route\(\s*"([^"]+)"\s*,\s*(get|post|put|patch|delete)\(/gs),
]
  .map(match => `${match[2].toUpperCase()} ${match[1]}`)
  .sort();
const specRoutes = Object.entries(spec.paths ?? {})
  .flatMap(([route, operations]) =>
    Object.keys(operations)
      .filter(method => ['get', 'post', 'put', 'patch', 'delete'].includes(method))
      .map(method => `${method.toUpperCase()} ${route}`)
  )
  .sort();
requireValue(
  JSON.stringify(codeRoutes) === JSON.stringify(specRoutes),
  `OpenAPI/code route drift: code=${codeRoutes.join(', ')} spec=${specRoutes.join(', ')}`
);
for (const schema of [
  'SignedExecRequest',
  'ExecutionRunStatus',
  'ExecutionReceipt',
  'ExecutionApiErrorEnvelope',
  'ExecutionEvidenceIndex',
  'ExecutionCertificationReport',
]) {
  requireValue(spec.components?.schemas?.[schema], `Prometheus Exec OpenAPI misses ${schema}`);
}
requireValue(
  spec.paths?.['/api/v2/exec/runs']?.post?.responses?.['409'],
  'run submission must document canonical request-hash conflict'
);
requireValue(
  spec.paths?.['/api/v2/exec/runs/{run_id}/events']?.get?.responses?.['200']?.content?.[
    'text/event-stream'
  ],
  'run events must document SSE resume semantics'
);

if (failures.length) {
  console.error(failures.map(failure => `Exec OpenAPI: ${failure}`).join('\n'));
  process.exit(1);
}
console.log('Prometheus Exec OpenAPI routes and schemas are valid.');
