# gcal-tui

Minimal agenda TUI over Google Calendar, authenticated through GNOME Online Accounts.

## Deps

https://github.com/GNOME/gnome-online-accounts

## Build

```
cargo build
cargo install
gcal-tui
```

## Usage

The calendar will default to the first GOA account, if you want to specify a specific one

```
busctl --user tree org.gnome.OnlineAccounts     # find your account object path
GOA_ACCOUNT=<account-path> gcal-tui
# or
gcal-tui <account-path>
```
