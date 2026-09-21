use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::{App, AppMode};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if matches!(app.mode, AppMode::DiffView) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let title = if let Some(idx) = app.selected_file {
        if let Some(entry) = app.file_tree.get(idx) {
            format!(" Diff: {} ", entry.path)
        } else {
            " Diff ".to_string()
        }
    } else {
        " Diff ".to_string()
    };

    let notes = app.current_file_notes();

    // Mock diff content
    let mock_lines = vec![
        " fn main() {",
        "     println!(\"Hello World\");",
        " }",
    ];

    let mut text = Vec::new();
    for (i, line) in mock_lines.iter().enumerate() {
        let line_num = (i + 1) as u32;
        
        let has_note = notes.iter().any(|n| n.line_start == Some(line_num));
        let gutter_mark = if has_note { "●" } else { " " };
        let gutter_style = if has_note { Style::default().fg(Color::Red) } else { Style::default() };
        
        text.push(Line::from(vec![
            Span::styled(format!("{} {:>3} | ", gutter_mark, line_num), gutter_style),
            Span::raw(*line),
        ]));
    }

    let p = Paragraph::new(text)
        .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));

    f.render_widget(p, area);
}
