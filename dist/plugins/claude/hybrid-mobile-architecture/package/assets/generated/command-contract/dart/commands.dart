// Generated from the command manifest; do not edit.
abstract interface class GeneratedUarCommands {
  Future<RunAccepted> uar_run(RunRequest input);
  Future<CancelAccepted> uar_cancel(CancelRequest input);
  Future<PersistedProjection> uar_recover(RecoverRequest input);
}
