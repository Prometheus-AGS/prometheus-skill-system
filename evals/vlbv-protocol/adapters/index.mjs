// Harness adapters for the VLBV protocol evals (vlbv-r4-portability T2).
//
// An adapter knows how to (1) say whether its harness can run here, (2) build a
// scratch environment with only the minimum auth material, and (3) invoke one
// headless prompt in the case project. Headless invocation is unverified until
// the boundary run; a harness that cannot authenticate in isolation produces an
// empty or error transcript and its cases are reported BLOCKED, never PASS.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { which } from '../lib/common.mjs';
import { baseEnv, copyAuth, copyFilteredJson, copyFilteredToml } from '../lib/isolation.mjs';

/** Model family from a model string, used to keep the grader cross-family. */
export function familyOf(model) {
  const m = String(model ?? '').toLowerCase();
  if (/claude|anthropic|opus|sonnet|haiku|fable/.test(m)) return 'anthropic';
  if (/minimax/.test(m)) return 'minimax';
  if (/kimi|moonshot|\bk[0-9]/.test(m)) return 'moonshot';
  if (/qwen|dashscope|alibaba/.test(m)) return 'alibaba';
  if (/glm|zhipu/.test(m)) return 'zhipu';
  if (/gpt|openai|\bo[0-9]|codex/.test(m)) return 'openai';
  return 'unknown';
}

function readRealModel(rel, pattern) {
  const file = path.join(os.homedir(), rel);
  if (!fs.existsSync(file)) return null;
  return fs.readFileSync(file, 'utf8').match(pattern)?.[1] ?? null;
}

/** CLAUDE_CODE_OAUTH_TOKEN from the environment or ~/.prometheus/kbd/secrets.env (never logged). */
function claudeToken() {
  if (process.env.CLAUDE_CODE_OAUTH_TOKEN) return process.env.CLAUDE_CODE_OAUTH_TOKEN;
  const file = path.join(os.homedir(), '.prometheus/kbd/secrets.env');
  if (!fs.existsSync(file)) return null;
  const line = fs.readFileSync(file, 'utf8').split('\n').find(l => /^\s*(export\s+)?CLAUDE_CODE_OAUTH_TOKEN\s*=/.test(l));
  if (!line) return null;
  const value = line.replace(/^\s*(export\s+)?CLAUDE_CODE_OAUTH_TOKEN\s*=\s*/, '').trim().replace(/^(['"])(.*)\1$/, '$2');
  return value || null;
}

const installed = (binary, label) => () => (which(binary) ? { ok: true } : { ok: false, reason: `${label} CLI (${binary}) not found on PATH` });

const ADAPTERS = {
  'claude-code': {
    binary: 'claude',
    family: () => 'anthropic',
    model: () => 'harness default',
    // Claude Code's interactive login lives in the macOS keychain under a name
    // tied to the default config dir, so a relocated CLAUDE_CONFIG_DIR is "Not
    // logged in" (B4 attempt 1). Isolated runs authenticate with the long-lived
    // token from `claude setup-token`, kept in ~/.prometheus/kbd/secrets.env as
    // CLAUDE_CODE_OAUTH_TOKEN and passed only through the child environment.
    available() {
      if (!which('claude')) return { ok: false, reason: 'Claude Code CLI (claude) not found on PATH' };
      if (!claudeToken()) return { ok: false, reason: 'CLAUDE_CODE_OAUTH_TOKEN not set (run `claude setup-token` and add it to ~/.prometheus/kbd/secrets.env)' };
      return { ok: true };
    },
    prepare(scratch) {
      return {
        env: { ...baseEnv(scratch), CLAUDE_CONFIG_DIR: path.join(scratch.home, '.claude'), CLAUDE_CODE_OAUTH_TOKEN: claudeToken() },
        copied: [],
      };
    },
    argv: prompt => ['-p', prompt, '--output-format', 'json', '--dangerously-skip-permissions'],
    text(stdout) {
      try { return String(JSON.parse(stdout).result ?? stdout); } catch { return stdout; }
    },
  },
  codex: {
    binary: 'codex',
    family: () => familyOf(readRealModel('.codex/config.toml', /^\s*model\s*=\s*"([^"]+)"/m) ?? 'openai'),
    model: () => readRealModel('.codex/config.toml', /^\s*model\s*=\s*"([^"]+)"/m) ?? 'harness default',
    available: installed('codex', 'Codex'),
    prepare(scratch) {
      const codexHome = path.join(scratch.home, '.codex');
      const copied = [
        copyAuth('.codex/auth.json', scratch.home),
        copyFilteredToml('.codex/config.toml', scratch.home,
          /^\s*(model|model_provider|model_reasoning_effort|preferred_auth_method)\s*=/, ['model_providers', 'profiles']),
      ].filter(Boolean);
      return { env: { ...baseEnv(scratch), CODEX_HOME: codexHome }, copied };
    },
    argv: (prompt, project) => ['exec', '--skip-git-repo-check', '--ephemeral', '--sandbox', 'workspace-write', '-C', project, prompt],
    text: stdout => stdout,
  },
  opencode: {
    binary: 'opencode',
    family: () => familyOf(readRealModel('.config/opencode/opencode.json', /"model"\s*:\s*"([^"]+)"/)),
    model: () => readRealModel('.config/opencode/opencode.json', /"model"\s*:\s*"([^"]+)"/) ?? 'harness default',
    available: installed('opencode', 'OpenCode'),
    prepare(scratch) {
      const copied = [
        copyAuth('.local/share/opencode/auth.json', scratch.home),
        copyFilteredJson('.config/opencode/opencode.json', scratch.home, ['$schema', 'model', 'small_model', 'provider']),
      ].filter(Boolean);
      return { env: baseEnv(scratch), copied };
    },
    argv: prompt => ['run', '--auto', prompt],
    text: stdout => stdout,
  },
  kimi: {
    binary: 'kimi',
    family: () => 'moonshot',
    model: () => readRealModel('.kimi-code/config.toml', /^\s*default_model\s*=\s*"([^"]+)"/m) ?? 'harness default',
    available: installed('kimi', 'Kimi Code'),
    prepare(scratch) {
      const copied = [
        copyAuth('.kimi-code/credentials', scratch.home),
        copyAuth('.kimi-code/oauth', scratch.home),
        copyAuth('.kimi-code/device_id', scratch.home),
        copyAuth('.kimi-code/region', scratch.home),
        copyFilteredToml('.kimi-code/config.toml', scratch.home,
          /^\s*(default_model|default_thinking)\s*=/, ['providers', 'models', 'thinking']),
      ].filter(Boolean);
      return { env: baseEnv(scratch), copied };
    },
    // Kimi refuses --prompt combined with --yolo or --auto (B5 attempt 2b); prompt
    // mode runs with its own non-interactive approval behaviour.
    argv: prompt => ['-p', prompt],
    text: stdout => stdout,
  },
  'qwen-code': {
    binary: 'qwen',
    family: () => 'alibaba',
    model: () => 'harness default',
    available: installed('qwen', 'Qwen Code'),
    prepare(scratch) {
      const copied = [
        copyFilteredJson('.qwen/settings.json', scratch.home, ['security', 'model', 'modelProviders', 'env', 'selectedAuthType']),
        copyAuth('.qwen/oauth_creds.json', scratch.home),
      ].filter(Boolean);
      return { env: baseEnv(scratch), copied };
    },
    argv: prompt => ['-p', prompt, '--approval-mode', 'yolo'],
    text: stdout => stdout,
  },
};

