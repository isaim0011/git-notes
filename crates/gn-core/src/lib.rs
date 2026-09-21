pub mod error;
pub mod namespace;
pub mod note;
pub mod diff;
pub mod merge;
pub mod engine;

pub use error::{GnError, Result};
pub use namespace::Namespace;
pub use note::{Note, NoteStatus};
pub use engine::NotesEngine;
pub use merge::{MergeStrategy, UnionStrategy, LwwStrategy, CompositeStrategy};
