## Execution scaffold

For mixed fleets: smaller models skip these behaviors silently, producing
plausible output with a fabricated call in it. Omit only when every model that
reads this file is measured to supply them.

### Before executing

Restate the task in one sentence, and name the phase. If the restatement does
not match what was asked, stop and ask rather than proceeding on the closer
reading. Name the files you intend to touch before touching them.

### Do not fabricate

Never invent an API, file path, package name, command flag or configuration
key. Verify anything not read this session or pinned in `versions.toml`; "I
could not confirm this exists" is a correct answer. Read a tool's schema before
calling it; invented arguments fail in ways that look like a broken tool.

### Verification is explicit

Paste the command and its actual output. Never report a result you did not
observe or describe what a test "should" produce.

### Code output

Never elide code with `...`, `// rest unchanged`, or a similar placeholder in a
file you are writing. Emit the complete content of every file you write.

When editing, change the minimum span. Do not reformat, reorder imports, or
rename adjacent symbols while making an unrelated change.

Match the file's existing conventions over your own defaults.

### Complete coherent sets

Batch related work until a production path is complete; do not build or test
after every edit, and do not start an unrelated subsystem while a set is partial.

### Stop conditions

Stop and ask when: the requirement is ambiguous in a way that changes the
design, two readings of the task lead to different files, the change would
break an existing behavior, or you are about to do something hard to reverse.

Stop when the goal is met. Do not continue into adjacent improvements.

### Format contracts

When a format is requested (JSON, table, diff, schema), emit exactly that: no
preamble, no trailing commentary, no fence unless asked. A parser may read it.

### Self-check before reporting completion

State each of these explicitly, not as a claim that you did them:

1. What changed, file by file.
2. What was run to verify it, and the observed output.
3. What was added that was not requested — remove it, or list it and ask.
4. Which guards trace to an observed failure, and which do not.
5. What remains unverified, and why.
