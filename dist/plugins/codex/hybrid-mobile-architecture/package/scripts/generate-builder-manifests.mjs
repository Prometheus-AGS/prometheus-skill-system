#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const checkOnly = process.argv.includes("--check");
const manifest = JSON.parse(
  await readFile(join(root, "builder.manifest.json"), "utf8"),
);
const pkg = manifest.package;
const skillsRoot = manifest.distribution.skillSourceRoot;
const packageSkill = manifest.distribution.packageSkill;
const skillDefinitions = await Promise.all(
  manifest.skills.map(async (name) => {
    const markdown = await readFile(
      join(root, skillsRoot, name, "SKILL.md"),
      "utf8",
    );
    const frontmatter = markdown.match(/^---\n([\s\S]*?)\n---\n/)?.[1] ?? "";
    const description =
      frontmatter.match(/^description:\s*(.+)$/m)?.[1]?.trim() ?? "";
    const triggerClause =
      description.match(/Triggers on\s+(.+)$/i)?.[1] ??
      description.match(/Use for\s+(.+)$/i)?.[1] ??
      "";
    const terms = [
      name.replaceAll("-", " "),
      ...triggerClause
        .split(/,\s*|\s+or\s+/)
        .map((term) => term.replace(/[.!]$/, "").trim().toLowerCase())
        .filter((term) => term.length >= 3 && term.length <= 64),
    ];
    return {
      name,
      description,
      terms: [...new Set(terms)].slice(0, 40),
    };
  }),
);
const publicSkills = [packageSkill, ...manifest.skills];
const skillPaths = publicSkills.map((skill) => `./${skillsRoot}/${skill}`);

const commandDescriptions = {
  new: "Create a validated project in an empty destination",
  adopt: "Adopt an evolved application without re-scaffolding it",
  upgrade: "Upgrade Builder-owned files with ownership-aware conflict handling",
  add: "Add a typed feature, auth surface, module, or legacy embed",
  skills: "Install or check pinned Builder skills",
  audit: "Audit a project against its selected Builder profile",
  doctor: "Check Builder, UAR, Prometheus, and harness compatibility",
  manifest: "Generate or verify canonical package manifests",
};

const plugin = {
  name: pkg.id,
  displayName: pkg.displayName,
  version: pkg.version,
  description: pkg.description,
  author: pkg.author,
  organization: pkg.organization,
  license: pkg.license,
  homepage: pkg.homepage,
  repository: { type: "git", url: pkg.repository },
  keywords: [
    "flutter",
    "tauri",
    "rust",
    "react",
    "axum",
    "uar",
    "a2ui",
    "ag-ui",
    "agentic-applications",
  ],
  categories: ["architecture", "scaffolding", "mobile", "desktop", "web", "ai-agent"],
  commands: manifest.commands.map((name) => ({
    name,
    description: commandDescriptions[name],
    executable: "knowme-builder",
  })),
  resources: [
    { name: "Architecture Reference", path: "references/arch-standard.md", type: "reference" },
    { name: "Prometheus Contract", path: manifest.contracts.prometheus, type: "contract" },
    { name: "UAR Contract", path: manifest.contracts.uar, type: "contract" },
  ],
  templates: manifest.templates.map(({ id, path }) => ({ name: id, path })),
  profiles: Object.keys(manifest.profiles),
  harnesses: manifest.supportedHarnesses,
};

const claudePlugin = {
  name: pkg.id,
  version: pkg.version,
  description: pkg.description,
  author: pkg.author,
  homepage: pkg.homepage,
  repository: pkg.repository,
  license: pkg.license,
  keywords: plugin.keywords,
  skills: skillPaths,
  mcpServers: "./.mcp.json",
};

const codexPlugin = {
  ...claudePlugin,
  skills: `./${skillsRoot}/`,
  interface: {
    displayName: pkg.displayName,
    shortDescription: "Build and audit governed hybrid applications.",
    longDescription: pkg.description,
    developerName: pkg.organization,
    category: "Productivity",
    capabilities: ["Skills", "MCP"],
    websiteURL: pkg.homepage,
    defaultPrompt: [
      "Adopt this application with KnowMe Builder.",
      "Audit this hybrid architecture.",
      "Choose a governed application profile.",
    ],
  },
};

const marketplacePlugin = {
  name: pkg.id,
  description: pkg.description,
  source: ".",
  version: pkg.version,
  category: "development",
  tags: ["flutter", "tauri", "react", "axum", "rust", "agentic"],
};

const marketplace = {
  name: "knowme-builder",
  version: pkg.version,
  description: pkg.description,
  owner: {
    name: "KnowMe Tools",
    url: "https://github.com/Know-Me-Tools",
  },
  plugins: [marketplacePlugin],
};

