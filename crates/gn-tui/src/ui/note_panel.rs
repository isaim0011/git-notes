use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::{App, AppMode};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if matches!(app.mode, AppMode::NotePanel) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let notes = app.current_file_notes();
    
    let mut text = Vec::new();
    
    if notes.is_empty() {
        text.push(Line::from("No notes for this file."));
    } else {
        for (i, note) in notes.iter().enumerate() {
            let is_selected = Some(i) == app.selected_note && matches!(app.mode, AppMode::NotePanel);
            let style = if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let indent = if note.thread_id.is_some() { "  ↳ " } else { "" };
            
            let status_color = match format!("{:?}", note.status).to_lowercase().as_str() {
                "open" => Color::Green,
                "approved" => Color::Blue,
                "rejected" => Color::Red,
                "resolved" => Color::Gray,
                _ => Color::White,
            };

            text.push(Line::from(vec![
                Span::raw(indent),
                Span::styled(format!("[{}] ", note.author), style.fg(Color::Cyan)),
                Span::styled(format!("[{}] ", note.timestamp), style.fg(Color::DarkGray)),
                Span::styled(format!("[{:?}]", note.status), style.fg(status_color)),
            ]));
            
            text.push(Line::from(vec![
                Span::raw(indent),
                Span::styled(note.body.clone(), style),
            ]));
            
            text.push(Line::from(""));
        }
    }
    
    if matches!(app.mode, AppMode::NotePanel) {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled("r=Reply  x=Resolve  q=Back", Style::default().fg(Color::DarkGray))));
    }

    let p = Paragraph::new(text)
        .block(Block::default().title(" Comments ").borders(Borders::ALL).border_style(border_style));

    f.render_widget(p, area);
}
