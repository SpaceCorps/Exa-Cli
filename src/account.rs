//! Account resolution and API key management.
//!
//! An API key can come from an explicit `--account <name>` (stored in the OS keystore),
//! an explicit `--api-key <key>` flag, or the `EXA_API_KEY` environment variable.

use crate::client::Client;
use crate::config::{self, Config};
use crate::error::{Error, ErrorCode, Result};
use crate::secrets;

pub struct Resolved {
    pub name: String,
    pub api_key: String,
}

impl Resolved {
    pub fn client(&self) -> Client {
        Client::new(&self.api_key)
    }
}

pub fn resolve(account: Option<&str>, api_key: Option<&str>) -> Result<Resolved> {
    let config = config::load()?;

    // 1. If an account is explicitly specified, resolve from keystore.
    if let Some(requested) = account.map(str::trim).filter(|s| !s.is_empty()) {
        let Some((name, _account)) = config.find(requested) else {
            return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{requested}'."))
                .detail(describe(&config))
                .fix("exa accounts list"));
        };

        let key = secrets::store()?.get(&secrets::account_key(name))?;
        let Some(api_key) = key.filter(|k| !k.trim().is_empty()) else {
            return Err(Error::new(ErrorCode::AuthRequired, format!("Account '{name}' has no stored API key."))
                .detail("The config entry exists but the keystore has nothing under it.")
                .fix(format!("exa accounts add {name} --api-key <key>")));
        };

        return Ok(Resolved { name: name.clone(), api_key });
    }

    // 2. If an API key is provided directly (via --api-key or EXA_API_KEY env var), use it.
    if let Some(key) = api_key.map(str::trim).filter(|s| !s.is_empty()) {
        return Ok(Resolved { name: "direct".into(), api_key: key.to_string() });
    }

    // 3. Otherwise, neither account nor API key was provided.
    if config.accounts.is_empty() {
        return Err(Error::new(
            ErrorCode::NoAccount,
            "No account or API key specified. Set EXA_API_KEY, use --api-key <key>, or run: exa login",
        )
        .fix("exa login"));
    }

    Err(Error::new(
        ErrorCode::NoAccount,
        "No account or API key specified. Pass --account <name>, --api-key <key>, or set EXA_API_KEY.",
    )
    .detail(describe(&config))
    .fix("exa accounts list"))
}

fn describe(config: &Config) -> String {
    if config.accounts.is_empty() {
        return "No accounts are configured yet. Run 'exa accounts add <name> --api-key <key>' or 'exa login'.".into();
    }
    let listed: Vec<String> = config
        .sorted()
        .into_iter()
        .map(|(k, v)| if v.identity.trim().is_empty() { k.clone() } else { format!("{k} ({})", v.identity) })
        .collect();
    format!("Configured accounts: {}", listed.join(", "))
}
