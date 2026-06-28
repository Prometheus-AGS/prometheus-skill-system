# Tasks — change-learn-004b

- [ ] Define `StorageProvider` trait with `read`, `write`, `merge`, `list`, and `watch` methods
- [ ] Define `CrdtEngine` trait with `apply`, `merge_docs`, and `export_state` methods
- [ ] Implement `LocalDirAdapter` (read/write JSON files from a configurable base path)
- [ ] Implement `automerge-rs` `CrdtEngine` backed by the schema from change-learn-001
- [ ] Add `IrohDocsAdapter` stub with `unimplemented!()` bodies and a TODO comment linking the Iroh Docs RFC
