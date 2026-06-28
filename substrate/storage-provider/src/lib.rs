pub mod automerge_engine;
pub mod iroh_docs;
pub mod local_dir;
pub mod traits;

pub use automerge_engine::AutomergeEngine;
pub use iroh_docs::IrohDocsAdapter;
pub use local_dir::LocalDirAdapter;
pub use traits::{CrdtEngine, StorageError, StorageProvider};
