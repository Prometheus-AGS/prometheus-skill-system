# Protected BDD tests at certification time

BDD tests remain the independent statement of intended behavior. Agents may
use Bash, Python, Edit, and Write freely while developing, including when an
intentional test change is part of the task. No hook parses or blocks a tool
call.

Final local certification compares two committed Git revisions:

```bash
node scripts/verify-protected-tests.mjs --base <certified-base> --candidate <candidate>
```

The verifier detects content changes, deletion, renames, and mode changes in:

- `tests/steps/*.steps.ts`
- `tests/support/*.ts`
- non-draft `tests/features/*.feature`

New scenarios under `tests/features/drafts/` remain unprotected candidates.
The check is method-independent: the Git result is the same whether Bash,
Python, Edit, Write, an IDE, or a formatter produced it.

## Intentional protected changes

Generate a canonical manifest template:

```bash
node scripts/verify-protected-tests.mjs \
  --base <certified-base> --candidate <candidate> \
  --template --approver <allowed-identity> \
  --reason "Why this test change is intentional" > approval.json
```

Sign the exact bytes under the dedicated namespace:

```bash
ssh-keygen -Y sign -f <private-key> \
  -n prometheus-test-change approval.json
```

Then certify with the checked-in trust policy:

```bash
node scripts/verify-protected-tests.mjs \
  --base <certified-base> --candidate <candidate> \
  --approval approval.json --signature approval.json.sig
```

The manifest binds the base and candidate commits, status, old/new paths,
modes, SHA-256 content hashes, reason, approver, and timestamp. Environment
variables and hosted PR labels cannot bypass the check.
