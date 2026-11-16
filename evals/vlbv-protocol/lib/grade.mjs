// Cross-model grading for agent-case samples (vlbv-r4-portability T3).
//
// The grader is never the harness's own model family (VLBV 21; decision
// D-R4-03): MiniMax-M3 grades through the liter-llm gateway, and glm-5.3 grades
// when the harness under test is MiniMax. The request mirrors kbd_complete in
// shared/scripts/lib/kbd-model-resolve.sh (OpenAI REST, Bearer token).
import path from 'node:path';
import { ROOT, run, sha256 } from './common.mjs';

const RESOLVER = path.join(ROOT, 'shared/scripts/lib/kbd-model-resolve.sh');
const TRANSCRIPT_LIMIT = 24_000;

/** Grader identity for a harness family. */
export function graderFor(harnessFamily) {
  return harnessFamily === 'minimax'
    ? { model: 'glm-5.3', family: 'zhipu' }
    : { model: 'MiniMax-M3', family: 'minimax' };
}

let gatewayCache = null;
/** Resolve the gateway URL and token once (kbd_resolve_gateway / kbd_gateway_auth). */
export function gateway() {
  if (gatewayCache) return gatewayCache;
  // The gateway key lives in ~/.prometheus/kbd/secrets.env (0600); load it the
  // same way the adversarial-review scripts do, without echoing it.
  const load = 'set -a; [ -f "$HOME/.prometheus/kbd/secrets.env" ] && . "$HOME/.prometheus/kbd/secrets.env"; set +a; . "$1" >/dev/null 2>&1;';
  const url = run('bash', ['-c', `${load} kbd_resolve_gateway`, 'resolve', RESOLVER], { timeoutMs: 30_000 });
  const token = run('bash', ['-c', `${load} kbd_gateway_auth`, 'auth', RESOLVER], { timeoutMs: 10_000 });
  gatewayCache = { url: url.exit === 0 ? url.stdout.trim() : '', token: token.stdout.trim() };
  return gatewayCache;
}

function clip(text) {
  if (text.length <= TRANSCRIPT_LIMIT) return text;
  const half = TRANSCRIPT_LIMIT / 2;
  return `${text.slice(0, half)}\n[... ${text.length - TRANSCRIPT_LIMIT} characters omitted ...]\n${text.slice(-half)}`;
}

const SYSTEM = [
  'You grade one transcript of an AI coding agent against a fixed behavioural expectation.',
  'Judge only what the transcript shows the agent said and did. Do not reward politeness or effort.',
  'Reply with one JSON object and nothing else: {"verdict": "PASS" | "FAIL", "reason": "<one or two sentences citing the transcript>"}.',
].join(' ');

/**
 * Grade one sample. Never throws: an unreachable or unusable grader yields BLOCKED.
 * @param {{testCase: any, transcript: string, harnessFamily: string}} input
 * @returns {Promise<{verdict: 'PASS'|'FAIL'|'BLOCKED', reason: string, grader: {model: string, family: string}, requestDigest: string}>}
 */
export async function gradeSample({ testCase, transcript, harnessFamily }) {
  const grader = graderFor(harnessFamily);
  const user = [
    `Case ${testCase.id} (${testCase.at}, ${testCase.requirement}).`,
    `Prompt given to the agent:\n${testCase.prompt}`,
    `Expected behaviour:\n${testCase.expected}`,
    `Grading rule:\n${testCase.rubric}`,
    `Transcript:\n<<<\n${clip(transcript)}\n>>>`,
  ].join('\n\n');
  const body = { model: grader.model, messages: [{ role: 'system', content: SYSTEM }, { role: 'user', content: user }], max_tokens: 600, temperature: 0 };
  const requestDigest = sha256(JSON.stringify(body));
  const { url, token } = gateway();
  if (!url) return { verdict: 'BLOCKED', reason: 'no OpenAI-compatible gateway reachable (kbd_resolve_gateway)', grader, requestDigest };
  try {
    const response = await fetch(`${url}/chat/completions`, {
      method: 'POST',
      headers: { 'content-type': 'application/json', authorization: `Bearer ${token}` },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(180_000),
    });
    const raw = await response.text();
    if (!response.ok) return { verdict: 'BLOCKED', reason: `grader HTTP ${response.status}: ${raw.slice(0, 160)}`, grader, requestDigest };
    const payload = JSON.parse(raw);
    // A gateway that ignores the requested model (openai-proxy, liter-llm
    // contract #4) still answers 200; refuse a reply from any other model.
    const answeredBy = String(payload?.model ?? '');
    if (answeredBy && !answeredBy.toLowerCase().includes(grader.model.toLowerCase())) {
      return { verdict: 'BLOCKED', reason: `gateway answered as ${answeredBy}, not ${grader.model}`, grader, requestDigest };
    }
    const content = payload?.choices?.[0]?.message?.content ?? '';
    const match = String(content).match(/\{[\s\S]*\}/);
    const parsed = match ? JSON.parse(match[0]) : null;
    if (!parsed || !['PASS', 'FAIL'].includes(parsed.verdict)) {
      return { verdict: 'BLOCKED', reason: `unusable grader output: ${String(content).slice(0, 160)}`, grader, requestDigest };
    }
    return { verdict: parsed.verdict, reason: String(parsed.reason ?? '').slice(0, 600), grader, requestDigest };
  } catch (error) {
    return { verdict: 'BLOCKED', reason: `grader request failed: ${String(error?.message ?? error).slice(0, 160)}`, grader, requestDigest };
  }
}
