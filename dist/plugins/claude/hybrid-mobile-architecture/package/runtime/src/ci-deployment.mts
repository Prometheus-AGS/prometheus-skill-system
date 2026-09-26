// TJ-ARCH-MOB-001 compliant
// Invoked only by explicit CI jobs; authoring this module does not publish.
import { appendFileSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { parse, stringify } from 'yaml';
import { invocation, main, object, run } from './portable/platform.mjs';
const baseline = ['knowme-web', 'knowme-docs', 'flint-forge-gateway', 'flint-realtime-fabric', 'flint-gate', 'prometheus-postgres18'];
const images = [...baseline, 'postgres-extension-flint-llm', 'postgres-extension-flint-vault', 'postgres-extension-flint-meta', 'postgres-extension-flint-auth', 'postgres-extension-flint-hooks', 'postgres-extension-pg-net', 'postgres-extension-pg-cron', 'postgres-extension-pgvector'];
function env(name: string): string { const value = process.env[name]; if (!value || /[\r\n\0]/.test(value)) throw new Error(`Missing or invalid ${name}`); return value; }
function digest(image: string): string {
  const registry = env('REGISTRY'), tag = env('TAG');
  if (!/^[a-z0-9./_-]+$/.test(registry) || !/^[\w][\w.-]*$/.test(tag)) throw new Error('Invalid registry or release tag');
  const value: unknown = JSON.parse(run('docker', ['buildx', 'imagetools', 'inspect', `${registry}/${image}:${tag}`, '--format', '{{json .Manifest.Digest}}'], { capture: true }).stdout);
  if (typeof value !== 'string' || !/^sha256:[a-f0-9]{64}$/.test(value)) throw new Error(`Invalid published digest for ${image}`);
  return value;
}
function promotion(): { name: string; source: string; sha: string } {
  const name = env('IMAGE_NAME'), source = env('SOURCE_REF');
  if (!images.includes(name) || !/^ghcr\.io\/[a-z0-9/_-]+@sha256:[a-f0-9]{64}$/.test(source)) throw new Error('Expected a catalog image and digest-pinned GHCR source');
  return { name, source, sha: source.split('@')[1] };
}
await main(async () => {
  const [operation, ...extra] = process.argv.slice(2); if (extra.length) throw new Error('Unexpected CI arguments');
  switch (operation) {
    case 'record-digests':
      for (const image of baseline) appendFileSync(env('GITHUB_OUTPUT'), `${image.replaceAll('-', '_')}=${digest(image)}\n`);
      break;
    case 'sign':
      for (const image of images) run('cosign', ['sign', '--yes', `${env('REGISTRY')}/${image}@${digest(image)}`]);
      break;
    case 'install-skopeo':
      run('sudo', ['apt-get', 'update']); run('sudo', ['apt-get', 'install', '-y', '--no-install-recommends', 'skopeo']); run('skopeo', ['--version']); break;
    case 'install-kustomize': {
      const version = env('KUSTOMIZE_VERSION'); if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error('Invalid Kustomize version');
      const work = mkdtempSync(join(tmpdir(), 'builder-kustomize-'));
      try {
        const archive = join(work, 'kustomize.tar.gz');
        const response = await fetch(`https://github.com/kubernetes-sigs/kustomize/releases/download/kustomize%2Fv${version}/kustomize_v${version}_linux_amd64.tar.gz`);
        if (!response.ok) throw new Error(`Kustomize download failed: ${response.status}`);
        writeFileSync(archive, new Uint8Array(await response.arrayBuffer()));
        run('tar', ['-xzf', archive, '-C', work, 'kustomize']); run('sudo', ['install', '-m', '0755', join(work, 'kustomize'), '/usr/local/bin/kustomize']); run('kustomize', ['version']);
      } finally { rmSync(work, { recursive: true, force: true }); }
      break;
    }
    case 'authenticate': {
      const gcp = env('GCP_REGISTRY_HOST'), acr = env('ACR_NAME'), ecr = env('ECR_REGISTRY');
      run('gcloud', ['auth', 'configure-docker', gcp, '--quiet']); run('az', ['acr', 'login', '--name', acr]);
      const password = run('aws', ['ecr', 'get-login-password'], { capture: true }).stdout;
      const docker = invocation('docker', ['login', '--username', 'AWS', '--password-stdin', ecr]);
      const result = spawnSync(docker.command, docker.args, { input: password, stdio: ['pipe', 'inherit', 'inherit'], shell: false });
      if (result.error) throw result.error; process.exitCode = result.status ?? 1; break;
    }
    case 'mirror': {
      const { name, source, sha } = promotion();
      const targets = [`${env('ACR_REGISTRY')}/${name}`, `${env('GCP_REGISTRY_HOST')}/${env('GCP_PROJECT')}/${env('GCP_REPOSITORY')}/${name}`, `${env('ECR_REGISTRY')}/${name}`];
      for (const target of targets) {
        const reference = `docker://${target}:${sha.slice(7)}`;
        run('skopeo', ['copy', '--all', `docker://${source}`, reference]);
        if (run('skopeo', ['inspect', reference, '--format', '{{.Digest}}'], { capture: true }).stdout.trim() !== sha) throw new Error(`Mirrored digest differs: ${target}`);
      }
      break;
    }
    case 'promote': {
      const { name, source } = promotion();
      const document = object(parse(readFileSync('images.lock.yaml', 'utf8')));
      if (!document.images || typeof document.images !== 'object' || Array.isArray(document.images)) throw new Error('images.lock.yaml has no image map');
      object(document.images)[name] = source;
      writeFileSync('images.lock.yaml', stringify(document));
      run('git', ['config', 'user.name', 'prometheus-image-promoter']); run('git', ['config', 'user.email', 'image-promoter@users.noreply.github.com']); run('git', ['add', 'images.lock.yaml']);
      const changed = run('git', ['diff', '--cached', '--quiet'], { allowFailure: true }).status;
      if (changed === 1) run('git', ['commit', '-m', `promote ${name}`]); else if (changed !== 0) throw new Error(`git diff failed: ${changed}`);
      run('git', ['push']); break;
    }
    default: throw new Error('Expected record-digests, sign, install-skopeo, install-kustomize, authenticate, mirror or promote');
  }
});
