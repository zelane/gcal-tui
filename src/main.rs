//! Minimal agenda TUI over Google Calendar, authenticated via GNOME Online Accounts.

use std::error::Error;

use zbus::blocking::Connection;

mod app;
mod cal;
mod goa;
mod ui;

use app::App;
use goa::{calendar_accounts, Account};

fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::session()
        .map_err(|e| format!("no session bus: {e} (is DBUS_SESSION_BUS_ADDRESS set?)"))?;

    let accounts =
        calendar_accounts(&conn).map_err(|e| format!("could not talk to goa-daemon: {e}"))?;

    if accounts.is_empty() {
        eprintln!("No GNOME Online Accounts expose a calendar.");
        eprintln!("Add one with `gnome-online-accounts-gtk`, or check that");
        eprintln!("Calendar is enabled for the account.");
        std::process::exit(1);
    }

    let account = pick_account(&accounts)?;

    let mut app = App::new(conn, account);
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}

fn pick_account(accounts: &[Account]) -> Result<Account, Box<dyn Error>> {
    let wanted = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("GOA_ACCOUNT").ok());

    match wanted {
        Some(p) => accounts
            .iter()
            .find(|a| a.path == p)
            .cloned()
            .ok_or_else(|| format!("no calendar account at {p}").into()),
        None => Ok(accounts
            .iter()
            .find(|a| a.provider == "google")
            .or_else(|| accounts.first())
            .cloned()
            .unwrap()),
    }
}
