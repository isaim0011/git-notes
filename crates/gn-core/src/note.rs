use crate::namespace::Namespace;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteStatus {
    Open,
    Resolved,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,             // content-addressed UUID v4
    pub commit: String,       // git commit SHA this note is anchored to
    pub file: Option<String>, // relative file path
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub body: String,   // markdown body
    pub author: String, // "Name <email>"
    pub timestamp: DateTime<Utc>,
    pub namespace: Namespace,
    pub thread_id: Option<Uuid>, // for replies — parent note id
    pub status: NoteStatus,
    pub tags: Vec<String>,
}

impl Note {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        commit: String,
        file: Option<String>,
        line_start: Option<u32>,
        line_end: Option<u32>,
        body: String,
        author: String,
        namespace: Namespace,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            commit,
            file,
            line_start,
            line_end,
            body,
            author,
            timestamp: Utc::now(),
            namespace,
            thread_id: None,
            status: NoteStatus::Open,
            tags: Vec::new(),
        }
    }

    pub fn reply(parent: &Note, body: String, author: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            commit: parent.commit.clone(),
            file: parent.file.clone(),
            line_start: parent.line_start,
            line_end: parent.line_end,
            body,
            author,
            timestamp: Utc::now(),
            namespace: parent.namespace.clone(),
            thread_id: Some(parent.id),
            status: NoteStatus::Open,
            tags: Vec::new(),
        }
    }
}
