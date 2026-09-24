use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};
use gn_core::note::Note;
use std::io::{stdout, Write};

/// Interactively prompt the user to pick a note from a list using arrow keys and Enter.
/// If user presses Esc or 'q', returns Ok(None).
pub fn pick_note<'a>(title: &str, notes: &'a [Note]) -> Result<Option<&'a Note>> {
    if notes.is_empty() {
        return Ok(None);
    }
    if notes.len() == 1 {
        return Ok(Some(&notes[0]));
    }

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, cursor::Hide)?;

    let mut selected = notes.len() - 1; // Default to latest

    let render = |stdout: &mut std::io::Stdout, sel: usize| -> Result<()> {
        execute!(stdout, cursor::MoveToColumn(0), terminal::Clear(ClearType::FromCursorDown))?;
        println!("\x1b[1;36m? {}\x1b[0m \x1b[90m(Use ↑/↓ arrows, Enter to select, Esc to cancel)\x1b[0m\r", title);

        let start = if sel > 5 { sel - 5 } else { 0 };
        let end = (start + 8).min(notes.len());

        for i in start..end {
            let n = &notes[i];
            let id_short = &n.id.to_string()[..8];
            let file = n.file.as_deref().unwrap_or("-");
            let line = n.line_start.unwrap_or(0);
            let author = n.author.split('<').next().unwrap_or(&n.author).trim();
            let body_preview = n.body.replace('\n', " ");
            let body_short = if body_preview.len() > 38 {
                format!("{}...", &body_preview[..35])
            } else {
                body_preview
            };

            if i == sel {
                println!(
                    "  \x1b[36m❯ [{}] {:<8} {:<18} {:<12} \"{}\"\x1b[0m\r",
                    i + 1,
                    id_short,
                    format!("{}:{}", file, line),
                    author,
                    body_short
                );
            } else {
                println!(
                    "    [{}] {:<8} {:<18} {:<12} \"{}\"\r",
                    i + 1,
                    id_short,
                    format!("{}:{}", file, line),
                    author,
                    body_short
                );
            }
        }
        stdout.flush()?;
        Ok(())
    };

    render(&mut stdout, selected)?;

    let result = loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                        // Move cursor back up
                        let lines_printed = ((selected + 1 - (if selected > 5 { selected - 5 } else { 0 })) + 2) as u16;
                        execute!(stdout, cursor::MoveUp(lines_printed.min(10)))?;
                        render(&mut stdout, selected)?;
                    }
                }
                KeyCode::Down => {
                    if selected + 1 < notes.len() {
                        selected += 1;
                        let lines_printed = ((selected - (if selected > 5 { selected - 5 } else { 0 })) + 2) as u16;
                        execute!(stdout, cursor::MoveUp(lines_printed.min(10)))?;
                        render(&mut stdout, selected)?;
                    }
                }
                KeyCode::Enter => {
                    break Ok(Some(&notes[selected]));
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    break Ok(None);
                }
                _ => {}
            }
        }
    };

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    println!();
    result
}
