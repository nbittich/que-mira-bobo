#![allow(unused)]
mod app;
mod sparql_context;

use app::draw_app;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use sparql_context::SparqlContext;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, Paragraph, Wrap},
    Frame, Terminal,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        Clear(crossterm::terminal::ClearType::All),
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut sparql_context = SparqlContext::default();
    loop {
        terminal.draw(|f| draw_app(f, &sparql_context))?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => return Ok(()),
                _ => {}
            },
            Event::Mouse(m) => match m.kind {
                event::MouseEventKind::Down(event::MouseButton::Left) => {
                    let row = m.row;
                    let column = m.column;
                    sparql_context.pos_cursor = (row, column);
                } // released left
                event::MouseEventKind::Down(_) => {} // useless
                event::MouseEventKind::Up(_) => {}   // released
                event::MouseEventKind::Drag(_) => {} // useless
                event::MouseEventKind::Moved => {}   // useless
                event::MouseEventKind::ScrollDown => {} // pagination,
                event::MouseEventKind::ScrollUp => {} // pagination,
            },
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Paste(_) => {}
            Event::Resize(_, _) => {}
        }
    }
}
