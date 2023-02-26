#![allow(unused)]
mod app;
mod sparql_context;

use app::draw_app;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use sparql_context::{Mode, SparqlContext};
use std::{collections::BTreeMap, error::Error, io};
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
    let mut sparql_context = mock_initial_sparql_context();
    loop {
        terminal.draw(|f| draw_app(f, &mut sparql_context))?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(())
                }
                KeyCode::Char(c) => match &sparql_context.mode {
                    Some(Mode::Url) => sparql_context.url.push(c),
                    Some(Mode::Query) => sparql_context.query.push(c),
                    _ => {}
                },

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

fn mock_initial_sparql_context() -> SparqlContext {
    let mut sparql_context = SparqlContext::default();
    sparql_context.url = "http://localhost:8093/sparql".into();
    sparql_context.prefixes = BTreeMap::from([
        ("xsd".into(), "http://www.w3.org/2001/XMLSchema#".into()),
        ("mu".into(), "http://mu.semte.ch/vocabularies/core/".into()),
        (
            "persoon".into(),
            "https://data.vlaanderen.be/ns/persoon#".into(),
        ),
        ("ext".into(), "http://mu.semte.ch/vocabularies/ext/".into()),
        ("person".into(), "http://www.w3.org/ns/person#".into()),
        (
            "session".into(),
            "http://mu.semte.ch/vocabularies/session/".into(),
        ),
        ("foaf".into(), "http://xmlns.com/foaf/0.1/".into()),
        (
            "besluit".into(),
            "http://data.vlaanderen.be/ns/besluit#".into(),
        ),
        (
            "ere".into(),
            "http://data.lblod.info/vocabularies/erediensten/".into(),
        ),
        (
            "mandaat".into(),
            "http://data.vlaanderen.be/ns/mandaat#".into(),
        ),
        ("org".into(), "http://www.w3.org/ns/org".into()),
        (
            "generiek".into(),
            "https://data.vlaanderen.be/ns/generiek#".into(),
        ),
    ]);
    sparql_context.query = r#"PREFIX ext: <http://mu.semte.ch/vocabularies/ext/>
    PREFIX person: <http://www.w3.org/ns/person#>
    PREFIX session: <http://mu.semte.ch/vocabularies/session/>
    PREFIX foaf: <http://xmlns.com/foaf/0.1/>
    PREFIX besluit: <http://data.vlaanderen.be/ns/besluit#>
    PREFIX ere: <http://data.lblod.info/vocabularies/erediensten/>
    PREFIX mandaat: <http://data.vlaanderen.be/ns/mandaat#>
    PREFIX org: <http://www.w3.org/ns/org>
    PREFIX generiek: <https://data.vlaanderen.be/ns/generiek#>


    DELETE {
      GRAPH <http://mu.semte.ch/graphs/organisatieportaal> {
        ?governingBody ?p ?o.
      }
    }
    INSERT {
      GRAPH <http://mu.semte.ch/graphs/worship-service> {
        ?governingBody ?p ?o.
      }
    }

    WHERE {
      GRAPH <http://mu.semte.ch/graphs/organisatieportaal> {
        ?governingBody <http://data.lblod.info/vocabularies/erediensten/wordtBediendDoor>
                       ?post; ?p ?o.

        filter exists {
          GRAPH <http://mu.semte.ch/graphs/worship-service> {
            ?mandatories <http://www.w3.org/ns/org#holds> ?post.
        
          }
        }
      }
   }
"#
    .into();
    sparql_context
}
