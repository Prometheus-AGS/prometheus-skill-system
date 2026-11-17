#!/usr/bin/env node
// mcp-initialize.mjs — complete one MCP `initialize` handshake over stdio.
//
//   node mcp-initialize.mjs [--timeout <ms>] -- <command> [args...]
//
// Spawns the command with this process's environment, sends `initialize`, and
// requires the FIRST line the server writes to stdout to be the JSON-RPC result.
// Anything else on stdout first (a banner, a stray echo from a wrapper) fails,
// because stdout is the MCP channel. Prints the server's name and version.
//
// Exit: 0 handshake completed · 1 it did not (reason and the server's stderr on stderr)

import { spawn } from 'node:child_process';

const DEFAULT_TIMEOUT_MS = 20_000;
const argv = process.argv.slice(2);
const separator = argv.indexOf('--');
const options = separator === -1 ? [] : argv.slice(0, separator);
const command = separator === -1 ? argv : argv.slice(separator + 1);
const timeoutIndex = options.indexOf('--timeout');
const timeoutMs = timeoutIndex === -1 ? DEFAULT_TIMEOUT_MS : Number(options[timeoutIndex + 1]);

if (command.length === 0 || !Number.isFinite(timeoutMs)) {
  process.stderr.write('usage: mcp-initialize.mjs [--timeout <ms>] -- <command> [args...]\n');
  process.exit(2);
}

const request = {
  jsonrpc: '2.0',
  id: 1,
  method: 'initialize',
  params: {
    protocolVersion: '2025-03-26',
    capabilities: {},
    clientInfo: { name: 'prometheus-launcher-test', version: '1' },
  },
};

const child = spawn(command[0], command.slice(1), { stdio: ['pipe', 'pipe', 'pipe'] });
let stdout = '';
let stderr = '';
let settled = false;

function finish(code, message) {
  if (settled) return;
  settled = true;
  clearTimeout(timer);
  child.kill('SIGTERM');
  if (code === 0) process.stdout.write(`${message}\n`);
  else process.stderr.write(`${message}\n--- server stderr ---\n${stderr}`);
  process.exitCode = code;
}

const timer = setTimeout(() => finish(1, `no initialize result within ${timeoutMs} ms`), timeoutMs);

child.on('error', error => finish(1, `could not start ${command[0]}: ${error.message}`));
child.on('exit', (code, signal) =>
  finish(1, `server exited before answering (code ${code}, signal ${signal})`)
);
child.stderr.on('data', chunk => {
  stderr += chunk;
});
child.stdin.on('error', () => {});
child.stdout.on('data', chunk => {
  stdout += chunk;
  const newline = stdout.indexOf('\n');
  if (newline === -1) return;
  const firstLine = stdout.slice(0, newline);
  let message;
  try {
    message = JSON.parse(firstLine);
  } catch {
    finish(1, `first stdout line is not JSON-RPC: ${firstLine.slice(0, 200)}`);
    return;
  }
  const info = message?.result?.serverInfo;
  if (message?.id !== 1 || !info) {
    finish(1, `first stdout line is not the initialize result: ${firstLine.slice(0, 200)}`);
    return;
  }
  finish(0, `initialize ok: ${info.name} ${info.version ?? ''}`.trim());
});

child.stdin.write(`${JSON.stringify(request)}\n`);
