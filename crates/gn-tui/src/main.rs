use anyhow::Result;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{stdout, Stdout};

mod app;
mod events;
mod ui;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create and run app
    let mut app = app::App::new(std::env::current_dir()?)?;
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    restore_terminal(&mut terminal)?;

    res
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut app::App) -> Result<()> {
    let events = events::EventHandler::new(250);

    app.load_notes()?;

    loop {
        terminal.draw(|f| ui::layout::draw(f, app))?;

        match events.next()? {
            events::Event::Key(key) => {
                if !app.handle_key(key) {
                    break;
                }
            }
            events::Event::Tick => {
                app.tick();
            }
            events::Event::Resize(_, _) => {}
        }
    }

    Ok(())
}
