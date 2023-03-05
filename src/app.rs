use std::{
    collections::{BTreeMap, HashMap},
    result,
};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};
use tui_textarea::TextArea;

use crate::sparql_context::{self, Mode, SparqlContext, SparqlResponse, SparqlResult};

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
        .split(frame_size);
    let area_middle = split_rect(92, 8, Direction::Vertical, chunks[1]);

    let sparql_fragment = chunks[0];

    let focus = cursor_focused(frame, context, Mode::Url, &sparql_fragment);

    draw_textarea(
        &mut context.url,
        draw_block("Sparql endpoint", focus),
        focus,
    );

    frame.render_widget(context.url.widget(), sparql_fragment);
    let focus = cursor_focused(frame, context, Mode::Query, &area_middle[0]);

    draw_textarea(&mut context.query, draw_block("Query", focus), focus);

    frame.render_widget(context.query.widget(), area_middle[0]);

    // BUTTON
    let button_chunk = split_rect(90, 10, Direction::Horizontal, area_middle[1])[1];
    let focus = cursor_focused(frame, context, Mode::Submit, &button_chunk);
    let paragraph = draw_paragraph("SUBMIT", draw_block("", focus), focus);
    frame.render_widget(paragraph, button_chunk);

    // OUTPUT
    let focus = cursor_focused(frame, context, Mode::Output, &chunks[2]);
    if let Some(response) = &context.output {
        let headers = &response.head.vars;
        let selected_style = Style::default().add_modifier(Modifier::REVERSED);
        let normal_style = Style::default().bg(Color::Blue);
        let header_cells = headers
            .iter()
            .map(|h| Cell::from(h.clone()).style(Style::default().fg(Color::Red)));
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);
        let rows = response.results.bindings.iter().map(|item| {
            let height = item
                .values()
                .map(|content| content.value.chars().filter(|c| *c == '\n').count())
                .max()
                .unwrap_or(0)
                + 1;
            let mut cells = vec![];
            for h in headers {
                cells.push(Text::from(
                    item.get(h)
                        .map(|i| i.value.clone())
                        .unwrap_or_else(String::new),
                ));
            }
            Row::new(cells).height(height as u16).bottom_margin(1)
        });
        let widths_cons: Vec<Constraint> = headers
            .iter()
            .map(|_| Constraint::Percentage(100u16 / (headers.len() as u16)))
            .collect();
        let table = Table::new(rows)
            .header(header)
            .block(draw_block("Response", focus))
            .highlight_style(selected_style)
            .highlight_symbol(">> ")
            .widths(widths_cons.as_slice());

        frame.render_stateful_widget(table, chunks[2], &mut context.output_state);
    } else {
        frame.render_widget(draw_block("Response", false), chunks[2]);
    }
    //let message = format!("{:?}", context.output);
    //let debug = draw_paragraph(&message, draw_block("DEBUG", false), false);
    //frame.render_widget(debug, chunks[2]);
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
