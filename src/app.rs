use std::collections::{BTreeMap, HashMap};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tui_textarea::TextArea;

use crate::sparql_context::{self, Mode, SparqlContext};

pub fn draw_app<B: Backend>(frame: &mut Frame<B>, context: &mut SparqlContext) {
    let frame_size = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(8),
                Constraint::Percentage(62),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(frame_size.clone());
    let area_middle = split_rect(70, 30, Direction::Horizontal, chunks[1]);

    let area_right = split_rect(50, 50, Direction::Vertical, area_middle[1]);

    let sparql_fragment = chunks[0];

    let query_fragment = area_middle[0];

    let prefixes_fragment = area_right[0];
    let saved_queries_fragement = area_right[1];

    let focus = cursor_focused(frame, context, Mode::Url, &sparql_fragment);

    draw_textarea(
        &mut context.url,
        draw_block("Sparql endpoint", focus),
        focus,
    );

    frame.render_widget(context.url.widget(), sparql_fragment);
    let focus = cursor_focused(frame, context, Mode::Query, &query_fragment);
    draw_textarea(&mut context.query, draw_block("Query", focus), focus);

    frame.render_widget(context.query.widget(), query_fragment);

    let focus = cursor_focused(frame, context, Mode::SavedPrefixes, &prefixes_fragment);
    let prefixes_list = draw_list(&context.prefixes, draw_block("Prefixes", focus), focus);

    frame.render_widget(prefixes_list, prefixes_fragment);

    let focus = cursor_focused(frame, context, Mode::SavedQueries, &saved_queries_fragement);
    let query_list = draw_list(&context.saved_queries, draw_block("Queries", focus), focus);

    frame.render_widget(query_list, saved_queries_fragement);

    // DEBUG
    let message = format!(
        r#"
          sparql_endpoint: {:?}
          query: {:?}
          prefixes: {:?}
          queries: {:?}

        "#,
        chunks[0], query_fragment, prefixes_fragment, saved_queries_fragement
    );
    let debug = draw_paragraph(&message, draw_block("DEBUG", false), false);
    frame.render_widget(debug, chunks[2]);
}
fn draw_textarea<'a>(textarea: &mut TextArea<'a>, block: Block<'a>, focus: bool) {
    textarea.set_block(block);
    if !focus {
        textarea.set_cursor_style(textarea.style());
    } else {
        textarea.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));
    }
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

fn cursor_focused<B: Backend>(
    frame: &mut Frame<B>,
    context: &mut SparqlContext,
    mode_if_focus: Mode,
    rect: &Rect,
) -> bool {
    let rect = rect.inner(&DEFAULT_MARGIN);
    let point = Rect {
        x: context.pos_cursor.1,
        y: context.pos_cursor.0,
        width: 1,
        height: 1,
    };
    let is_focused = rect.intersects(point);
    if is_focused {
        context.mode = Some(mode_if_focus);
    }
    is_focused
}

const DEFAULT_MARGIN: Margin = Margin {
    vertical: 1,
    horizontal: 1,
};
