#![allow(unused)]
mod app;
mod sparql_context;

use app::draw_app;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use reqwest::{blocking, header::CONTENT_TYPE};
use sparql_context::{Mode, SparqlContext, SparqlResponse};
use std::{collections::BTreeMap, error::Error, io, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, Paragraph, Wrap},
    Frame, Terminal,
};
use tui_textarea::{Input, Key, TextArea};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableBracketedPaste,
        EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        Clear(crossterm::terminal::ClearType::Purge),
        LeaveAlternateScreen,
        Clear(crossterm::terminal::ClearType::All),
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    terminal.clear()?;
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
        #[allow(clippy::single_match)]
        match sparql_context.mode {
            Some(Mode::Submit) => {
                execute_query(&mut sparql_context);
            }
            _ => {}
        }
        match event::read()? {
            Event::Key(key) => {
                if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(());
                }
                if matches!(sparql_context.mode, Some(Mode::Url)) {
                    let textarea = &mut sparql_context.url;
                    if key.code != KeyCode::Enter {
                        textarea.input(key);
                    }
                }
                if matches!(sparql_context.mode, Some(Mode::Query)) {
                    let textarea = &mut sparql_context.query;
                    textarea.input(key);
                }
            }
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
            Event::Paste(_) => todo!(),
            Event::Resize(_, _) => {}
        }
    }
}

fn mock_initial_sparql_context<'a>() -> SparqlContext<'a> {
    SparqlContext::<'_> {
        url: TextArea::from(["http://localhost:8890/sparql"]),
        query: TextArea::from(
            r#"PREFIX ext: <http://mu.semte.ch/vocabularies/ext/>
      PREFIX person: <http://www.w3.org/ns/person#>
      PREFIX session: <http://mu.semte.ch/vocabularies/session/>
      PREFIX foaf: <http://xmlns.com/foaf/0.1/>
      PREFIX besluit: <http://data.vlaanderen.be/ns/besluit#>
      PREFIX ere: <http://data.lblod.info/vocabularies/erediensten/>
      PREFIX mandaat: <http://data.vlaanderen.be/ns/mandaat#>
      PREFIX org: <http://www.w3.org/ns/org>
      PREFIX generiek: <https://data.vlaanderen.be/ns/generiek#>

      select * where {
          graph ?g {
              ?s ?p ?o
          }
      } limit 1

      "#
            .lines(),
        ),
        ..Default::default()
    }
}

fn execute_query(context: &mut SparqlContext) {
    let uri = context.url.lines()[0].trim();
    let client = reqwest::blocking::Client::builder()
        .use_rustls_tls()
        .timeout(Duration::from_secs(300))
        .build()
        .unwrap();
    let response = client
        .post(uri)
        .header(CONTENT_TYPE, "application/sparql-results+json")
        .query(&[
            ("query", context.query.lines().join("\n")),
            ("format", "application/sparql-results+json".into()),
        ])
        .send()
        .unwrap();
    //dbg!(response.text().unwrap());
    let result: SparqlResponse = response.json().unwrap();
    context.output = Some(result);
    context.mode = None;
    context.pos_cursor = (0, 0);
}
