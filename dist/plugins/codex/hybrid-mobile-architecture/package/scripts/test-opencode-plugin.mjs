#!/usr/bin/env node

import { strict as assert } from "node:assert";
import { KnowMeBuilderPlugin } from "../.opencode/plugins/knowme-builder.mjs";

const hooks = await KnowMeBuilderPlugin({ directory: process.cwd() });
await hooks["chat.message"](
  { sessionID: "test-session" },
  { parts: [{ type: "text", text: "Audit the Flutter accessibility experience" }] },
);
const output = { system: [] };
await hooks["experimental.chat.system.transform"](
  { sessionID: "test-session", model: {} },
  output,
);
assert.equal(output.system.length, 1);
assert.match(output.system[0], /a11y-gate/);
assert.match(output.system[0], /Prometheus remains/);

const unrelated = { system: [] };
await hooks["chat.message"](
  { sessionID: "test-session" },
  { parts: [{ type: "text", text: "Correct a spelling mistake" }] },
);
await hooks["experimental.chat.system.transform"](
  { sessionID: "test-session", model: {} },
  unrelated,
);
assert.deepEqual(unrelated.system, []);

process.stdout.write("OpenCode advisory plugin contract passed.\n");
