/**
 * RFC 8785 (JSON Canonicalization Scheme) serialization.
 *
 * WHY A SECOND CANONICALIZER EXISTS
 *
 * `install-plugin-generation.js` already had a `canonicalJson()` that sorts keys
 * and pretty-prints with two-space indentation. That is deterministic, but it is
 * NOT RFC 8785: JCS forbids insignificant whitespace, so a digest computed by a
 * Rust, Go, or Python peer can never equal one produced by the old function.
 * `prometheus-exec` already signs receipts over JCS, so the generation identity
 * is aligned with it here rather than inventing a third convention.
 *
 * RFC 8785 section 3.2.2 defines serialization as ECMAScript `JSON.stringify`
 * with (a) no insignificant whitespace and (b) object members ordered by the
 * UTF-16 code units of their names. `Array.prototype.sort()` with no comparator
 * is exactly UTF-16 code-unit order, and `JSON.stringify` already emits ES
 * `Number::toString` for numbers and the shortest legal escapes for strings. So
 * the whole scheme is "rebuild with sorted keys, then stringify with no spacer"
 * — provided the input contains nothing `JSON.stringify` would silently drop or
 * coerce, which is what the guards below reject loudly instead.
 *
 * The sort must NOT use `localeCompare`. Several call sites in this repository
 * sort with `localeCompare` for human-facing ordering; that is locale-sensitive
 * and would make a digest depend on the host's ICU data.
 */

function rejectUnserializable(value, pointer) {
  const kind = typeof value;
  if (kind === 'undefined') throw new Error(`JCS: undefined at ${pointer || '<root>'}`);
  if (kind === 'function') throw new Error(`JCS: function at ${pointer || '<root>'}`);
  if (kind === 'symbol') throw new Error(`JCS: symbol at ${pointer || '<root>'}`);
  if (kind === 'bigint') throw new Error(`JCS: bigint at ${pointer || '<root>'}`);
  if (kind === 'number' && !Number.isFinite(value)) {
    throw new Error(`JCS: non-finite number at ${pointer || '<root>'}`);
  }
}

function normalize(value, pointer) {
  rejectUnserializable(value, pointer);
  if (value === null) return null;
  if (Array.isArray(value)) {
    return value.map((item, index) => normalize(item, `${pointer}/${index}`));
  }
  if (typeof value === 'object') {
    if (typeof value.toJSON === 'function') return normalize(value.toJSON(), pointer);
    const result = {};
    for (const key of Object.keys(value).sort()) {
      result[key] = normalize(value[key], `${pointer}/${key}`);
    }
    return result;
  }
  return value;
}

/** Canonical RFC 8785 text for `value`. No trailing newline: JCS has none. */
export function jcs(value) {
  return JSON.stringify(normalize(value, ''));
}
