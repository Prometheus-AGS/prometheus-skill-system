# Tasks

- [ ] Add the constraint/trigger so DELETE of a Builtin skill fails at the database
- [ ] Test calls the storage provider DIRECTLY, bypassing SkillService, and is refused
- [ ] Confirm the existing service.rs:374 guard still returns 409 on the normal path
