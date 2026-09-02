/**
 * Read `name` and `description` out of a SKILL.md YAML frontmatter block.
 *
 * WHY THIS IS A SHARED MODULE AND NOT A REGEX
 *
 * Two files parsed this independently and both carried the same defect, so the
 * parser now lives in one place. The defect is worth recording, because it is
 * subtle and it was silent for a long time.
 *
 * Both used a negative lookahead to skip block scalars:
 *
 *     /^description:\s*(?![>|])['"]?([^\n]+?)['"]?\s*$/m
 *
 * `\s*` can match ZERO characters, so the regex engine backtracks and evaluates
 * the lookahead at the position immediately after the colon -- where the next
 * character is a SPACE, not `>`. The lookahead therefore passes, the inline
 * branch captures the literal `" >"`, and the folded branch below it is never
 * reached. 231 of the 232 skills using a folded description were affected.
 *
 * It was not cosmetic. This description feeds `searchText` in the generation's
 * skill index, which is what tools match against to find a skill, so those
 * skills shipped effectively undiscoverable.
 *
 * The fix does not try to write a cleverer lookahead. It reads the remainder of
 * the `description:` line as text and then decides, which has no backtracking
 * behaviour to get wrong.
 */

/** YAML block scalar introducer: `>`, `|`, with optional chomping/indent hints. */
const BLOCK_SCALAR = /^[>|][-+]?\d*$/;

function unquote(value) {
  const trimmed = value.trim();
  const first = trimmed.at(0);
  if ((first === '"' || first === "'") && trimmed.at(-1) === first && trimmed.length >= 2) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

/**
 * Collect the indented body of a block scalar that starts on `lines[start]`.
 *
 * A block scalar's body is every following line indented more than the key.
 * A blank line inside it is a paragraph break, which folds to a space here
 * because the consumers all want one line.
 */
function readBlockScalar(lines, start) {
  const body = [];
  for (let index = start + 1; index < lines.length; index += 1) {
    const line = lines[index];
    if (line.trim() === '') {
      body.push('');
      continue;
    }
    if (!/^[ \t]/.test(line)) break;
    body.push(line.trim());
  }
  return body.filter(Boolean).join(' ');
}

/**
 * Parse a frontmatter block into `{ name, description }`.
 *
 * `description` is always collapsed to a single line, because every consumer --
 * a markdown table, a JSON index, a search string -- wants one.
 */
export function parseFrontmatter(block) {
  const lines = String(block ?? '').split('\n');
  const result = { name: '', description: '' };
  for (let index = 0; index < lines.length; index += 1) {
    const match = lines[index].match(/^(name|description):[ \t]*(.*)$/);
    if (!match) continue;
    const [, key, rest] = match;
    const value = BLOCK_SCALAR.test(rest.trim()) ? readBlockScalar(lines, index) : unquote(rest);
    result[key] = value.replace(/\s+/g, ' ').trim();
  }
  return result;
}

/** Extract the frontmatter block from a SKILL.md, tolerating a BOM and CRLF. */
export function frontmatterBlock(text) {
  const normalized = String(text ?? '')
    .replace(/^﻿/, '')
    .replace(/\r\n/g, '\n');
  return normalized.match(/^---\n([\s\S]*?)\n---/)?.[1] ?? '';
}

/** Convenience: parse a SKILL.md's full text. */
export function parseSkillFrontmatter(text) {
  return parseFrontmatter(frontmatterBlock(text));
}
