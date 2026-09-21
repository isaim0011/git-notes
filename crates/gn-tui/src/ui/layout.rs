use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use crate::app::{App, AppMode};
use super::{diff_view, note_panel, input};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = if matches!(app.mode, AppMode::InputBar) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.size());
        
        input::draw(f, app, main_chunks[1]);
        
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(55),
                Constraint::Percentage(25),
            ])
            .split(main_chunks[0])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(55),
                Constraint::Percentage(25),
            ])
            .split(f.size())
    };

    draw_file_tree(f, app, chunks[0]);
    diff_view::draw(f, app, chunks[1]);
    note_panel::draw(f, app, chunks[2]);
}

fn draw_file_tree(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if matches!(app.mode, AppMode::FileTree) {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let items: Vec<ListItem> = app.file_tree.iter().enumerate().map(|(i, entry)| {
        let style = if Some(i) == app.selected_file && matches!(app.mode, AppMode::FileTree) {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else if Some(i) == app.selected_file {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        
        let content = format!("{} [{}]", entry.path, entry.note_count);
        ListItem::new(content).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" Files ").borders(Borders::ALL).border_style(border_style));
        
    f.render_widget(list, area);
}
