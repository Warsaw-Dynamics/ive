//! Where run configurations' secrets live: the OS wallet (Secret Service on Linux, Keychain on macOS).

/// A store of secrets by id; `vars.toml` holds only the ids.
pub trait Wallet {
    /// The secret stored under `id`; `None` when there is none.
    fn get(&self, id: &str) -> Result<Option<String>, String>;
    fn set(&self, id: &str, secret: &str) -> Result<(), String>;
}

/// The OS wallet, entries under service `ive` and account `<profile>/<id>`.
pub struct OsWallet {
    profile: String,
}

const SERVICE: &str = "ive";

impl OsWallet {
    pub fn new(profile: &str) -> Self {
        Self { profile: profile.to_owned() }
    }

    fn account(&self, id: &str) -> String {
        format!("{}/{id}", self.profile)
    }
}

/// Over oo7, which gpui already builds. The items keep keyring-rs's layout (`service`, `username`, `target`), so
/// secrets IVE stored through keyring-rs are still found.
#[cfg(target_os = "linux")]
impl Wallet for OsWallet {
    fn get(&self, id: &str) -> Result<Option<String>, String> {
        let account = self.account(id);
        futures::executor::block_on(async {
            let keyring = unlocked_keyring().await?;
            let items = keyring.search_items(&[("service", SERVICE), ("username", account.as_str())]).await.map_err(unavailable)?;
            let Some(item) = items.first() else {
                return Ok(None);
            };
            item.unlock().await.map_err(unavailable)?;
            let secret = item.secret().await.map_err(unavailable)?;
            String::from_utf8(secret.to_vec()).map(Some).map_err(|_| format!("the OS wallet's secret {account} is not UTF-8"))
        })
    }

    fn set(&self, id: &str, secret: &str) -> Result<(), String> {
        let account = self.account(id);
        let label = format!("IVE secret {account}");
        let attributes = [("service", SERVICE), ("username", account.as_str()), ("target", "default")];
        futures::executor::block_on(async {
            let keyring = unlocked_keyring().await?;
            keyring.create_item(&label, &attributes, secret, true).await.map_err(unavailable)
        })
    }
}

#[cfg(target_os = "linux")]
async fn unlocked_keyring() -> Result<oo7::Keyring, String> {
    let keyring = oo7::Keyring::new().await.map_err(unavailable)?;
    keyring.unlock().await.map_err(unavailable)?;
    Ok(keyring)
}

#[cfg(target_os = "macos")]
impl Wallet for OsWallet {
    fn get(&self, id: &str) -> Result<Option<String>, String> {
        match self.entry(id)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(unavailable(error)),
        }
    }

    fn set(&self, id: &str, secret: &str) -> Result<(), String> {
        self.entry(id)?.set_password(secret).map_err(unavailable)
    }
}

#[cfg(target_os = "macos")]
impl OsWallet {
    fn entry(&self, id: &str) -> Result<keyring::Entry, String> {
        keyring::Entry::new(SERVICE, &self.account(id)).map_err(unavailable)
    }
}

fn unavailable(error: impl std::fmt::Display) -> String {
    format!("the OS wallet is not available: {error}")
}

/// A wallet in memory, for tests.
#[cfg(test)]
#[derive(Default)]
pub struct MemoryWallet(std::cell::RefCell<std::collections::HashMap<String, String>>);

#[cfg(test)]
impl Wallet for MemoryWallet {
    fn get(&self, id: &str) -> Result<Option<String>, String> {
        Ok(self.0.borrow().get(id).cloned())
    }

    fn set(&self, id: &str, secret: &str) -> Result<(), String> {
        self.0.borrow_mut().insert(id.to_owned(), secret.to_owned());
        Ok(())
    }
}
