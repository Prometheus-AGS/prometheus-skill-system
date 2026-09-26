// TJ-ARCH-MOB-001 compliant
import { createRequire as __createRequire } from 'node:module'; const require = __createRequire(import.meta.url);

// src/native-helpers/validate-gitops.mts
import { existsSync as existsSync2 } from "node:fs";
import { basename, dirname as dirname2, join as join2 } from "node:path";

// src/native-helpers/common.mts
import { spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, realpathSync, statSync } from "node:fs";
import { delimiter, dirname, join, resolve } from "node:path";
var ToolError = class extends Error {
  constructor(message, code = 1) {
    super(message);
    this.code = code;
  }
  code;
};
function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: options.cwd, env: options.env ?? process.env, encoding: "utf8", stdio: options.capture ? ["ignore", "pipe", "pipe"] : "inherit", shell: false, maxBuffer: 64 * 1024 * 1024 });
  if (result.error) throw new ToolError(`required tool failed: ${command}: ${result.error.message}`, 127);
  if (result.status !== 0 && !options.allowFailure) throw new ToolError(`${command} failed (${result.status ?? result.signal})${result.stderr ? `: ${result.stderr.trim()}` : ""}`, result.status ?? 1);
  return result.status === 0 ? result.stdout ?? "" : "";
}
function files(root) {
  if (!existsSync(root)) throw new ToolError(`required directory not found: ${root}`);
  return readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name)).flatMap((entry) => entry.isDirectory() ? files(join(root, entry.name)) : entry.isFile() ? [join(root, entry.name)] : []);
}
function repoRoot() {
  return resolve(run("git", ["rev-parse", "--show-toplevel"], { capture: true }).trim());
}
function text(path) {
  return readFileSync(path, "utf8");
}
function assert(condition, message, code = 1) {
  if (!condition) throw new ToolError(message, code);
}
async function main(fn) {
  try {
    await fn();
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = error instanceof ToolError ? error.code : 1;
  }
}

// src/native-helpers/validate-gitops.mts
await main(() => {
  const root = repoRoot(), gitops = join2(root, "deploy/gitops");
  for (const file of files(join2(root, ".github/workflows"))) assert(!/kubectl apply|helm upgrade|argocd sync/.test(text(file)), `direct deployment command found in CI: ${file}`);
  for (const file of files(gitops).filter((file2) => file2.endsWith(".yaml"))) assert(!/^\s+(data|stringData):/m.test(text(file)), `inline Kubernetes secret data found: ${file}`);
  for (const file of files(gitops).filter((file2) => basename(file2) === "kustomization.yaml")) run("kustomize", ["build", dirname2(file)], { capture: true });
  for (const component of ["nginx-ingress", "nginx-gateway", "traefik-ingress", "traefik-gateway", "envoy-gateway"]) assert(existsSync2(join2(gitops, "components/edge", component, "kustomization.yaml")), `missing edge component: ${component}`);
  const docs = text(join2(root, "docs/deployment/edge-routing-and-tls.md"));
  assert(/[Ww]ildcard.*DNS-01/.test(docs) && docs.includes("Certbot"), "edge routing documentation requires wildcard DNS-01 and Certbot guidance");
  console.log("GitOps validation passed");
});
