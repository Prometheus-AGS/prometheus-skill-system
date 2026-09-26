// TJ-ARCH-MOB-001 compliant
class Capability {
  const Capability({required this.id, required this.name, required this.kind});
  final String id;
  final String name;
  final String kind;
}

abstract interface class CapabilityRepository {
  Future<List<Capability>> list();
}
