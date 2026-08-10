//! Where HomeLumen keeps, between runs, what it has learned about reaching a
//! device.
//!
//! An [`Account`] is what a user typed, filed under a driver's slug. A device
//! secret is never typed: a driver reads it once from wherever the
//! manufacturer keeps it and files it under the device's own identity, so a
//! local route that needs it keeps working even when that source, typically
//! a cloud account, is not reachable again.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::device::DeviceId;

const FOLDER: &str = "HomeLumen";
const ACCOUNTS_FILE: &str = "accounts.json";
const DEVICE_SECRETS_FILE: &str = "device-secrets.json";

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
    let mut accounts: BTreeMap<String, Account> = read_json(ACCOUNTS_FILE);
    accounts.remove(driver).filter(Account::is_complete)
}

/// Files `account` under `driver`, replacing whatever was there.
pub fn set_account(driver: &str, account: Account) -> io::Result<()> {
    let mut accounts: BTreeMap<String, Account> = read_json(ACCOUNTS_FILE);
    accounts.insert(driver.to_owned(), account);
    write_json(ACCOUNTS_FILE, &accounts)
}

/// The secret cached for `id`, if a driver has ever learned one.
pub fn device_secret(id: &DeviceId) -> Option<String> {
    let mut secrets: BTreeMap<String, String> = read_json(DEVICE_SECRETS_FILE);
    secrets.remove(id.as_str()).filter(|secret| !secret.trim().is_empty())
}

/// Caches `secret` for `id`, replacing whatever was cached for it.
pub fn set_device_secret(id: &DeviceId, secret: &str) -> io::Result<()> {
    let mut secrets: BTreeMap<String, String> = read_json(DEVICE_SECRETS_FILE);
    secrets.insert(id.as_str().to_owned(), secret.to_owned());
    write_json(DEVICE_SECRETS_FILE, &secrets)
}

/// Forgets whatever secret was cached for `id`, so the next attempt to reach
/// it starts from scratch instead of trusting a value that just proved wrong.
pub fn forget_device_secret(id: &DeviceId) -> io::Result<()> {
    let mut secrets: BTreeMap<String, String> = read_json(DEVICE_SECRETS_FILE);
    secrets.remove(id.as_str());
    write_json(DEVICE_SECRETS_FILE, &secrets)
}

/// Reads `file` from the configuration folder. A missing or unreadable file
/// simply means nothing has been filed yet: there is no failure to report to
/// someone who has not got round to filling this in.
fn read_json<T: Default + DeserializeOwned>(file: &str) -> T {
    let Some(path) = path(file) else {
        return T::default();
    };

    let Ok(body) = fs::read(path) else {
        return T::default();
    };

    serde_json::from_slice(&body).unwrap_or_default()
}

fn write_json<T: Serialize>(file: &str, value: &T) -> io::Result<()> {
    let path = path(file).ok_or_else(|| {
        io::Error::other("aucun dossier de configuration sur cette machine")
    })?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    fs::write(&path, body)?;
    restrict(&path)
}

fn path(file: &str) -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(FOLDER).join(file))
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