/** A provisioned no-output harness for the EVAL-NOTRUN-1 negative control. */
function stubAdapter(kind) {
  if (kind !== 'exit0') throw new Error(`unknown stub ${kind}`);
  let binary = null;
  return {
    id: 'stub-exit0',
    binary: 'stub-exit0',
    family: () => 'stub',
    model: () => 'stub',
    available: () => ({ ok: true }),
    prepare(scratch) {
      binary = path.join(scratch.tmp, 'stub-exit0');
      fs.writeFileSync(binary, '#!/bin/sh\nexit 0\n', { mode: 0o755 });
      return { env: baseEnv(scratch), copied: [] };
    },
    resolveBinary: () => binary,
    argv: () => [],
    text: stdout => stdout,
  };
}

const NOT_RUN_REASONS = {
  zed: 'Zed has no headless agent CLI (only an editor CLI); evidence/harness-baseline.txt',
  'minimax-code': 'MiniMax Code CLI is not installed on this machine; evidence/harness-baseline.txt',
  'mimo-code': 'MiMo-Code CLI is not installed on this machine; evidence/harness-baseline.txt',
};

/**
 * Resolve an adapter by harness id.
 * @param {string} id @param {{stub?: string}} [options]
 */
export function adapterFor(id, options = {}) {
  if (options.stub) return stubAdapter(options.stub);
  if (NOT_RUN_REASONS[id]) {
    return { id, binary: null, family: () => 'n/a', model: () => 'n/a', available: () => ({ ok: false, reason: NOT_RUN_REASONS[id] }) };
  }
  const adapter = ADAPTERS[id];
  if (!adapter) throw new Error(`unknown harness ${id}`);
  return { id, resolveBinary: () => which(adapter.binary), ...adapter };
}
