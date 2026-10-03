/**
 * Rules about what a generated hook entry is allowed to spawn.
 *
 * WHY A HOOK EXECUTABLE MAY NOT BE A SCRIPT
 *
 * Exec-form dispatch means a `command` plus an argument vector, spawned
 * directly with no shell on any platform. That is what makes a substituted
 * plugin root containing backslashes, `$`, or backticks arrive at the
 * dispatcher unmodified: nothing tokenizes it.
 *
 * The price is that `command` must be something the operating system can
 * actually start. Windows `CreateProcess` cannot launch a `.bat` or `.cmd` --
 * those need `cmd.exe` -- and cannot launch a `.sh`, `.py`, or `.js` at all,
 * because there is no shebang mechanism to supply the interpreter. Node
 * additionally refuses to spawn `.bat`/`.cmd` without `shell: true` at all, as
 * a command-injection fix.
 *
 * So a hook entry naming a script as its executable is not a portable hook
 * entry; it is one that happens to work where a shell is doing hidden work. The
 * check lives in the GENERATOR because a configuration that cannot be
 * dispatched without a shell is a defect in the artifact, and catching it at
 * generation means it can never reach a host.
 */

/** Extensions that only a host shell or an explicit interpreter can start. */
export const SHELL_ONLY_EXECUTABLES = Object.freeze([
  '.sh',
  '.bash',
  '.zsh',
  '.bat',
  '.cmd',
  '.ps1',
  '.py',
  '.js',
  '.mjs',
  '.rb',
  '.pl',
]);

/** The executable a hook entry would spawn, given its `command` string. */
export function hookExecutable(command) {
  return String(command ?? '')
    .trim()
    .split(/\s+/)[0]
    .split(/[\\/]/)
    .pop();
}

/**
 * A message naming why `command` cannot be spawned without a shell, or null.
 *
 * Returning a message rather than throwing lets the generator collect every
 * offending hook in one pass instead of reporting them one run at a time.
 */
export function shellOnlyExecutableError(command) {
  const executable = hookExecutable(command);
  if (!executable) return 'hook entry has no executable';
  const dot = executable.lastIndexOf('.');
  if (dot <= 0) return null;
  const extension = executable.slice(dot).toLowerCase();
  if (!SHELL_ONLY_EXECUTABLES.includes(extension)) return null;
  return (
    `hook executable ${executable} needs a host shell or an explicit interpreter to spawn; ` +
    'exec-form dispatch requires a real executable'
  );
}

/**
 * Harnesses that run only the `command` string and ignore an `args` vector.
 *
 * Verified 2026-10-03 against codex-cli 0.158.0 with a probe plugin: every
 * exec-form entry (`command: "node"` plus `args`) failed -- including one whose
 * script path was absolute -- while single-string commands ran, with
 * `${CLAUDE_PLUGIN_ROOT}` substituted and both `CLAUDE_PLUGIN_ROOT` and
 * `PLUGIN_ROOT` exported. Emitting exec form for Codex therefore shipped hooks
 * that never fired. Codex entries are one string whose tokens contain no
 * whitespace, so it does not matter whether Codex hands it to a shell.
 */
export const COMMAND_STRING_HARNESSES = Object.freeze(['codex']);

/** The argument vector of a hook entry in either exec form or command-string form. */
export function hookArgv(hook) {
  if (Array.isArray(hook?.args)) return hook.args;
  return String(hook?.command ?? '')
    .trim()
    .split(/\s+/)
    .slice(1);
}

/**
 * Smallest hook timeout emitted for Codex, in milliseconds.
 *
 * Codex reads `timeout` as milliseconds; Claude Code reads the same field as
 * seconds. Starting `hook-entry.mjs` costs about one second on its own, so a
 * contract budget of 1000 -- harmless to Claude Code, where it means sixteen
 * minutes -- made Codex kill `sessionstart-kbd-control` and
 * `precompact-kbd-control` every time (observed 2026-10-03, codex-cli 0.158.0:
 * the hook ran in ~1.1s against a 1000ms limit; every hook with a larger budget
 * completed).
 */
export const CODEX_MIN_HOOK_TIMEOUT_MS = 5000;
