use crate::app::{App, AppMode};
use gn_core::NoteStatus;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if matches!(app.mode, AppMode::NotePanel) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let notes = app.current_file_notes();

    let mut text: Vec<Line> = Vec::new();

    if notes.is_empty() {
        text.push(Line::from(Span::styled(
            "No notes for this file.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (i, note) in notes.iter().enumerate() {
            let is_selected =
                Some(i) == app.selected_note && matches!(app.mode, AppMode::NotePanel);

            // Container header / card separator
            let card_indicator = if is_selected { "▶ " } else { "  " };
            let card_header_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let indent = if note.thread_id.is_some() {
                "    ↳ "
            } else {
                card_indicator
            };

            // Badge styling: [● Open], [✔ Approved], [✗ Rejected], [✓ Resolved]
            let (badge_text, badge_style) = match note.status {
                NoteStatus::Open => (
                    "[● Open]",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                NoteStatus::Approved => (
                    "[✔ Approved]",
                    Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
                ),
                NoteStatus::Rejected => (
                    "[✗ Rejected]",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                NoteStatus::Resolved => (
                    "[✓ Resolved]",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            };

            let timestamp_str = note.timestamp.format("%Y-%m-%d %H:%M").to_string();

            let line_range_str = match (note.line_start, note.line_end) {
                (Some(s), Some(e)) if s == e => format!(" :L{}", s),
                (Some(s), Some(e)) => format!(" :L{}-{}", s, e),
                (Some(s), None) => format!(" :L{}", s),
                _ => String::new(),
            };

            // Header line with Author, Timestamp, Line range, and Status Badge
            text.push(Line::from(vec![
                Span::raw(indent),
                Span::styled(format!("@{} ", note.author), card_header_style.fg(Color::Cyan)),
                Span::styled(format!("({}){} ", timestamp_str, line_range_str), Style::default().fg(Color::DarkGray)),
                Span::styled(badge_text, badge_style),
            ]));

            // Markdown preview rendering of note body
            let body_indent = if note.thread_id.is_some() {
                "      "
            } else {
                "    "
            };

            for line in note.body.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("### ") {
                    text.push(Line::from(vec![
                        Span::raw(body_indent),
                        Span::styled(
                            &trimmed[4..],
                            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        ),
                    ]));
                } else if trimmed.starts_with("## ") {
                    text.push(Line::from(vec![
                        Span::raw(body_indent),
                        Span::styled(
                            &trimmed[3..],
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        ),
                    ]));
                } else if trimmed.starts_with("# ") {
                    text.push(Line::from(vec![
                        Span::raw(body_indent),
                        Span::styled(
                            &trimmed[2..],
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        ),
                    ]));
                } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                    let item_content = &trimmed[2..];
                    let mut spans = vec![
                        Span::raw(body_indent),
                        Span::styled("• ", Style::default().fg(Color::Yellow)),
                    ];
                    spans.extend(render_markdown_inline(item_content));
                    text.push(Line::from(spans));
                } else if trimmed.starts_with('>') {
                    let quote_content = trimmed.trim_start_matches('>').trim_start();
                    let mut spans = vec![
                        Span::raw(body_indent),
                        Span::styled("▌ ", Style::default().fg(Color::DarkGray)),
                    ];
                    spans.extend(render_markdown_inline(quote_content));
                    text.push(Line::from(spans));
                } else if trimmed.starts_with("```") {
                    text.push(Line::from(vec![
                        Span::raw(body_indent),
                        Span::styled(line, Style::default().fg(Color::DarkGray)),
                    ]));
                } else {
                    let mut spans = vec![Span::raw(body_indent)];
                    spans.extend(render_markdown_inline(line));
                    text.push(Line::from(spans));
                }
            }

            // Divider between note cards
            text.push(Line::from(""));
        }
    }

    if matches!(app.mode, AppMode::NotePanel) {
        text.push(Line::from(Span::styled(
            "──────────────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )));
        text.push(Line::from(Span::styled(
            "a=Approve  x=Resolve  r=Reply  j/k=Scroll  q=Back",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD),
        )));
    }

    let p = Paragraph::new(text)
        .block(
            Block::default()
                .title(" Review & Notes ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .scroll((app.note_scroll, 0));

    f.render_widget(p, area);
}

/// Helper function to parse basic inline markdown formatting: `code`, **bold**, *italic*
fn render_markdown_inline(text: &str) -> Vec<Span<'_>> {
    let mut spans = Vec::new();
    let mut rest = text;

    while !rest.is_empty() {
        if let Some(code_start) = rest.find('`') {
            if code_start > 0 {
                spans.push(Span::raw(rest[..code_start].to_string()));
            }
            let after_tick = &rest[code_start + 1..];
            if let Some(code_end) = after_tick.find('`') {
                let code_content = &after_tick[..code_end];
                spans.push(Span::styled(
                    format!(" {} ", code_content),
                    Style::default().fg(Color::Magenta).bg(Color::Rgb(30, 30, 46)),
                ));
                rest = &after_tick[code_end + 1..];
            } else {
                spans.push(Span::raw(rest[code_start..].to_string()));
                break;
            }
        } else if let Some(bold_start) = rest.find("**") {
            if bold_start > 0 {
                spans.push(Span::raw(rest[..bold_start].to_string()));
            }
            let after_bold = &rest[bold_start + 2..];
            if let Some(bold_end) = after_bold.find("**") {
                let bold_content = &after_bold[..bold_end];
                spans.push(Span::styled(
                    bold_content.to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                rest = &after_bold[bold_end + 2..];
            } else {
                spans.push(Span::raw(rest[bold_start..].to_string()));
                break;
            }
        } else {
            spans.push(Span::raw(rest.to_string()));
            break;
        }
    }

    spans
}
