// TJ-ARCH-MOB-001 compliant
fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "list_notes",
            "create_note",
            "list_capabilities",
        ]),
    ))
    .expect("Tauri build configuration");
}
