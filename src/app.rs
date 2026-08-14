//! Application state and the event loop.

use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use chrono::{Duration, Local, NaiveDate};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use zbus::blocking::Connection;

use crate::cal::{self, CalEvent};
use crate::goa::{access_token, Account};
use crate::ui;

const DAYS_BACK: i64 = 1;
const DAYS_FORWARD: i64 = 5;

enum Msg {
    Input(Event),
    Fetched(Result<Vec<CalEvent>, String>),
}

fn spawn_input_thread(tx: Sender<Msg>) {
    thread::spawn(move || loop {
        match event::read() {
            Ok(ev) => {
                if tx.send(Msg::Input(ev)).is_err() {
                    return;
                }
            }
            Err(_) => return,
        }
    });
}

pub struct App {
    conn: Connection,
    pub account: Account,
    pub days: Vec<NaiveDate>,
    pub selected: usize,
    events: Vec<CalEvent>,
    pub status: String,
    pub fetching: bool,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl App {
    pub fn new(conn: Connection, account: Account) -> Self {
        let today = Local::now().date_naive();
        let days: Vec<NaiveDate> = (-DAYS_BACK..=DAYS_FORWARD)
            .map(|off| today + Duration::days(off))
            .collect();

        let (tx, rx) = mpsc::channel();

        App {
            conn,
            account,
            days,
            selected: DAYS_BACK as usize, // start on today
            events: Vec::new(),
            status: String::new(),
            fetching: false,
            tx,
            rx,
        }
    }

    pub fn current_day(&self) -> NaiveDate {
        self.days[self.selected]
    }

    pub fn events_for_day(&self) -> Vec<&CalEvent> {
        let day = self.current_day();
        self.events.iter().filter(|e| e.date == day).collect()
    }

    fn refresh(&mut self) {
        if self.fetching {
            return;
        }
        self.fetching = true;
        self.status = "Fetching…".to_string();

        let conn = self.conn.clone();
        let path = self.account.path.clone();
        let first = self.days[0];
        let last = self.days[self.days.len() - 1];
        let tx = self.tx.clone();

        thread::spawn(move || {
            let result = access_token(&conn, &path)
                .and_then(|t| cal::fetch_events(&t, first, last))
                .map_err(|e| e.to_string());
            let _ = tx.send(Msg::Fetched(result));
        });
    }

    fn apply_fetch(&mut self, result: Result<Vec<CalEvent>, String>) {
        self.fetching = false;
        match result {
            Ok(events) => {
                self.status = format!("{} events", events.len());
                self.events = events;
            }
            Err(e) => self.status = format!("Error: {e}"),
        }
    }

    fn prev_day(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn next_day(&mut self) {
        if self.selected + 1 < self.days.len() {
            self.selected += 1;
        }
    }

    fn today(&mut self) {
        self.selected = DAYS_BACK as usize;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        spawn_input_thread(self.tx.clone());
        self.refresh();

        loop {
            terminal.draw(|f| ui::render(f, self))?;

            let msg = self.rx.recv()?;

            match msg {
                Msg::Fetched(result) => self.apply_fetch(result),

                Msg::Input(Event::Key(key)) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Left | KeyCode::Char('h') => self.prev_day(),
                        KeyCode::Right | KeyCode::Char('l') => self.next_day(),
                        KeyCode::Char('t') => self.today(),
                        KeyCode::Char('r') => self.refresh(),
                        _ => {}
                    }
                }

                Msg::Input(_) => {}
            }
        }

        Ok(())
    }
}
