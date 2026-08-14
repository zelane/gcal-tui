//! Rendering.

use chrono::Local;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Row, Table},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        // Constraint::Length(3), // day strip
        Constraint::Min(1),    // agenda
        Constraint::Length(1), // status
    ])
    .split(frame.area());

    // render_day_strip(frame, app, chunks[0]);
    render_agenda(frame, app, chunks[0]);
    render_status(frame, app, chunks[1]);
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" </> ", Style::default().fg(Color::Cyan)),
            Span::raw("day  "),
            Span::styled("t ", Style::default().fg(Color::Cyan)),
            Span::raw("today  "),
            Span::styled("r ", Style::default().fg(Color::Cyan)),
            Span::raw("refresh  "),
            Span::styled("q ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("quit    {}  {}", app.account.identity, app.status)),
        ])),
        area,
    );
}

// fn render_day_strip(frame: &mut Frame, app: &App, area: Rect) {
//     let today = Local::now().date_naive();

//     let mut spans = vec![Span::raw(" ")];
//     for (i, day) in app.days.iter().enumerate() {
//         let label = (*day).format("%a %-d").to_string();
//         let style = if i == app.selected {
//             Style::default()
//                 .fg(Color::Black)
//                 .bg(Color::Cyan)
//                 .add_modifier(Modifier::BOLD)
//         } else if *day == today {
//             Style::default().fg(Color::Cyan)
//         } else {
//             Style::default().fg(Color::DarkGray)
//         };
//         spans.push(Span::styled(format!(" {label} "), style));
//         spans.push(Span::raw(" "));
//     }

//     frame.render_widget(
//         Paragraph::new(Line::from(spans)).block(Block::bordered()),
//         area,
//     );
// }

#[derive(Default)]
struct Widths(Vec<usize>);

impl Widths {
    fn observe(&mut self, cells: &[String]) {
        if self.0.len() < cells.len() {
            self.0.resize(cells.len(), 0);
        }
        for (slot, c) in self.0.iter_mut().zip(cells) {
            *slot = (*slot).max(c.lines().map(UnicodeWidthStr::width).max().unwrap_or(0));
        }
    }
    fn constraints(self) -> Vec<Constraint> {
        self.0
            .into_iter()
            .map(|n| Constraint::Length(n as u16))
            .collect()
    }
}

fn render_agenda(frame: &mut Frame, app: &App, area: Rect) {
    let now = Local::now();
    let day = app.current_day();
    let events = app.events_for_day();

    let title = format!(" {} ", day.format("%A %-d %B %Y"));
    let title_style = if day == now.date_naive() {
        Style::default().fg(Color::LightGreen)
    } else {
        Style::default().fg(Color::LightMagenta)
    };

    let block = Block::bordered()
        .title(Span::styled(
            title,
            title_style.add_modifier(Modifier::BOLD),
        ))
        .padding(Padding::new(1, 1, 1, 1));

    if events.is_empty() {
        let placeholder = if app.fetching {
            "  Loading…"
        } else {
            "  Nothing scheduled"
        };

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                placeholder,
                Style::default().fg(Color::DarkGray),
            )))
            .block(block),
            area,
        );
        return;
    }
    let mut w = Widths::default();

    // let header = ["Start", "End", "Duration", "Summary"];
    // w.observe(&header.map(String::from));

    let items: Vec<Row> = events
        .iter()
        .map(|e| {
            let (cells, past, ongoing) = match (e.start, e.end) {
                (Some(ss), Some(ee)) => {
                    let dur = ee - ss;
                    let dur_string = if dur.num_minutes() < 60 {
                        dur.num_minutes().to_string() + "m"
                    } else {
                        dur.num_hours().to_string() + "h"
                    };
                    (
                        vec![
                            ss.format("%H:%M").to_string(),
                            dur_string,
                            ee.format("%H:%M").to_string(),
                            e.summary.clone(),
                        ],
                        ss < now,
                        ss > now && ee < now && ss.date_naive() == now.date_naive(),
                    )
                }
                (Some(s), None) => (
                    vec![
                        s.format("%H:%M").to_string(),
                        "".to_string(),
                        "".to_string(),
                        e.summary.clone(),
                    ],
                    s < now && s.date_naive() == now.date_naive(),
                    false,
                ),
                _ => (
                    vec![
                        "-".to_string(),
                        "".to_string(), 
                        "".to_string(),
                        e.summary.clone()
                    ],
                    e.date < now.date_naive(),
                    false,
                ),
            };

            let row_style = if ongoing {
                Style::default().fg(Color::LightGreen)
            } else if past {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            w.observe(&cells);
            return Row::new(cells).style(row_style);
        })
        .collect();

    let table = Table::new(items, w.constraints())
        // .header(Row::new(header).style(Style::new().bold()).bottom_margin(0))
        .column_spacing(2)
        .style(Color::White)
        .block(block);

    frame.render_widget(table, area);
}
