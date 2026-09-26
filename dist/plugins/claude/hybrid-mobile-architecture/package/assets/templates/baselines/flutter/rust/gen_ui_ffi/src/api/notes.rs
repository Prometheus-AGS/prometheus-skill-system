// TJ-ARCH-MOB-001 compliant
#[derive(Clone, Debug)]
pub struct NoteDto {
    pub id: String,
    pub title: String,
}
impl From<gen_ui_notes::Note> for NoteDto {
    fn from(note: gen_ui_notes::Note) -> Self {
        Self {
            id: note.id,
            title: note.title,
        }
    }
}
pub fn list_notes(database_path: String) -> Result<Vec<NoteDto>, String> {
    gen_ui_notes::open(std::path::Path::new(&database_path))
        .and_then(|notes| notes.list())
        .map(|notes| notes.into_iter().map(NoteDto::from).collect())
        .map_err(|error| error.to_string())
}
pub fn create_note(database_path: String, title: String) -> Result<NoteDto, String> {
    gen_ui_notes::open(std::path::Path::new(&database_path))
        .and_then(|notes| notes.create(&title))
        .map(NoteDto::from)
        .map_err(|error| error.to_string())
}

#[derive(Clone, Debug)]
pub struct CapabilityDto {
    pub id: String,
    pub name: String,
    pub kind: String,
}
pub fn list_capabilities() -> Result<Vec<CapabilityDto>, String> {
    gen_ui_notes::capabilities()
        .map(|items| {
            items
                .into_iter()
                .map(|c| CapabilityDto {
                    id: c.id,
                    name: c.name,
                    kind: c.kind,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}
