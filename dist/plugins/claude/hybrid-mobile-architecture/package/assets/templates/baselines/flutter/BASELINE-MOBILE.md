# Flutter/Rust note baseline

Prerequisites: pinned Flutter stable3.47.5 with its Dart3.13.4 SDK, Rust1.97.1, flutter_rust_bridge_codegen2.12.0, plus Android SDK/NDK or Xcode/CocoaPods for the selected device. Run from mobile/: `flutter pub get`, `flutter_rust_bridge_codegen generate`, `dart run build_runner build`, `flutter analyze`, then `flutter run`. Android/iOS host files and the upstream Cargokit plugin are included; Cargokit compiles and packages the Rust library for the host target.

Widget → annotated Riverpod controller → repository port → FRB2.12 → shared Rust note use case → SQLite. Code generation is mandatory after bridge/provider changes. No native library stubs or Dart persistence substitute are used. Stop/relaunch the app to recover notes from the application-support directory. `flutter test integration_test/notes_test.dart -d <device> --no-uninstall` exercises the real native bridge. Execute a second launch to certify restart persistence; widget/unit tests alone do not prove native execution.

UAR is unconfigured and no agent behavior is simulated. Desktop products use the Tauri baseline. Flutter platform tests need compatible Android emulators/devices and an iOS simulator on macOS; physical iOS builds require the user's signing team. Generated Xcode files do not embed a developer team.

Restart certification: after the integration test saves its note, stop the application completely and repeat the same test with `--no-uninstall --dart-define=VERIFY_RESTART=true`. The second phase requires the existing native record and makes no write. Keep the same application data; uninstalling between phases invalidates the proof. Flutter integration tests uninstall by default, so both phases require --no-uninstall.

The pinned SDK and generator must be on the same PATH: use the Dart binary shipped with the selected Flutter SDK. Local baseline verification used Flutter3.48.0-0.5.pre; the package authority pins current stable3.47.5, so exact-pin native CI remains separate evidence.
