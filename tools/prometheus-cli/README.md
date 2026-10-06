# Prometheus CLI in The Boss

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
