# Native research client

`deep-research-ui.html` is the browser shell served by the full pack's
`prometheus-research` binary. It uses relative assets and HTTP/SSE routes from
that server. A static HTML preview does not execute a research job.

The full skill's Bash/Python package helpers and the native daemon have separate
entry points. Read [stage contracts](../../skills/research/deep-research/references/stage-contracts.md)
for report, citation and thread-merge requirements. Mini has its own Node research
workflow and does not ship this native daemon.

## Run an installed server

From a local machine with the binary and required research harness available:

```bash
prometheus-research --mode server
```

Open `http://127.0.0.1:7891/`. The selected source, provider and harness determine
whether a job can run; seeing the shell or `/health` is not successful research
acceptance. Inspect job status and the emitted package and citation evidence.

The full binary installer can register a macOS `com.prometheus.research` user
service after building the binary. Installation does not prove that the service
started or that the spawned research harness is available. Read
[service operations](../guide/26-service-operations.md) before changing user services.

## Work on the native source

The crate is `substrate/prometheus-research`. Complete the production phase
before building or running its real local integration gate. Native sandbox and
process behavior have host-specific prerequisites; an installed Rust toolchain
alone does not establish support on every OS.

The local [Dockerfile](Dockerfile) and [Compose recipe](docker-compose.yml) build
that crate and expose loopback port 7891. They are development recipes, not a
published, certified image or a substitute for installing the job's collaborating
harness and credentials. Do not use a hosted runner as product acceptance.

## Client and recovery boundaries

The server's API is defined in `substrate/prometheus-research/src/http_server/`.
The shell starts and cancels jobs, reads checkpoints and subscribes to job events.
Opening the HTML through `file://` is not the supported API origin. Static-site
packaging can show the design, but it cannot supply the local native backend.

If the shell is empty or actions fail, inspect the selected server's endpoint,
job status, harness configuration and logs. Do not stop another process merely
because it owns port 7891. Choose a separate local endpoint or use the owner's
service stop procedure. Preserve checkpoints and failed-job evidence before
retrying; source changes and client rendering do not establish job completion.
