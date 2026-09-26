// TJ-ARCH-MOB-001 compliant
#[test]
fn persists_across_independent_connections_and_preserves_order() {
    let directory = tempfile::tempdir().expect("fixture");
    let path = directory.path().join("notes.sqlite3");
    let first = {
        let notes = gen_ui_notes::open(&path).expect("open");
        notes.create("  First note  ").expect("create")
    };
    let notes = gen_ui_notes::open(&path).expect("reopen");
    assert_eq!(notes.list().expect("list"), vec![first]);
    notes.create("Second note").expect("second");
    assert_eq!(notes.list().expect("list")[0].title, "Second note");
}
#[test]
fn rejects_invalid_input_without_persisting_it() {
    let directory = tempfile::tempdir().expect("fixture");
    let notes = gen_ui_notes::open(&directory.path().join("notes.sqlite3")).expect("open");
    assert!(matches!(
        notes.create("  "),
        Err(gen_ui_notes::NoteError::InvalidTitle)
    ));
    assert!(notes.create(&"x".repeat(201)).is_err());
    assert!(notes.list().expect("list").is_empty());
}
#[test]
fn unicode_and_sql_text_remain_data_after_reopen() {
    let directory = tempfile::tempdir().expect("fixture");
    let path = directory.path().join("notes.sqlite3");
    gen_ui_notes::open(&path)
        .expect("open")
        .create("📝 '; DROP TABLE notes; --")
        .expect("create");
    assert_eq!(
        gen_ui_notes::open(&path)
            .expect("reopen")
            .list()
            .expect("list")[0]
            .title,
        "📝 '; DROP TABLE notes; --"
    );
}
