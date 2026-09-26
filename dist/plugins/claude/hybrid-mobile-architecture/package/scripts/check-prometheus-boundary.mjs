#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const settings = JSON.parse(
  await readFile(join(root, ".claude/settings.json"), "utf8"),
);
const prohibitedLifecycleHooks = [
  "Stop",
  "SubagentStop",
  "PreCompact",
  "SessionStart",
  "Interrupt",
];
const failures = [];
for (const hook of prohibitedLifecycleHooks) {
  if (Object.hasOwn(settings.hooks ?? {}, hook)) {
    failures.push(`Builder must not install the ${hook} lifecycle hook`);
  }
}
const adapters = [
  ".claude/hooks/skill-activation.mjs",
  ".kimi-code/hooks/skill-activation.mjs",
  ".opencode/hooks/skill-activation.mjs",
];
for (const adapter of adapters) {
  const text = await readFile(join(root, adapter), "utf8");
  if (
    /current-waypoint|progress\.json|position\.json|prometheus\s+kbd\s+(pause|resume|claim|handoff)/.test(
      text,
    )
  ) {
    failures.push(`${adapter} attempts to own Prometheus lifecycle state`);
  }
}
for (const failure of failures) process.stderr.write(`error: ${failure}\n`);
if (failures.length > 0) process.exitCode = 1;
else process.stdout.write("Builder-Prometheus authority boundary is clean\n");
