use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let text = vec![Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.input_buffer),
    ])];

    let p = Paragraph::new(text)
        .block(Block::default().title(" Compose Reply ").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow)));

    f.render_widget(p, area);
}
