mod app;
mod config;
mod event;
mod stats;
mod topology;
mod ui;

use std::io;

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, ViewMode};
use config::Config;
use event::{handle_key, Event, EventHandler, KeyAction};
use topology::detect_topology;
use ui::{render, Theme};

fn main() -> Result<()> {
    // Parse command line arguments
    let config = Config::parse_args();

    // Detect system topology
    let topology = detect_topology().context("Failed to detect CPU topology")?;

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Create application state
    let mut app = App::new(topology);
    let theme = Theme::default();
    let event_handler = EventHandler::new(config.refresh_rate);

    // Main loop
    let result = run_app(&mut terminal, &mut app, &theme, &event_handler);

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    theme: &Theme,
    event_handler: &EventHandler,
) -> Result<()> {
    loop {
        // Render
        terminal.draw(|frame| render(frame, app, theme))?;

        // Handle events
        match event_handler.next()? {
            Event::Key(key) => {
                if app.show_help {
                    // Any key closes help
                    app.toggle_help();
                } else {
                    match handle_key(key) {
                        KeyAction::Quit => app.quit(),
                        KeyAction::ViewCore => app.set_view_mode(ViewMode::Core),
                        KeyAction::ViewCcd => app.set_view_mode(ViewMode::Ccd),
                        KeyAction::ViewNps => app.set_view_mode(ViewMode::Nps),
                        KeyAction::ViewNuma => app.set_view_mode(ViewMode::Numa),
                        KeyAction::ToggleSmt => app.toggle_smt(),
                        KeyAction::CycleDisplayMode => app.cycle_display_mode(),
                        KeyAction::ToggleHelp => app.toggle_help(),
                        KeyAction::ScrollUp => app.scroll_up(),
                        KeyAction::ScrollDown => app.scroll_down(),
                        KeyAction::None => {}
                    }
                }
            }
            Event::Tick => {
                app.refresh_stats();
            }
            Event::Resize(_, _) => {
                // Terminal will handle resize automatically
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
