#!/usr/bin/env node
// TJ-ARCH-MOB-001 compliant

import fs from "node:fs";

const [settingsPath] = process.argv.slice(2);

if (!settingsPath) {
  console.error("usage: merge-zed-context-servers.mjs <settings.json>");
  process.exit(2);
}

const entries = [
  {
    name: "dart",
    body: `    "dart": {\n      "command": "dart",\n      "args": ["mcp-server", "--force-roots-fallback"],\n    },\n`,
  },
  {
    name: "shadcn",
    body: `    "shadcn": {\n      "command": "npx",\n      "args": ["shadcn@latest", "mcp"],\n    },\n`,
  },
];

let source = fs.existsSync(settingsPath)
  ? fs.readFileSync(settingsPath, "utf8")
  : "{\n}\n";

const missing = entries.filter(
  ({ name }) => !new RegExp(`^[ \\t]*["']${name}["'][ \\t]*:`, "m").test(source),
);

if (missing.length === 0) {
  console.log("Zed MCP entries already present");
  process.exit(0);
}

const marker = /("context_servers"\s*:\s*\{\s*\n)/m;
const addition = missing.map(({ body }) => body).join("");

if (marker.test(source)) {
  source = source.replace(marker, `$1${addition}`);
} else {
  const opening = /\{\s*\n/m;
  if (!opening.test(source)) {
    throw new Error(`Zed settings file is not a JSON object: ${settingsPath}`);
  }
  source = source.replace(opening, `{\n  "context_servers": {\n${addition}  },\n`);
}

const temporaryPath = `${settingsPath}.knowme-install.tmp`;
fs.writeFileSync(temporaryPath, source, { mode: 0o600 });
fs.renameSync(temporaryPath, settingsPath);
console.log(`Added ${missing.map(({ name }) => name).join(", ")} to Zed MCP settings`);
