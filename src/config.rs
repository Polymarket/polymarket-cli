use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const ENV_VAR: &str = "POLYMARKET_PRIVATE_KEY";
const SIG_TYPE_ENV_VAR: &str = "POLYMARKET_SIGNATURE_TYPE";
pub const DEFAULT_SIGNATURE_TYPE: &str = "gnosis-safe";

pub const NO_WALLET_MSG: &str =
    "No wallet configured. Run `polymarket wallet create` or `polymarket wallet import <key>`";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub private_key: String,
    pub chain_id: u64,
    #[serde(default = "default_signature_type")]
    pub signature_type: String,
}

fn default_signature_type() -> String {
    DEFAULT_SIGNATURE_TYPE.to_string()
}

pub enum KeySource {
    Flag,
    EnvVar,
    ConfigFile,
    None,
}

impl KeySource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Flag => "--private-key flag",
            Self::EnvVar => "POLYMARKET_PRIVATE_KEY env var",
            Self::ConfigFile => "config file",
            Self::None => "not configured",
        }
    }
}

fn config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join("polymarket"))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn config_exists() -> bool {
    config_path().is_ok_and(|p| p.exists())
}

pub fn delete_config() -> Result<()> {
    let dir = config_dir()?;
    if dir.exists() {
        fs::remove_dir_all(&dir).context("Failed to remove config directory")?;
    }
    Ok(())
}

pub fn load_config() -> Option<Config> {
    let path = config_path().ok()?;
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Priority: CLI flag > env var > config file > default ("proxy").
pub fn resolve_signature_type(cli_flag: Option<&str>) -> String {
    if let Some(st) = cli_flag {
        return st.to_string();
    }
    if let Ok(st) = std::env::var(SIG_TYPE_ENV_VAR)
        && !st.is_empty()
    {
        return st;
    }
    if let Some(config) = load_config() {
        return config.signature_type;
    }
    DEFAULT_SIGNATURE_TYPE.to_string()
}

pub fn save_wallet(key: &str, chain_id: u64, signature_type: &str) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).context("Failed to create config directory")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    }

    let config = Config {
        private_key: key.to_string(),
        chain_id,
        signature_type: signature_type.to_string(),
    };
    let json = serde_json::to_string_pretty(&config)?;
    let path = config_path()?;

    #[cfg(unix)]
    {
        use std::io::Write as _;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)
            .context("Failed to create config file")?;
        file.write_all(json.as_bytes())
            .context("Failed to write config file")?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&path, &json).context("Failed to write config file")?;
    }

    Ok(())
}

/// Priority: CLI flag > env var > config file.
pub fn resolve_key(cli_flag: Option<&str>) -> (Option<String>, KeySource) {
    if let Some(key) = cli_flag {
        return (Some(key.to_string()), KeySource::Flag);
    }
    if let Ok(key) = std::env::var(ENV_VAR)
        && !key.is_empty()
    {
        return (Some(key), KeySource::EnvVar);
    }
    if let Some(config) = load_config() {
        return (Some(config.private_key), KeySource::ConfigFile);
    }
    (None, KeySource::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to serialize env var tests (set_var is not thread-safe)
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    unsafe fn set(var: &str, val: &str) {
        unsafe { std::env::set_var(var, val) };
    }

    unsafe fn unset(var: &str) {
        unsafe { std::env::remove_var(var) };
    }

    #[test]
    fn resolve_key_flag_overrides_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(ENV_VAR, "env_key") };
        let (key, source) = resolve_key(Some("flag_key"));
        assert_eq!(key.unwrap(), "flag_key");
        assert!(matches!(source, KeySource::Flag));
        unsafe { unset(ENV_VAR) };
    }

    #[test]
    fn resolve_key_env_var_returns_env_value() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(ENV_VAR, "env_key_value") };
        let (key, source) = resolve_key(None);
        assert_eq!(key.unwrap(), "env_key_value");
        assert!(matches!(source, KeySource::EnvVar));
        unsafe { unset(ENV_VAR) };
    }

    #[test]
    fn resolve_key_skips_empty_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(ENV_VAR, "") };
        let (_, source) = resolve_key(None);
        assert!(!matches!(source, KeySource::EnvVar));
        unsafe { unset(ENV_VAR) };
    }

    #[test]
    fn resolve_sig_type_flag_overrides_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(SIG_TYPE_ENV_VAR, "eoa") };
        assert_eq!(resolve_signature_type(Some("gnosis-safe")), "gnosis-safe");
        unsafe { unset(SIG_TYPE_ENV_VAR) };
    }

    #[test]
    fn resolve_sig_type_env_var_returns_env_value() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(SIG_TYPE_ENV_VAR, "eoa") };
        assert_eq!(resolve_signature_type(None), "eoa");
        unsafe { unset(SIG_TYPE_ENV_VAR) };
    }

    #[test]
    fn resolve_sig_type_without_env_returns_nonempty() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { unset(SIG_TYPE_ENV_VAR) };
        let result = resolve_signature_type(None);
        assert!(!result.is_empty());
    }
}
