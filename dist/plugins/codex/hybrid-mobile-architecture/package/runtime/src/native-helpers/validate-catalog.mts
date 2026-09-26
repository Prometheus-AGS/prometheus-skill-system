import { existsSync, statSync } from 'node:fs';
import { basename, join } from 'node:path';
import { parse } from 'yaml';
import { assert, files, main, repoRoot, run, text } from './common.mjs';
type Row = Record<string, unknown>;
function object(value: unknown): Row { assert(value && typeof value === 'object' && !Array.isArray(value), 'catalog value must be a mapping'); return value as Row; }
function rows(value: unknown): [string, Row][] { return Object.entries(object(value ?? {})).map(([key, row]) => [key, object(row)]); }
await main(() => {
  const root = repoRoot();
  const load = (name: string): Row => { const lock = object(parse(text(join(root, 'deploy', name)))); assert(lock.schema_version === 1, `invalid catalog schema: ${name}`); return lock; };
  const sources = load('sources.lock.yaml'), thirdParty = load('third-party.lock.yaml'), images = load('images.lock.yaml');
  for (const [name, source] of rows(sources.sources)) {
    for (const key of ['repository', 'commit', 'owner', 'dockerfile', 'context', 'platforms', 'license']) assert(key in source, `${name} is missing source field: ${key}`);
    assert(/^[0-9a-f]{40}$/.test(String(source.commit)), `${name} does not use a full commit SHA`);
    assert(String(source.repository).startsWith('https://'), `${name} source must use HTTPS`);
    const dockerfile = join(root, String(source.dockerfile));
    assert(existsSync(dockerfile) && statSync(dockerfile).isFile(), `${name} Dockerfile does not exist: ${source.dockerfile}`);
  }
  for (const [name, image] of rows(thirdParty.images)) { assert(/@sha256:[0-9a-f]{64}$/.test(String(image.reference)), `${name} third-party image is not digest pinned`); assert(image.license, `${name} third-party image has no license record`); }
  for (const [name, artifact] of rows(thirdParty.artifacts)) assert(/^[0-9a-f]{64}$/.test(String(artifact.sha256)), `${name} artifact has no SHA-256 checksum`);
  if (images.status === 'released') for (const [name, image] of rows(images.images)) assert(/^sha256:[0-9a-f]{64}$/.test(String(image.digest)), `${name} released image has no immutable digest`);
  assert(!/repository:.*#(?:main|master)|commit: (?:main|master|HEAD)$/m.test(text(join(root, 'deploy/sources.lock.yaml'))), 'floating source revision found');
  for (const file of files(join(root, 'deploy'))) {
    if (basename(file) === 'README.md' || /validate-catalog\.(?:sh|mjs|mts)$/.test(file)) continue;
    assert(!/(^|[/:])latest([@:]|$)/m.test(text(file)), `floating latest image found: ${file}`);
  }
  const dockerfiles = files(join(root, 'deploy/docker'));
  for (const file of dockerfiles) assert(!/RUN\s+git clone|git clone --depth/.test(text(file)), `Dockerfiles must consume pinned BuildKit contexts: ${file}`);
  for (const file of [...dockerfiles.filter(file => file.endsWith('.Dockerfile')), join(root, 'site/Dockerfile')]) {
    const stages = new Set<string>();
    for (const line of text(file).split(/\r?\n/)) {
      const from = /^FROM\s+(?:--platform=\S+\s+)?(\S+)(?:\s+AS\s+(\S+))?/i.exec(line);
      if (!from) continue;
      const image = from[1]!;
      assert(image === 'scratch' || stages.has(image.toLowerCase()) || /@sha256:[0-9a-f]{64}$/.test(image), `unpinned Docker base image found: ${file}: ${image}`);
      if (from[2]) stages.add(from[2].toLowerCase());
    }
  }
  run('docker', ['buildx', 'bake', '-f', 'deploy/docker-bake.hcl', '--print'], { cwd: root, capture: true });
  console.log('deployment catalog validation passed');
});
