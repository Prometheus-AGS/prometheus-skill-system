// Generated from /Users/gqadonis/Projects/hybrid-mobile-architecture-src/assets/templates/command-contract/commands.json; do not edit.
pub const GENERATED_COMMANDS: &[&str] = &[
    "uar_run",
    "uar_cancel",
    "uar_recover",
];

#[macro_export]
macro_rules! generated_invoke_handler {
    () => { tauri::generate_handler![uar_run, uar_cancel, uar_recover] };
}
