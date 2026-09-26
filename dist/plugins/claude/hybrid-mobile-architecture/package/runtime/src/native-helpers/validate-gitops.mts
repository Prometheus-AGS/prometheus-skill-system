import { existsSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { assert, files, main, repoRoot, run, text } from './common.mjs';
await main(() => {
  const root = repoRoot(), gitops = join(root, 'deploy/gitops');
  for (const file of files(join(root, '.github/workflows'))) assert(!/kubectl apply|helm upgrade|argocd sync/.test(text(file)), `direct deployment command found in CI: ${file}`);
  for (const file of files(gitops).filter(file => file.endsWith('.yaml'))) assert(!/^\s+(data|stringData):/m.test(text(file)), `inline Kubernetes secret data found: ${file}`);
  for (const file of files(gitops).filter(file => basename(file) === 'kustomization.yaml')) run('kustomize', ['build', dirname(file)], { capture: true });
  for (const component of ['nginx-ingress', 'nginx-gateway', 'traefik-ingress', 'traefik-gateway', 'envoy-gateway']) assert(existsSync(join(gitops, 'components/edge', component, 'kustomization.yaml')), `missing edge component: ${component}`);
  const docs = text(join(root, 'docs/deployment/edge-routing-and-tls.md'));
  assert(/[Ww]ildcard.*DNS-01/.test(docs) && docs.includes('Certbot'), 'edge routing documentation requires wildcard DNS-01 and Certbot guidance');
  console.log('GitOps validation passed');
});
