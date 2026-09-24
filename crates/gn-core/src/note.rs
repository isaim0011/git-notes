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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_new_with_all_fields() {
        let commit = "a1b2c3d4e5f6".to_string();
        let file = Some("src/main.rs".to_string());
        let line_start = Some(10);
        let line_end = Some(20);
        let body = "This is a test note body.".to_string();
        let author = "Tester <test@example.com>".to_string();
        let namespace = Namespace::Comments;

        let note = Note::new(
            commit.clone(),
            file.clone(),
            line_start,
            line_end,
            body.clone(),
            author.clone(),
            namespace.clone(),
        );

        assert!(!note.id.is_nil());
        assert_eq!(note.commit, commit);
        assert_eq!(note.file, file);
        assert_eq!(note.line_start, line_start);
        assert_eq!(note.line_end, line_end);
        assert_eq!(note.body, body);
        assert_eq!(note.author, author);
        assert_eq!(note.namespace, namespace);
        assert_eq!(note.thread_id, None);
        assert_eq!(note.status, NoteStatus::Open);
        assert!(note.tags.is_empty());
    }

    #[test]
    fn test_note_new_with_optional_fields_none() {
        let commit = "a1b2c3d4e5f6".to_string();
        let body = "Note without file or lines.".to_string();
        let author = "Tester <test@example.com>".to_string();
        let namespace = Namespace::Comments;

        let note = Note::new(
            commit.clone(),
            None,
            None,
            None,
            body.clone(),
            author.clone(),
            namespace.clone(),
        );

        assert!(!note.id.is_nil());
        assert_eq!(note.file, None);
        assert_eq!(note.line_start, None);
        assert_eq!(note.line_end, None);
        assert_eq!(note.status, NoteStatus::Open);
    }

    #[test]
    fn test_note_reply() {
        let parent = Note::new(
            "a1b2c3d4e5f6".to_string(),
            Some("src/lib.rs".to_string()),
            Some(1),
            Some(5),
            "Parent note".to_string(),
            "Parent Author <parent@example.com>".to_string(),
            Namespace::Comments,
        );

        let reply_body = "This is a reply".to_string();
        let reply_author = "Replier <replier@example.com>".to_string();

        let reply = Note::reply(&parent, reply_body.clone(), reply_author.clone());

        assert_ne!(reply.id, parent.id);
        assert!(!reply.id.is_nil());
        assert_eq!(reply.commit, parent.commit);
        assert_eq!(reply.file, parent.file);
        assert_eq!(reply.line_start, parent.line_start);
        assert_eq!(reply.line_end, parent.line_end);
        assert_eq!(reply.namespace, parent.namespace);
        assert_eq!(reply.thread_id, Some(parent.id));
        assert_eq!(reply.body, reply_body);
        assert_eq!(reply.author, reply_author);
        assert_eq!(reply.status, NoteStatus::Open);
        assert!(reply.tags.is_empty());
    }
}
