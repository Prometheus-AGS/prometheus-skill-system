/**
 * OpenCode tool definition for kbd-close.
 * Shells out to the universal session-close wrapper at ~/.local/bin/kbd-close
 * which compiles the session summary into the wiki via pk + logs to learning-log
 * + proposes skill-update candidates. Always exits 0.
 *
 * Invoke explicitly at the end of every KBD phase / OpenCode session.
 */
import { spawn } from 'child_process';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';
import { homedir } from 'os';

const WRAPPER = join(homedir(), '.local', 'bin', 'kbd-close');

export default {
  name: 'kbd-close',
  description:
    'Close the current session and contribute it to the Prometheus self-learning loop. ' +
    'Reads the session summary (from stdin, a file argument, or the last-session-summary ' +
    'fallback), compiles it into the wiki via pk, appends a learning-log entry, and ' +
    'proposes skill-update candidates. Always exits 0. Works from any AI tool.',
  parameters: {
    type: 'object' as const,
    properties: {
      summary_file: {
        type: 'string',
        description:
          'Path to a markdown / text file containing the session summary. ' +
          'If omitted, reads from stdin; if stdin is empty, falls back to ' +
          '~/.prometheus/last-session-summary.txt.',
      },
      source_tag: {
        type: 'string',
        description:
          'Tag for the wiki entry source (e.g. "opencode-kbd-phase-3"). ' +
          'Defaults to "opencode-kbd-close".',
      },
      summary_text: {
        type: 'string',
        description:
          'Inline summary text. If provided, takes precedence over file/stdin.',
      },
    },
  },
  execute: async (
    args: Record<string, unknown>,
    context: { directory: string }
  ) => {
    const { summary_file, source_tag, summary_text } = args as {
      summary_file?: string;
      source_tag?: string;
      summary_text?: string;
    };

    // Build the stdin payload
    let payload = '';
    if (summary_text) {
      payload = summary_text;
    } else if (summary_file && existsSync(summary_file)) {
      payload = readFileSync(summary_file, 'utf-8');
    } else {
      const fallback = join(homedir(), '.prometheus', 'last-session-summary.txt');
      if (existsSync(fallback)) {
        payload = readFileSync(fallback, 'utf-8');
      }
    }

    // Spawn the wrapper; it always exits 0 and logs to ~/.prometheus/logs/kbd-close.log
    return new Promise((resolve) => {
      const child = spawn(WRAPPER, summary_file ? [summary_file] : [], {
        env: {
          ...process.env,
          KBD_CLOSE_SOURCE: source_tag || 'opencode-kbd-close',
        },
        cwd: context.directory,
      });
      if (payload) {
        child.stdin.write(payload);
        child.stdin.end();
      } else {
        child.stdin.end();
      }
      let stdout = '';
      let stderr = '';
      child.stdout.on('data', (d) => (stdout += d.toString()));
      child.stderr.on('data', (d) => (stderr += d.toString()));
      child.on('close', (code) => {
        resolve({
          action: 'kbd-close',
          wrapper: WRAPPER,
          exit_code: code,
          stdout_tail: stdout.split('\n').slice(-5).join('\n'),
          stderr_tail: stderr.split('\n').slice(-5).join('\n'),
          source_tag: source_tag || 'opencode-kbd-close',
          wiki: '~/.prometheus/knowledge/shared/wiki/',
          learning_log: '~/.prometheus/learning-log/',
          skill_updates: '~/.prometheus/skill-updates/',
          next_step:
            'New wiki entries will be available via `pk focus <topic>` on the next session.',
        });
      });
    });
  },
};