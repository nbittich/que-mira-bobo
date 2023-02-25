use std::collections::{BTreeMap, HashMap};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::sparql_context::{self, Mode, SparqlContext};

pub fn draw_app<B: Backend>(frame: &mut Frame<B>, context: &SparqlContext) {
    let frame_size = frame.size();
    let current_mode = match context.mode {
        Some(mode) => mode.clone(),
        None => Mode::Url,
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(frame_size.clone());

    let focus = cursor_focused(context, &chunks[0]);
    let url_paragraph = draw_paragraph(&context.url, draw_block("Sparql endpoint", focus), focus);

    frame.render_widget(url_paragraph, chunks[0]);

    let area_middle = split_rect(70, 30, Direction::Horizontal, chunks[1]);
    let area_left = area_middle[0];

    let focus = cursor_focused(context, &area_left);
    let query_paragraph = draw_paragraph(&context.query, draw_block("Query", focus), focus);

    frame.render_widget(query_paragraph, area_left);

    let area_right = split_rect(50, 50, Direction::Vertical, area_middle[1]);

    let focus = cursor_focused(context, &area_right[0]);
    let prefixes_list = draw_list(&context.prefixes, draw_block("Prefixes", focus), focus);

    frame.render_widget(prefixes_list, area_right[0]);

    let focus = cursor_focused(context, &area_right[1]);
    let prefixes_list = draw_list(&context.saved_queries, draw_block("Queries", focus), focus);

    frame.render_widget(prefixes_list, area_right[1]);

    // DEBUG
    let message = format!(
        r#"
          sparql_endpoint: {:?}
          query: {:?}
          prefixes: {:?}
          queries: {:?}

        "#,
        chunks[0], area_left, area_right[0], area_right[1]
    );
    let debug = draw_paragraph(&message, draw_block("DEBUG", false), false);
    frame.render_widget(debug, chunks[2]);
}

fn draw_paragraph<'a>(content: &'a str, block: Block<'a>, focus: bool) -> Paragraph<'a> {
    Paragraph::new(content)
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
}
fn draw_list<'a>(content: &'a BTreeMap<String, String>, block: Block<'a>, focus: bool) -> List<'a> {
    let list_items: Vec<ListItem> = content
        .keys()
        .map(|k| draw_span(k))
        .map(|s| ListItem::new(s).style(Style::default().fg(Color::Black).bg(Color::White)))
        .into_iter()
        .collect();
    List::new(list_items)
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .block(block)
}
fn draw_block(title: &str, focus: bool) -> Block {
    let border_color = if focus {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let style_modifier = if focus {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(border_color))
        .title(Span::styled(title, style_modifier))
}

fn split_rect(size_first: u16, size_second: u16, direction: Direction, r: Rect) -> [Rect; 2] {
    let area = Layout::default()
        .direction(direction)
        .constraints(
            [
                Constraint::Percentage(size_first),
                Constraint::Percentage(size_second),
            ]
            .as_ref(),
        )
        .split(r);
    [area[0], area[1]]
}

fn draw_span(content: &str) -> Spans {
    Spans::from(Span::styled(
        content,
        Style::default().add_modifier(Modifier::ITALIC),
    ))
}

fn cursor_focused(context: &SparqlContext, rect: &Rect) -> bool {
    let point = Rect {
        x: context.pos_cursor.1,
        y: context.pos_cursor.0,
        width: 1,
        height: 1,
    };
    rect.intersects(point)
}
