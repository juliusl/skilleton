//! Skill file I/O — reading and writing skills to the directory layout (ADR-0006).
//! Repository pattern for storage and retrieval (ADR-0008).

mod error;
mod file;
mod formats;
mod loader;
mod memory;
mod traits;
mod writer;

pub use error::StorageError;
pub use file::FileRepository;
pub use loader::SkillLoader;
pub use memory::InMemoryRepository;
pub use traits::{ItemRef, RepositoryError, SkillRepository};
pub use writer::SkillWriter;
