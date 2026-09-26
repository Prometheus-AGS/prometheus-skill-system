// TJ-ARCH-MOB-001 compliant
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../../data/rust_capability_repository.dart';
import '../../domain/capability.dart';
part 'capabilities.g.dart';

@Riverpod(keepAlive: true)
CapabilityRepository capabilityRepository(Ref ref) =>
    RustCapabilityRepository();

@riverpod
Future<List<Capability>> capabilities(Ref ref) =>
    ref.watch(capabilityRepositoryProvider).list();
