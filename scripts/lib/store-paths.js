/**
 * Path comparison for the generation store.
 *
 * THE VERBATIM PREFIX
 *
 * Windows has two spellings for the same absolute path. `C:\Users\x` is the
 * ordinary one; `\\?\C:\Users\x` is the VERBATIM (or "extended-length") one,
 * which skips the Win32 path parser and lifts the 260-character limit. Several
 * APIs hand back the verbatim form whether or not you asked for it:
 *
 *   * Rust's `std::fs::canonicalize` ALWAYS returns it.
 *   * `std::fs::read_link` returns it for a junction, because a junction's
 *     substitute name is stored as `\??\C:\...`.
 *   * `GetFinalPathNameByHandleW` returns it by default.
 *   * Node's `fs.realpathSync.native` can return it for a long path.
 *
 * The two spellings do not compare equal. A containment check written as a
 * prefix or component comparison sees `\\?\C:\store\generations\<id>` and
 * `C:\store\generations` as unrelated, so the escape guard fires on every valid
 * bundle -- the guard reports a compromise where there is none, which is worse
 * than a guard that never fires, because it is indistinguishable from a real
 * attack in a log.
 *
 * Both sides are therefore normalized to the ordinary spelling before any
 * comparison. Normalization NEVER widens what the guard accepts: it only makes
 * two spellings of one path compare as one path. A candidate genuinely outside
 * the store is outside it in either spelling.
 *
 * The MSYS2 form of the same problem appears in the shell runtime, where a
 * verbatim path surfaces as `//?/c/...`; `hook-runtime-v1.sh` strips it in the
 * same way and for the same reason.
 */

import path from 'node:path';

/** `generations/<sha256>` -- the only shape an activation pointer may hold. */
export const POINTER_PATTERN = /^generations\/[a-f0-9]{64}$/;

/**
 * Reduce a Windows verbatim path to its ordinary spelling.
 *
 * `\\?\C:\x` becomes `C:\x` and `\\?\UNC\server\share` becomes
 * `\\server\share`. Anything else, including every POSIX path, is returned
 * unchanged.
 */
export function stripVerbatimPrefix(value) {
  if (typeof value !== 'string') return value;
  if (value.startsWith('\\\\?\\UNC\\')) return `\\\\${value.slice(8)}`;
  if (value.startsWith('\\\\?\\')) return value.slice(4);
  return value;
}

/**
 * True when `candidate` is `parent` or lies beneath it.
 *
 * Both operands are stripped of a verbatim prefix and resolved before the
 * relative-path test, so the answer does not depend on which API produced
 * either string.
 */
export function isWithin(parent, candidate) {
  const normalizedParent = path.resolve(stripVerbatimPrefix(parent));
  const normalizedCandidate = path.resolve(stripVerbatimPrefix(candidate));
  const relative = path.relative(normalizedParent, normalizedCandidate);
  return (
    relative === '' ||
    (!relative.startsWith(`..${path.sep}`) && relative !== '..' && !path.isAbsolute(relative))
  );
}
