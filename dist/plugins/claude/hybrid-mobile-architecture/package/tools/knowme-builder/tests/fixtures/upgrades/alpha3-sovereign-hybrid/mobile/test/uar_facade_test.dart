import 'package:output_mobile/uar_facade.dart';
import 'package:test/test.dart';

void main() {
  test('emits an A2UI surface and completion', () async {
    final events = await DeterministicUarRuntime()
        .run(runId: 'run-1', message: 'hello', idempotencyId: 'command-1')
        .toList();
    expect(events.map((event) => event['type']), contains('a2uiSurface'));
    expect(events.last['type'], 'completed');
  });
}
