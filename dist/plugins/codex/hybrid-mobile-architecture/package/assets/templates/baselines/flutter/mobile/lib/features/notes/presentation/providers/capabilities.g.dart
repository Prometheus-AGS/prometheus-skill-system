// TJ-ARCH-MOB-001 compliant
// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'capabilities.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(capabilityRepository)
final capabilityRepositoryProvider = CapabilityRepositoryProvider._();

final class CapabilityRepositoryProvider
    extends
        $FunctionalProvider<
          CapabilityRepository,
          CapabilityRepository,
          CapabilityRepository
        >
    with $Provider<CapabilityRepository> {
  CapabilityRepositoryProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'capabilityRepositoryProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$capabilityRepositoryHash();

  @$internal
  @override
  $ProviderElement<CapabilityRepository> $createElement(
    $ProviderPointer pointer,
  ) => $ProviderElement(pointer);

  @override
  CapabilityRepository create(Ref ref) {
    return capabilityRepository(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(CapabilityRepository value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<CapabilityRepository>(value),
    );
  }
}

String _$capabilityRepositoryHash() =>
    r'd6f681223eb9079820d7458aebe88f7342857295';

@ProviderFor(capabilities)
final capabilitiesProvider = CapabilitiesProvider._();

final class CapabilitiesProvider
    extends
        $FunctionalProvider<
          AsyncValue<List<Capability>>,
          List<Capability>,
          FutureOr<List<Capability>>
        >
    with $FutureModifier<List<Capability>>, $FutureProvider<List<Capability>> {
  CapabilitiesProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'capabilitiesProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$capabilitiesHash();

  @$internal
  @override
  $FutureProviderElement<List<Capability>> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<List<Capability>> create(Ref ref) {
    return capabilities(ref);
  }
}

String _$capabilitiesHash() => r'cd4eb52de2d531d6ddc88064b9414abacaa2b98d';
