import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const siteRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const spec = JSON.parse(
  fs.readFileSync(path.join(siteRoot, 'static/openapi/surreal-memory-v2.openapi.json'), 'utf8'),
);
const outputPath = path.join(siteRoot, 'static/openapi/examples.generated.json');
const generated = {
  schema_version: 1,
  source: 'surreal-memory-v2.openapi.json',
  add_memory_request: spec.components.examples.AddMemoryRequest.value,
  accepted_receipt: spec.components.examples.AcceptedReceipt.value,
  committed_replay_receipt: spec.components.examples.CommittedReceipt.value,
  correctness_scenarios: spec['x-correctness-scenarios'],
};
const rendered = `${JSON.stringify(generated, null, 2)}\n`;

if (process.argv.includes('--check')) {
  const existing = fs.existsSync(outputPath) ? fs.readFileSync(outputPath, 'utf8') : '';
  if (existing !== rendered) {
    console.error('Generated API examples drifted. Run npm run generate:api-examples.');
    process.exit(1);
  }
  console.log('Generated API examples are deterministic and current.');
} else {
  fs.writeFileSync(outputPath, rendered);
  console.log(`Wrote ${path.relative(siteRoot, outputPath)}`);
}

