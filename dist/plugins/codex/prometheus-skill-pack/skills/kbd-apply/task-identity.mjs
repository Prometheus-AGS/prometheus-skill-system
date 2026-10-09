// Runtime task identity for kbd-apply.
//
// /kbd-plan may register a change's tasks before apply runs, under IDs that differ from the
// backend ordinal (for example "<change>-t1" vs "1"). Registering the backend ID beside them
// creates duplicates: the planned tasks stay pending and the change can never complete. So an
// exact ID wins; with no registered tasks the backend ID is new; otherwise reuse the task with
// the same normalized title, then the unique task with the same sequence. Anything else is
// refused rather than duplicated. Mirrors resolve_runtime_task_id in the full pack's kbd-apply.sh.

const normalizeTitle = (title) =>
  String(title ?? '')
    .toLowerCase()
    .replace(/^\s*\d+(\.\d+)*\s+/, '')
    .replace(/\s+/g, ' ')
    .trim();

/**
 * @param {Record<string, {sequence?: number|string, title?: string}>|undefined} registered
 *   the change's registered runtime tasks, keyed by task ID
 * @returns {{id: string, mapped: boolean}} the runtime task ID to use; `mapped` is true when it
 *   differs from the backend ID
 * @throws {Error} when the change has registered tasks and none matches
 */
export function resolveRuntimeTaskId(registered, backendId, sequence, title) {
  const tasks = registered ?? {};
  if (Object.hasOwn(tasks, backendId)) return { id: backendId, mapped: false };
  const entries = Object.entries(tasks);
  if (entries.length === 0) return { id: backendId, mapped: false };

  const wanted = normalizeTitle(title);
  const byTitle = entries.filter(([, task]) => normalizeTitle(task?.title) === wanted);
  if (byTitle.length === 1) return { id: byTitle[0][0], mapped: true };

  const bySequence = entries.filter(([, task]) => String(task?.sequence) === String(sequence));
  if (bySequence.length === 1) return { id: bySequence[0][0], mapped: true };

  throw new Error(
    `change has registered runtime tasks but none matches backend task ${backendId} ` +
      `(sequence ${sequence}, "${title}"); refusing to register a duplicate. ` +
      'Register it with this ID or reconcile the plan.'
  );
}
