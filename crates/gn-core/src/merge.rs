use crate::note::Note;
use std::collections::HashMap;

pub trait MergeStrategy: Send + Sync {
    fn merge(&self, local: &[Note], remote: &[Note]) -> Vec<Note>;
    fn name(&self) -> &'static str;
}

pub struct UnionStrategy;

impl MergeStrategy for UnionStrategy {
    fn merge(&self, local: &[Note], remote: &[Note]) -> Vec<Note> {
        let mut notes_map = HashMap::new();
        
        for note in local {
            notes_map.insert(note.id, note.clone());
        }
        for note in remote {
            notes_map.insert(note.id, note.clone());
        }

        notes_map.into_values().collect()
    }

    fn name(&self) -> &'static str {
        "union"
    }
}

pub struct LwwStrategy;

impl MergeStrategy for LwwStrategy {
    fn merge(&self, local: &[Note], remote: &[Note]) -> Vec<Note> {
        let mut notes_map: HashMap<_, Note> = HashMap::new();

        for note in local.iter().chain(remote.iter()) {
            notes_map
                .entry(note.id)
                .and_modify(|existing| {
                    if note.timestamp > existing.timestamp {
                        *existing = note.clone();
                    }
                })
                .or_insert_with(|| note.clone());
        }

        notes_map.into_values().collect()
    }

    fn name(&self) -> &'static str {
        "lww"
    }
}

pub struct CompositeStrategy;

impl MergeStrategy for CompositeStrategy {
    fn merge(&self, local: &[Note], remote: &[Note]) -> Vec<Note> {
        // Simple composite: LWW handles existing overlapping notes, union effectively merges disjoint notes.
        // For our implementation, LwwStrategy does both.
        let lww = LwwStrategy;
        lww.merge(local, remote)
    }

    fn name(&self) -> &'static str {
        "composite"
    }
}