const agentsMarketplace = {
  ...marketplace,
  interface: { displayName: pkg.displayName },
  plugins: [
    {
      ...marketplacePlugin,
      source: { source: "local", path: "./" },
      policy: {
        installation: "INSTALLED_BY_DEFAULT",
        authentication: "ON_INSTALL",
      },
    },
  ],
};

const publicMarketplace = {
  schema_version: "1.0",
  skill: {
    id: pkg.id,
    name: pkg.displayName,
    slug: pkg.id,
    version: pkg.version,
    status: "prerelease",
    visibility: "public",
    tier: "professional",
    author: {
      handle: "travisjames",
      name: pkg.author.name,
      url: pkg.author.url,
      verified: true,
      organization: pkg.organization,
    },
    summary: pkg.description,
    categories: plugin.categories,
    tags: plugin.keywords,
    documentation: {
      readme: "README.md",
      architectural_standard: "references/arch-standard.md",
      changelog: "CHANGELOG.md",
    },
    requirements: {
      platform: ["macos", "linux"],
      tools: ["knowme-builder", "git"],
    },
    profiles: Object.keys(manifest.profiles),
  },
};

const activationManifest = {
  schemaVersion: 1,
  packageVersion: pkg.version,
  generatedFrom: "builder.manifest.json",
  harnesses: manifest.supportedHarnesses,
  skills: skillDefinitions.map(({ name, description, terms }) => ({
    name,
    description,
    terms,
    explicitInvocation: `/${name}`,
    canonicalPath: `${skillsRoot}/${name}`,
  })),
};

const activationCapabilities = {
  schemaVersion: 1,
  lifecycleAuthority: "prometheus",
  mutationAuthority: "prometheus-kbd",
  adapters: {
    "claude-code": {
      discovery: "native-agent-skills",
      activationAid: "user-prompt-submit",
      lifecycleHooks: "prometheus-capability-manifest",
    },
    codex: {
      discovery: "native-agent-skills",
      activationAid: "native-description-routing",
      lifecycleHooks: "prometheus-capability-manifest",
    },
    opencode: {
      discovery: "native-agent-skills",
      activationAid: "plugin-prompt-event",
      lifecycleHooks: "prometheus-capability-manifest",
    },
    "kimi-code": {
      discovery: "native-agent-skills",
      activationAid: "prompt-hook",
      lifecycleHooks: "prometheus-capability-manifest",
    },
  },
};

const targets = new Map([
  ["plugin.json", plugin],
  ["marketplace.json", publicMarketplace],
  [".claude-plugin/plugin.json", claudePlugin],
  [".claude-plugin/marketplace.json", marketplace],
  [".codex-plugin/plugin.json", codexPlugin],
  [".agents/plugins/marketplace.json", agentsMarketplace],
  ["templates/activation-manifest.json", activationManifest],
  ["templates/harness-activation-capabilities.json", activationCapabilities],
]);

const mutationCommands = new Set(["new", "adopt", "upgrade", "add", "skills"]);
for (const [harness, commandRoot] of Object.entries(manifest.generatedCommandRoots)) {
  for (const command of manifest.commands) {
    const fileName =
      harness === "claude-code"
        ? `${command}.md`
        : `knowme-builder-${command}.md`;
    const argumentsToken =
      harness === "claude-code" ? "$ARGUMENTS" : "$ARGUMENTS";
    const checkGuidance = mutationCommands.has(command)
      ? "Preview with the command's `--check` form before applying changes when available."
      : "Do not mutate application files outside the typed CLI contract.";
    targets.set(
      `${commandRoot}/${fileName}`,
      `# KnowMe Builder: ${command}\n\n` +
        `Run \`knowme-builder ${command} ${argumentsToken}\` from the project root. ` +
        `${checkGuidance} Report the typed result, conflicts, and next required gate. ` +
        "Do not edit Builder ownership files or Prometheus compatibility projections directly.\n",
    );
  }
}

let drift = false;
for (const [target, value] of targets) {
  const path = join(root, target);
  const expected =
    typeof value === "string" ? value : `${JSON.stringify(value, null, 2)}\n`;
  let actual = "";
  try {
    actual = await readFile(path, "utf8");
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }

  if (actual === expected) continue;
  drift = true;
  if (checkOnly) {
    process.stderr.write(`generated manifest drift: ${relative(root, path)}\n`);
  } else {
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, expected);
    process.stdout.write(`generated ${relative(root, path)}\n`);
  }
}

if (checkOnly && drift) process.exitCode = 1;
