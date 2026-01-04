use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};

/// Application events
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// Keyboard event
    Key(KeyEvent),
    /// Terminal tick (for refresh)
    Tick,
    /// Resize event
    Resize(u16, u16),
}

/// Event handler for keyboard input
pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Poll for next event
    pub fn next(&self) -> Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Resize(w, h) => Ok(Event::Resize(w, h)),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}

/// Handle key event and return action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Quit,
    ViewCore,
    ViewCcd,
    ViewNps,
    ViewNuma,
    ToggleSmt,
    ToggleHelp,
    CycleDisplayMode,
    ScrollUp,
    ScrollDown,
    None,
}

pub fn handle_key(key: KeyEvent) -> KeyAction {
    match key.code {
        // Quit
        KeyCode::Char('q') => KeyAction::Quit,
        KeyCode::Esc => KeyAction::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Quit,

        // View modes
        KeyCode::Char('c') => KeyAction::ViewCore,
        KeyCode::Char('d') => KeyAction::ViewCcd,
        KeyCode::Char('n') => KeyAction::ViewNps,
        KeyCode::Char('u') => KeyAction::ViewNuma,

        // SMT toggle
        KeyCode::Char('s') => KeyAction::ToggleSmt,

        // Display mode
        KeyCode::Char('m') => KeyAction::CycleDisplayMode,

        // Help
        KeyCode::Char('h') => KeyAction::ToggleHelp,
        KeyCode::Char('?') => KeyAction::ToggleHelp,

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => KeyAction::ScrollUp,
        KeyCode::Down | KeyCode::Char('j') => KeyAction::ScrollDown,

        _ => KeyAction::None,
    }
}
