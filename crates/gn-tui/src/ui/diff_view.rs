use crate::app::{App, AppMode};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::fs;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = matches!(app.mode, AppMode::DiffView);
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let selected_file_path = app
        .selected_file
        .and_then(|idx| app.file_tree.get(idx))
        .map(|entry| entry.path.clone());

    let title = if let Some(ref path) = selected_file_path {
        format!(" Diff: {} ", path)
    } else {
        " Diff ".to_string()
    };

    let notes = app.current_file_notes();

    let mut lines_rendered: Vec<Line> = Vec::new();

    if let Some(ref rel_path) = selected_file_path {
        let full_path = app.repo_path.join(rel_path);
        match fs::read_to_string(&full_path) {
            Ok(content) => {
                let extension = Path::new(rel_path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                let ps = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();
                let syntax = ps
                    .find_syntax_by_extension(extension)
                    .unwrap_or_else(|| ps.find_syntax_plain_text());
                let theme = &ts.themes["base16-ocean.dark"];
                let mut highlighter = HighlightLines::new(syntax, theme);

                for (idx, raw_line) in content.lines().enumerate() {
                    let line_num = (idx + 1) as u32;

                    // Check for notes attached to this line
                    let has_note = notes.iter().any(|n| {
                        if let Some(start) = n.line_start {
                            let end = n.line_end.unwrap_or(start);
                            line_num >= start && line_num <= end
                        } else {
                            false
                        }
                    });

                    let (gutter_bullet, gutter_style) = if has_note {
                        // Highlight gutter with bright cyan bullet and bright yellow line number
                        ("●", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    } else {
                        (" ", Style::default().fg(Color::DarkGray))
                    };

                    let line_num_style = if has_note {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let mut spans = vec![
                        Span::styled(format!("{} ", gutter_bullet), gutter_style),
                        Span::styled(format!("{:>4} │ ", line_num), line_num_style),
                    ];

                    // Syntax highlight the code line
                    if let Ok(ranges) = highlighter.highlight_line(raw_line, &ps) {
                        for (style, text) in ranges {
                            let fg_color = Color::Rgb(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                            );
                            spans.push(Span::styled(text.to_string(), Style::default().fg(fg_color)));
                        }
                    } else {
                        spans.push(Span::raw(raw_line.to_string()));
                    }

                    lines_rendered.push(Line::from(spans));
                }
            }
            Err(_) => {
                // If file cannot be read from disk, show friendly placeholder/diff message
                lines_rendered.push(Line::from(Span::styled(
                    format!("Unable to read file '{}' from disk.", full_path.display()),
                    Style::default().fg(Color::Red),
                )));
            }
        }
    } else {
        lines_rendered.push(Line::from(Span::styled(
            "No file selected.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let p = Paragraph::new(lines_rendered)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .scroll((app.diff_scroll, 0));

    f.render_widget(p, area);
}
