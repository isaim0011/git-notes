use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use gn_core::{Note, NotesEngine};
use std::collections::HashMap;
use std::path::PathBuf;

pub enum AppMode {
    FileTree,
    DiffView,
    NotePanel,
    InputBar,
}

pub struct FileEntry {
    pub path: String,
    pub note_count: usize,
}

pub struct App {
    pub mode: AppMode,
    pub file_tree: Vec<FileEntry>,
    pub selected_file: Option<usize>,
    pub notes: Vec<Note>,
    pub notes_by_file: HashMap<String, Vec<Note>>,
    pub selected_note: Option<usize>,
    pub input_buffer: String,
    pub repo_path: PathBuf,
}

impl App {
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        Ok(Self {
            mode: AppMode::FileTree,
            file_tree: Vec::new(),
            selected_file: None,
            notes: Vec::new(),
            notes_by_file: HashMap::new(),
            selected_note: None,
            input_buffer: String::new(),
            repo_path,
        })
    }

    #[allow(dead_code)]
    pub fn set_notes(&mut self, notes: Vec<Note>) {
        self.notes = notes;
        self.rebuild_notes_by_file();
    }

    #[allow(dead_code)]
    pub fn rebuild_notes_by_file(&mut self) {
        let mut notes_by_file: HashMap<String, Vec<Note>> = HashMap::new();
        for note in &self.notes {
            if let Some(file) = &note.file {
                notes_by_file
                    .entry(file.clone())
                    .or_default()
                    .push(note.clone());
            }
        }
        self.notes_by_file = notes_by_file;
    }

    pub fn load_notes(&mut self) -> Result<()> {
        let engine = NotesEngine::new(&self.repo_path);

        // This is a naive load, assuming we merge all namespaces
        let mut all_notes = Vec::new();
        for ns in &["comments", "review", "todos"] {
            let namespace_enum = gn_core::Namespace::Custom(ns.to_string());
            if let Ok(notes) = engine.read_notes(&namespace_enum) {
                all_notes.extend(notes);
            }
        }

        // Group by file
        let mut file_counts: HashMap<String, usize> = HashMap::new();
        let mut notes_by_file: HashMap<String, Vec<Note>> = HashMap::new();
        for note in &all_notes {
            if let Some(file) = &note.file {
                *file_counts.entry(file.clone()).or_insert(0) += 1;
                notes_by_file
                    .entry(file.clone())
                    .or_default()
                    .push(note.clone());
            }
        }

        self.file_tree = file_counts
            .into_iter()
            .map(|(path, note_count)| FileEntry { path, note_count })
            .collect();
        self.file_tree.sort_by(|a, b| a.path.cmp(&b.path));

        if !self.file_tree.is_empty() {
            self.selected_file = Some(0);
        }

        self.notes = all_notes;
        self.notes_by_file = notes_by_file;

        Ok(())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.mode {
            AppMode::FileTree => match key.code {
                KeyCode::Char('q') => return false,
                KeyCode::Down => {
                    if let Some(i) = self.selected_file {
                        if i < self.file_tree.len().saturating_sub(1) {
                            self.selected_file = Some(i + 1);
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(i) = self.selected_file {
                        if i > 0 {
                            self.selected_file = Some(i - 1);
                        }
                    }
                }
                KeyCode::Right | KeyCode::Enter => {
                    self.mode = AppMode::DiffView;
                }
                _ => {}
            },
            AppMode::DiffView => match key.code {
                KeyCode::Char('q') | KeyCode::Left => self.mode = AppMode::FileTree,
                KeyCode::Right | KeyCode::Enter => {
                    self.mode = AppMode::NotePanel;
                    if !self.current_file_notes().is_empty() {
                        self.selected_note = Some(0);
                    }
                }
                _ => {}
            },
            AppMode::NotePanel => match key.code {
                KeyCode::Char('q') | KeyCode::Left => {
                    self.mode = AppMode::DiffView;
                    self.selected_note = None;
                }
                KeyCode::Down => {
                    if let Some(i) = self.selected_note {
                        if i < self.current_file_notes().len().saturating_sub(1) {
                            self.selected_note = Some(i + 1);
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(i) = self.selected_note {
                        if i > 0 {
                            self.selected_note = Some(i - 1);
                        }
                    }
                }
                KeyCode::Char('r') => {
                    self.mode = AppMode::InputBar;
                    self.input_buffer.clear();
                }
                _ => {}
            },
            AppMode::InputBar => match key.code {
                KeyCode::Esc => {
                    self.mode = AppMode::NotePanel;
                    self.input_buffer.clear();
                }
                KeyCode::Enter => {
                    // In a real app we'd save the reply
                    self.mode = AppMode::NotePanel;
                    self.input_buffer.clear();
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                _ => {}
            },
        }
        true
    }

    pub fn tick(&mut self) {
        // Handle tick events
    }

    pub fn current_file_notes(&self) -> Vec<Note> {
        if let Some(idx) = self.selected_file {
            if let Some(entry) = self.file_tree.get(idx) {
                if let Some(notes) = self.notes_by_file.get(&entry.path) {
                    return notes.clone();
                }
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gn_core::Namespace;

    #[test]
    fn test_current_file_notes_indexed() {
        let mut app = App::new(PathBuf::from(".")).unwrap();
        app.file_tree = vec![
            FileEntry {
                path: "src/main.rs".to_string(),
                note_count: 2,
            },
            FileEntry {
                path: "src/lib.rs".to_string(),
                note_count: 1,
            },
        ];
        app.selected_file = Some(0);

        let note1 = Note::new(
            "sha1".to_string(),
            Some("src/main.rs".to_string()),
            Some(1),
            Some(5),
            "Note 1".to_string(),
            "Author".to_string(),
            Namespace::Custom("comments".to_string()),
        );
        let note2 = Note::new(
            "sha1".to_string(),
            Some("src/main.rs".to_string()),
            Some(10),
            Some(15),
            "Note 2".to_string(),
            "Author".to_string(),
            Namespace::Custom("comments".to_string()),
        );
        let note3 = Note::new(
            "sha1".to_string(),
            Some("src/lib.rs".to_string()),
            Some(2),
            Some(4),
            "Note 3".to_string(),
            "Author".to_string(),
            Namespace::Custom("comments".to_string()),
        );

        app.set_notes(vec![note1.clone(), note2.clone(), note3.clone()]);

        let main_notes = app.current_file_notes();
        assert_eq!(main_notes.len(), 2);
        assert_eq!(main_notes[0].body, "Note 1");
        assert_eq!(main_notes[1].body, "Note 2");

        app.selected_file = Some(1);
        let lib_notes = app.current_file_notes();
        assert_eq!(lib_notes.len(), 1);
        assert_eq!(lib_notes[0].body, "Note 3");
    }
}
