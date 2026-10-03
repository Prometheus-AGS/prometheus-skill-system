# Delivery Cadence storage recovery gate — 2026-10-03

The production storage module was exercised once at the completed change boundary on macOS with Node.js. The disposable gate used `saveEvent` and `loadState` from `skills/process/delivery-cadence/scripts/lib/storage.mjs` and removed its temporary run afterward.

- Nine events each carried a 65 MiB state field. The 512 MiB active-journal boundary archived events 1–8, leaving event 9 active.
- The gzip segment was 531,356 bytes. Its recorded SHA-256 matched the archived bytes, and replay found all eight events in order. Loading the active run returned sequence 9.
- An incomplete append was rejected by a read-only load. Recovery saved the exact incomplete bytes in an `interrupted-tail-*.jsonl` file, restored sequence 9, and a further save/load reached sequence 10.
- The production Cadence CLI `status` command read the existing 5.3 GiB Agent Fabric journal without mutation or Node's former 2 GiB `readFile` error. It completed in 1.15 seconds and reported run `adcf4ebd-9fa7-4880-99ae-7545b6bd6bdf`, sequence 1075, active iteration `checkpoint-failed`, and `publicationDue: true`.
- The changed full-pack `SKILL.md`, `storage.mjs`, and `lifecycle.mjs` matched the copied mini files byte for byte at this gate.

This gate proves local storage rollover, archive replay, interrupted-tail recovery, and read-only loading of the current large journal. It does not claim live-run resume, Windows operation, publication, or product feature acceptance. The live journal and its publication obligation were left unchanged.

After applying the change to the newer schema-v3 delivery pipeline, the same bounded storage scenario passed again: archived range 1–8, eight replayed events, preserved interrupted tail, and resumed sequence 10. The schema-v3 CLI also loaded the live journal read-only in 0.23 seconds and reported the same run, sequence 1075, `checkpoint-failed` state, and publication debt. No live mutation occurred.
