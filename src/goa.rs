//! GNOME Online Accounts over D-Bus.

use std::collections::HashMap;
use std::error::Error;

use zbus::blocking::{fdo::ObjectManagerProxy, Connection, Proxy};
use zbus::zvariant::OwnedValue;

const GOA_SERVICE: &str = "org.gnome.OnlineAccounts";
const GOA_ROOT: &str = "/org/gnome/OnlineAccounts";
const IFACE_ACCOUNT: &str = "org.gnome.OnlineAccounts.Account";
const IFACE_CALENDAR: &str = "org.gnome.OnlineAccounts.Calendar";
const IFACE_OAUTH2: &str = "org.gnome.OnlineAccounts.OAuth2Based";

#[derive(Debug, Clone)]
pub struct Account {
    pub path: String,
    pub identity: String,
    pub provider: String,
}

/// Enumerate GOA accounts that expose the Calendar interface.
pub fn calendar_accounts(conn: &Connection) -> Result<Vec<Account>, Box<dyn Error>> {
    let om = ObjectManagerProxy::builder(conn)
        .destination(GOA_SERVICE)?
        .path(GOA_ROOT)?
        .build()?;

    let managed = om.get_managed_objects()?;
    let mut accounts = Vec::new();

    for (path, ifaces) in managed {
        if !ifaces.contains_key(IFACE_CALENDAR) {
            continue;
        }
        let props: &HashMap<String, OwnedValue> = match ifaces.get(IFACE_ACCOUNT) {
            Some(p) => p,
            None => continue,
        };

        let get = |key: &str| -> String {
            props
                .get(key)
                .and_then(|v| <&str>::try_from(&**v).ok())
                .map(String::from)
                .unwrap_or_default()
        };

        accounts.push(Account {
            path: path.to_string(),
            identity: get("PresentationIdentity"),
            provider: get("ProviderType"),
        });
    }

    accounts.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(accounts)
}

pub fn access_token(conn: &Connection, account_path: &str) -> Result<String, Box<dyn Error>> {
    let proxy = Proxy::new(conn, GOA_SERVICE, account_path, IFACE_OAUTH2)?;
    let (token, _expires_in): (String, i32) = proxy.call("GetAccessToken", &())?;
    Ok(token)
}
