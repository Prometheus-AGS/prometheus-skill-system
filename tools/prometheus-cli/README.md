# Prometheus CLI

## Full pack and local KBD

Use the pack installer to acquire the CLI for your platform. To inspect a registered project, run `prometheus kbd --path "$PROJECT" status --json`. Two sessions can follow the same project with `prometheus kbd --path "$PROJECT" watch`; the watcher can resume with `--after-event` and a saved event ID. The optional Companion supplies a separate selected authority. The [CLI guide](../../docs/guide/16-cli-and-scripts.md) covers saving a cursor and confirming receiver-side delivery.

## Application-managed installations

The native `prometheus` source declares both source-checkout and application-managed
installation paths. Windows, macOS and Linux packaging and live application
behavior have separate acceptance boundaries.

An application-managed installation supplies `prometheus-host.json` beside the
executable. Its `owner` is
`the-boss` and `configuration` points to the application's private runtime JSON.
Before Tokio starts, the CLI launches its command with only the supported host
environment keys. The managed host boundary excludes provider credentials from its documented
output. The configuration
supplies the mini payload, command and service directories, memory endpoint and
token, and gateway endpoint/key and model names.

With a valid managed manifest, the source routes:

- `prometheus doctor` runs the packaged mini doctor; `--json` emits mini's
  documented JSON-lines contract. `--fix --yes` invokes its `copy-skills` action.
- `prometheus setup --check` reports the installed mini runtime. `setup --full`
  starts only the existing `the-boss-prometheus` Compose project after The Boss
  has initialized its credentials. Application updates own binary replacement;
  `--rebuild` and `doctor --refresh` explain that boundary instead of invoking Cargo.
- `prometheus memory install` uses that same managed Compose project.
- `prometheus install --local` selects the project's agent directory. Windows
  and application-managed installs copy files without elevation or symlinks.
  `--no-symlink` also selects copying on other platforms. Existing directories
  are preserved and installation failures return an unsuccessful exit status.

Other commands keep their existing behavior. Without the host manifest, the CLI
retains its full skill-pack setup and doctor workflows. Complete the production phase before compiler or integration work. Installed
behavior must be accepted through The Boss on the target operating system; a
manifest, source branch or successful build is not that acceptance.

## Local terminal-close consumer

The host source supports one optional Node.js consumer per canonical checkout and KBD project ID. A direct `kbd phase close` dispatches after the runtime records a terminal closure, including a signed operator exit. An identical close replay can dispatch again. A refused, failed or blocked nonterminal attempt, contract binding, successor start or disposition read does not dispatch. The ordinary close command keeps its stdout contract and exit status. Delivery diagnostics go to stderr.

To opt in, the operator places a UTF-8 JSON registration in `<home>/.prometheus/kbd/close-consumers/<scope>.json`. Compute `scope` as lowercase hexadecimal SHA-256 of the UTF-8 canonical absolute project path with no trailing newline. Remove the file to disable delivery. With no registration, the host launches no consumer. Each registration permits one fixed local invocation and contains exactly these fields:

| Field | Value |
| --- | --- |
| `schema` | `kbd.close-consumer/v1` |
| `projectRoot`, `projectId` | Canonical absolute checkout path and its registered KBD identity |
| `nodePath`, `nodeSha256` | Canonical absolute path and lowercase SHA-256 of the reviewed Node executable |
| `packageRoot`, `packageSha256` | Canonical absolute directory and tree digest of the reviewed, immutable installation |
| `entryPath`, `entrySha256` | Canonical absolute path and lowercase SHA-256 of the dedicated `.mjs` entry inside the installation |

The tree digest covers all regular files, including bundled dependencies and hidden files. Sort their UTF-8 relative paths by byte order using `/` as the separator. For each file append its path bytes, one NUL byte, its 64 lowercase hexadecimal SHA-256 characters, and one LF byte. Hash the concatenated bytes with SHA-256 and encode in lowercase hexadecimal. Empty directories contribute no bytes. Links and special files are rejected. Installation or runtime changes require explicit review and new digests.

Use an operator-controlled home and dedicated, non-project-writable runtime and installation outside the checkout. Keep all consumer code and dependencies inside the registered installation or the trusted Node runtime. The operator is trusting executable code running as their account, including its dependency and subprocess behavior. The host does not sandbox that code. On Unix the registration and its ancestor directories inside the operator home must be owned by the home owner and deny group and other write access. Runtime and package paths must also be operator- or system-owned and deny group and other writes. A system-owned sticky scratch directory may contain a private installation directory. Provision equivalent ACLs for an operator-controlled home and installation on Windows. Registration in the checkout, project discovery, shell strings, configurable arguments and automatic installation are unsupported.

The host invokes the pinned executable directly with the following argument vector, using the installed package as the working directory:

```text
<entryPath> --project-root <projectRoot> --phase-id <phaseId> --closure-key <terminalClosureKey>
```

The child gets the operator home and a system utility search path with no inherited project environment, standard input or captured output. It must re-read the canonical disposition for the named project and phase, verify the supplied key and reconcile idempotently. Exit zero acknowledges delivery only. Other exits, configuration errors and the 30-second timeout produce a `kbd.close-delivery/v1` diagnostic with `status: reconcile_required` under `delivery`. The phase stays closed. Use the diagnostic's phase and key to inspect consumer records. Reconcile the original closure with the consumer, or replay the identical close after resolving the cause. A lost acknowledgement can follow a successful write, so consumers must deduplicate against durable records before retrying effects.
