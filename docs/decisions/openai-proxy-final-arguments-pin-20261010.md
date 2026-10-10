# Pin the demonstrated final-only tool arguments repair

Status: approved bounded native-stack dependency correction, 2026-10-10.

The full pack's optional `tools/openai-proxy` source pin advances from
`7833663d3b46f7467f2017f2cce392c09ec1b7ac` to
`d269910e45811c8450c7d80004405b6ea3cd9665` in
[GQAdonis/openai-proxy](https://github.com/GQAdonis/openai-proxy/commit/d269910e45811c8450c7d80004405b6ea3cd9665).
This exact source was compiled and operated on macOS before this distribution
change. Its upstream PR #2 has subsequently merged; the pin deliberately remains
the operated commit rather than the newer merge commit.

## Observed problem and repair

An actual subscription response supplied complete function arguments in
`response.function_call_arguments.done` and `response.output_item.done`, without
argument deltas. The proxy forwarded empty arguments and premature per-item
finish markers. This prevented tool-enabled UAR teams from dispatching valid
filesystem calls through the configured external liter-llm gateway.

The pinned repair forwards missing final argument suffixes once and emits the
tool-call finish marker at response completion. It does not substitute API-key
billing, widen timeouts or change model selection.

The operator approved this precise observed-defect source-pin override. The
operator-owned release/version configuration remains unchanged. No other
submodule, skill, Cadence payload or application dependency pin is advanced.

## Actual operation evidence

Safe receipts reside in the coordinated convergence repository at
`librefang/docs/plans/agent-fabric-convergence/receipts/`. Their immutable digests
identify the evidence independently of local filesystem location:

| Receipt | SHA-256 |
| --- | --- |
| `customer-proxy-final-only-tool-arguments-20261010.json` | `ea61447477af02c40621bfc7e6cdcc51c58a7ad203d0bf499167cbf6812715e7` |
| `customer-proxy-runtime-install-20261010.json` | `7da52d63bef545f0f4011ccd2f1820a0de517463535eb25768b51bf53ecf69e4` |
| `customer-proxy-final-arguments-corrected-stream-20261010.json` | `dd67b68279e09dd66215cb0dcfb3fc1eb4c3bcfec44b59f2a07252845ab76bd7` |
| `customer-coding-core-proxy-repaired-2.2.26-20261010.json` | `55d3f0ece161517652a8184769dd2fd6acf1f6b1095fb0f74c1f6bb6e1233357` |

The installed Mac binary is 6,121,008 bytes, SHA-256
`1a7a708312048d4c84166cf257795647d666c613e8d2d9075567f7faa01238f6`.
The corrected real subscription/gateway stream on 2026-10-10
00:37:51.818–00:37:57.432 UTC returned HTTP 200, two complete 42-byte argument
payloads, one terminal `tool_calls` marker and `[DONE]`. That bounded operation
used synthetic tool schemas and executed no tools.

Separately, the packaged 2.2.26 coding operation completed six recorded checks,
including a real bounded repository edit, worker/reviewer model attempts and
artifact handoff. Its overall receipt remains partially passed: the later
`durable-work-reopening` stage failed with
`C14_WORK_WORKSPACE_MENU_UNAVAILABLE`; reopening, cancellation and isolation
qualification remain outstanding. The proxy correction is not a claim of
complete customer-workflow qualification.

## Distribution and remaining limits

The existing optional `scripts/install-binaries.sh` source-build path consumes
this gitlink for native full-pack installations. Pinning source alone neither
installs nor starts a service. No installer, global distribution, new build or
operation was run for this two-file distribution change; the existing operated
binary and stream receipts above are reused with their exact source identity.

The Boss's current frozen payload and mini pack do not bundle this proxy. The
recorded customer operation uses a separately configured external native stack;
it does not demonstrate fresh-install bundled subscription readiness. Native
Windows proxy operation remains pending. Mini's service/dependency contract is
unchanged.

See [the optional vendoring decision](openai-proxy-vendoring.md) for installer
ownership and absence behavior.
