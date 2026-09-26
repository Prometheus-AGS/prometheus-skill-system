// TJ-ARCH-MOB-001 compliant
use gen_ui_types::{Note, NoteError, NoteRepository, NoteTitle};
use rusqlite::{Connection, params};
use std::{path::Path, time::Duration};
pub struct SqliteNotes {
    connection: Connection,
}
fn storage(error: impl std::fmt::Display) -> NoteError {
    NoteError::Storage(error.to_string())
}
impl SqliteNotes {
    pub fn open(path: &Path) -> Result<Self, NoteError> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(storage)?;
        }
        let connection = Connection::open(path).map_err(storage)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(storage)?;
        connection
            .execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")
            .map_err(storage)?;
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(storage)?;
        if version > 1 {
            return Err(NoteError::Storage(
                "database schema is newer than this application".to_owned(),
            ));
        }
        if version == 0 {
            connection.execute_batch("BEGIN IMMEDIATE; CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL); PRAGMA user_version=1; COMMIT;").map_err(storage)?;
        }
        Ok(Self { connection })
    }
}
impl NoteRepository for SqliteNotes {
    fn list(&self) -> Result<Vec<Note>, NoteError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, title FROM notes ORDER BY id DESC")
            .map_err(storage)?;
        statement
            .query_map([], |row| {
                Ok(Note {
                    id: row.get::<_, i64>(0)?.to_string(),
                    title: row.get(1)?,
                })
            })
            .map_err(storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage)
    }
    fn create(&self, title: &NoteTitle) -> Result<Note, NoteError> {
        self.connection
            .execute(
                "INSERT INTO notes (title) VALUES (?1)",
                params![title.as_str()],
            )
            .map_err(storage)?;
        Ok(Note {
            id: self.connection.last_insert_rowid().to_string(),
            title: title.as_str().to_owned(),
        })
    }
}
