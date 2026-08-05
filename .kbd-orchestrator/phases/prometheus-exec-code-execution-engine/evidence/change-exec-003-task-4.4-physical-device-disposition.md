# Change EXEC-003 task 4.4 — physical-device evidence disposition

Date: 2026-08-04

Status: **PENDING EVIDENCE**

## Required evidence

Mobile runtime certification requires a receipt-producing execution round trip
on each of:

- a physical iOS device;
- a physical Android device.

The round trip must return the component value, ordered lifecycle events, a
terminal signed receipt, resolvable artifacts, and successful public-key-only
offline verification. Simulator, emulator, host-native, and cross-build results
do not satisfy this requirement.

## Host enumeration

Commands run locally:

```bash
xcrun xctrace list devices
adb devices -l
system_profiler SPUSBDataType
```

Redacted findings:

- Xcode reports one paired physical iPhone under `Devices Offline`.
- No physical iOS device is online or available as a run destination.
- ADB reports an empty attached-device list.
- USB enumeration contains no connected iOS or Android device usable for the
  required run.
- Installed Apple simulators were deliberately excluded from certification.
- The ADB server started by enumeration was stopped after the check; no
  temporary device process was left running.

## Disposition

No physical-device execution was attempted because neither required device was
available. iOS and Android Tier W runtime certification therefore remain
`pending_evidence`.

The following narrower evidence remains valid and separately labeled:

- iOS arm64 and Android arm64 release cross-builds pass;
- both target graphs select Pulley execution with `jit_permitted=false`;
- the corrected dispatcher-retaining `skill-ffi` deltas **fail** the 12 MiB
  limit, so mobile Tier W is not release-ready even after device evidence is
  collected;
- host-native embedded/FFI fixtures exercise returned values, ordered events,
  signed receipts, artifact retrieval, interruption recovery, and offline
  verification.

This disposition must be replaced or supplemented with real device receipts
before either mobile runtime is described as certified or release-ready.
