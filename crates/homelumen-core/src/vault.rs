//! Where HomeLumen keeps the accounts a user typed, between runs.
//!
//! Only a driver reaching a manufacturer's own service needs one: a bulb
//! answering on the local network has no account behind it. Entries are
//! filed under a driver's slug, so nothing here has to know which
//! manufacturers exist.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const FOLDER: &str = "HomeLumen";
const FILE: &str = "accounts.json";

/// What a driver shows a manufacturer's service to speak for the user.
///
/// Two opaque strings, which is what every account-based service HomeLumen
/// has met asks for: one naming the application, one proving it. Neither is
/// ever read by anything but the driver that filed it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// Public half, safe to show back to the user.
    pub id: String,
    /// Private half.
    pub secret: String,
}

impl Account {
    /// Whether both halves are actually there. A half-typed account is no
    /// account at all: it would only earn a rejection from the service.
    pub fn is_complete(&self) -> bool {
        !self.id.trim().is_empty() && !self.secret.trim().is_empty()
    }
}

/// The account filed under `driver`, if a complete one was ever given.
pub fn account(driver: &str) -> Option<Account> {
    stored().remove(driver).filter(Account::is_complete)
}

/// Files `account` under `driver`, replacing whatever was there.
pub fn set_account(driver: &str, account: Account) -> io::Result<()> {
    let mut accounts = stored();
    accounts.insert(driver.to_owned(), account);
    write(&accounts)
}

/// Everything on file. A missing or unreadable file simply means nothing has
/// been given yet: there is no failure to report to someone who has not got
/// round to filling this in.
fn stored() -> BTreeMap<String, Account> {
    let Some(path) = path() else {
        return BTreeMap::new();
    };

    let Ok(body) = fs::read(path) else {
        return BTreeMap::new();
    };

    serde_json::from_slice(&body).unwrap_or_default()
}

fn write(accounts: &BTreeMap<String, Account>) -> io::Result<()> {
    let path = path().ok_or_else(|| {
        io::Error::other("aucun dossier de configuration sur cette machine")
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = serde_json::to_vec_pretty(accounts).map_err(io::Error::other)?;
    fs::write(&path, body)?;
    restrict(&path)
}

fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(FOLDER).join(FILE))
}

/// Keeps the file to whoever owns it: a secret every account on the machine
/// can read is not much of a secret. Windows already hands each user a
/// private profile directory, so only Unix has anything to say here.
#[cfg(unix)]
fn restrict(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Account;

    #[test]
    fn a_half_typed_account_is_no_account() {
        let account =
            Account { id: "  ".to_owned(), secret: "secret".to_owned() };

        assert!(!account.is_complete());
    }

    #[test]
    fn both_halves_make_an_account() {
        let account =
            Account { id: "id".to_owned(), secret: "secret".to_owned() };

        assert!(account.is_complete());
    }
}
