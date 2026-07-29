import { tool, type Plugin, type PluginModule, type Hooks } from '@opencode-ai/plugin';
import { execFile } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { promisify } from 'node:util';
import evolveToolDef from './tools/evolve.js';
import gitopsToolDef from './tools/gitops.js';
import kbdToolDef from './tools/kbd.js';
import kbdCloseToolDef from './tools/kbd-close.js';

const execFileAsync = promisify(execFile);
const skillPackRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

// Observational only. The pre-mutation fence was removed because it gated the
// operator's own shell on KBD lifecycle and lease state, which blocks ordinary
// work such as editing a submodule or a project this one depends on. Failures
// here are swallowed: recording position must never deny a tool call.
async function kbdControl(event: string): Promise<void> {
  try {
    await execFileAsync(
      'bash',
      [resolve(skillPackRoot, 'shared/scripts/kbd-harness-adapter.sh'), event, 'opencode'],
      {
        cwd: process.cwd(),
        timeout: 1000,
        env: {
          ...process.env,
          PROMETHEUS_HARNESS: 'opencode',
          PROMETHEUS_SKILL_PACK_ROOT: skillPackRoot,
        },
      }
    );
  } catch {
    // Noncritical: the daemon reconciles from the deferred-hook outbox.
  }
}

const evolveTool = tool({
  description: evolveToolDef.description,
  args: {
    evolution_name: tool.schema
      .string()
      .describe('Human-friendly name for cross-session retrieval'),
    domain: tool.schema
      .enum([
        'software',
        'business',
        'product',
        'research',
        'content',
        'operations',
        'compliance',
        'generic',
      ])
      .optional()
      .describe('Evolution domain'),
    goals: tool.schema
      .array(tool.schema.string())
      .optional()
      .describe('What are we trying to achieve?'),
    phase: tool.schema
      .enum(['assess', 'analyze', 'plan', 'execute', 'reflect', 'full'])
      .optional()
      .describe('Run a specific phase or full cycle'),
  },
  async execute(args, context) {
    const result = await evolveToolDef.execute(args as Record<string, unknown>, context);
    return JSON.stringify(result, null, 2);
  },
});

const gitopsTool = tool({
  description: gitopsToolDef.description,
  args: {
    command: tool.schema
      .enum(['bootstrap', 'transform', 'argocd', 'kustomize'])
      .describe('GitOps sub-command'),
    service: tool.schema
      .string()
      .optional()
      .describe('Service name (kebab-case) for kustomize overlays'),
    clouds: tool.schema
      .array(tool.schema.enum(['gke', 'aks', 'eks']))
      .optional()
      .describe('Target clouds (auto-detected if omitted)'),
  },
  async execute(args, context) {
    const result = await gitopsToolDef.execute(args as Record<string, unknown>, context);
    return JSON.stringify(result, null, 2);
  },
});

const kbdTool = tool({
  description: kbdToolDef.description,
  args: {
    command: tool.schema
      .enum(['init', 'assess', 'plan', 'execute', 'reflect', 'status', 'new-phase'])
      .describe('KBD sub-command to run'),
    phase: tool.schema
      .string()
      .optional()
      .describe('Phase name (defaults to active waypoint phase)'),
    goals: tool.schema
      .array(tool.schema.string())
      .optional()
      .describe('Phase goals (required for new-phase command)'),
    backend: tool.schema
      .enum(['openspec', 'native-tool', 'hybrid', 'manual'])
      .optional()
      .describe('Execution backend (for execute command)'),
  },
  async execute(args, context) {
    const result = await kbdToolDef.execute(args as Record<string, unknown>, context);
    return JSON.stringify(result, null, 2);
  },
});

const kbdCloseTool = tool({
  description: kbdCloseToolDef.description,
  args: {
    summary_file: tool.schema
      .string()
      .optional()
      .describe(
        'Path to a markdown / text file containing the session summary. ' +
          'If omitted, reads from stdin; if stdin is empty, falls back to ' +
          '~/.prometheus/last-session-summary.txt.'
      ),
    source_tag: tool.schema
      .string()
      .optional()
      .describe(
        'Tag for the wiki entry source (e.g. "opencode-kbd-phase-3"). ' +
          'Defaults to "opencode-kbd-close".'
      ),
    summary_text: tool.schema
      .string()
      .optional()
      .describe('Inline summary text. If provided, takes precedence over file/stdin.'),
  },
  async execute(args, context) {
    const result = await kbdCloseToolDef.execute(args as Record<string, unknown>, context);
    return JSON.stringify(result, null, 2);
  },
});

const hooks: Hooks = {
  tool: {
    evolve: evolveTool,
    gitops: gitopsTool,
    kbd: kbdTool,
    'kbd-close': kbdCloseTool,
  },

  'shell.env': async (_input, output) => {
    output.env['PROMETHEUS_SKILL_PACK'] = '1';
  },

  'tool.execute.after': async (_input, _output) => {
    await kbdControl('post_mutation');
  },
};

const plugin: Plugin = async (_input, _options) => hooks;

export default { server: plugin } satisfies PluginModule;
