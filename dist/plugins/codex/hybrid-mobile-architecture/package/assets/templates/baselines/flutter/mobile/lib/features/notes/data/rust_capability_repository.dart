// TJ-ARCH-MOB-001 compliant
import 'dart:convert';

import 'package:flutter/services.dart';

import '../../../bridge/generated/api/notes.dart' as rust;
import '../domain/capability.dart';

class RustCapabilityRepository implements CapabilityRepository {
  @override
  Future<List<Capability>> list() async {
    final json =
        jsonDecode(
              await rootBundle.loadString('assets/capabilities/index.json'),
            )
            as Map<String, dynamic>;
    if (json['schemaVersion'] != 1) {
      throw StateError('Unsupported capability registry schema');
    }
    final items = <String, Capability>{};
    for (final item in json['capabilities'] as List<dynamic>) {
      final capability = item as Map<String, dynamic>;
      final value = Capability(
        id: capability['id'] as String,
        name: capability['name'] as String,
        kind: capability['kind'] as String,
      );
      items[value.id] = value;
    }
    for (final item in await rust.listCapabilities()) {
      items[item.id] = Capability(
        id: item.id,
        name: item.name,
        kind: item.kind,
      );
    }
    return items.values.toList();
  }
}
