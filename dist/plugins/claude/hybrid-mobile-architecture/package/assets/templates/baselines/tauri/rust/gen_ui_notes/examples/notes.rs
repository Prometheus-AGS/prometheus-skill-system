// TJ-ARCH-MOB-001 compliant
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = PathBuf::from(args.next().ok_or("usage: notes <database> [title]")?);
    let service = gen_ui_notes::open(&path)?;
    if let Some(title) = args.next() {
        service.create(&title)?;
    }
    for note in service.list()? {
        println!("{}\t{}", note.id, note.title);
    }
    Ok(())
}
