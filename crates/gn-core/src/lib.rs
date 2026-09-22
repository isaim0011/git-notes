pub mod diff;
pub mod engine;
pub mod error;
pub mod merge;
pub mod namespace;
pub mod note;

pub use engine::NotesEngine;
pub use error::{GnError, Result};
pub use merge::{CompositeStrategy, LwwStrategy, MergeStrategy, UnionStrategy};
pub use namespace::Namespace;
pub use note::{Note, NoteStatus};
