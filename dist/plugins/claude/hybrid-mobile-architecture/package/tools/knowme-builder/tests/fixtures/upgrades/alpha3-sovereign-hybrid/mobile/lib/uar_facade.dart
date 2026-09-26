abstract interface class UarRuntimeFacade {
  Stream<Map<String, Object?>> run({
    required String runId,
    required String message,
    required String idempotencyId,
  });

  Future<void> cancel(String runId);
}

final class DeterministicUarRuntime implements UarRuntimeFacade {
  @override
  Stream<Map<String, Object?>> run({
    required String runId,
    required String message,
    required String idempotencyId,
  }) async* {
    yield <String, Object?>{'type': 'modelDelta', 'text': message};
    yield <String, Object?>{
      'type': 'a2uiSurface',
      'surfaceId': 'result',
      'props': <String, Object?>{'value': message},
    };
    yield <String, Object?>{'type': 'completed', 'runId': runId};
  }

  @override
  Future<void> cancel(String runId) async {}
}
