# Pre-decode MCP payload bound remediation

The allocation-bound warning was valid. `decode_bounded` previously checked
the decoded byte length only after base64 had allocated the complete result.

The decoder now derives the exact maximum unpadded-base64url character count
from each byte ceiling and rejects a longer string before decoding. The same
derived limits are emitted as `maxLength` for `codeBase64` and every value in
the `inputs` map, so the checked schema and runtime agree.

Local verification:

- `base64_limit_is_checked_before_decoding` passed and proves rejection occurs
  from the encoded-length guard;
- the schema contract test passed;
- warnings-denied clippy passed;
- generated contract readback reports code `maxLength: 11184811` and input
  value `maxLength: 22369622`.

Task 6.3 remains the self-referential closure item described in prior feedback
and is completed only after a zero-finding review is recorded.
