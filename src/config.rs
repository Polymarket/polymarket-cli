use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const ENV_VAR: &str = "POLYMARKET_PRIVATE_KEY";
const SIG_TYPE_ENV_VAR: &str = "POLYMARKET_SIGNATURE_TYPE";
const PASSWORD_ENV_VAR: &str = "POLYMARKET_PASSWORD";
pub(crate) const DEFAULT_SIGNATURE_TYPE: &str = "proxy";
/// The one file in `keystore_dir()` that holds the wallet.
pub(crate) const KEYSTORE_FILE: &str = "wallet.json";

/// The decrypted key, kept for the rest of the command so that a command that
/// resolves the key several times asks for the password once.
static UNLOCKED_KEY: Mutex<Option<String>> = Mutex::new(None);

pub(crate) const NO_WALLET_MSG: &str =
    "No wallet configured. Run `polymarket wallet create` or `polymarket wallet import <key>`";

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub private_key: String,
    pub chain_id: u64,
    #[serde(default = "default_signature_type")]
    pub signature_type: String,
    #[serde(default)]
    pub encrypted: bool,
}

fn default_signature_type() -> String {
    DEFAULT_SIGNATURE_TYPE.to_string()
}

pub(crate) enum KeySource {
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

pub fn keystore_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join("keystore"))
}

/// Prompt for a password, or read from `POLYMARKET_PASSWORD` env var.
pub fn read_password(prompt: &str) -> Result<String> {
    if let Ok(pw) = std::env::var(PASSWORD_ENV_VAR)
        && !pw.is_empty()
    {
        return Ok(pw);
    }
    rpassword::prompt_password(prompt).context("Failed to read password")
}

/// Prompt for a new password with confirmation.
pub fn read_new_password() -> Result<String> {
    let pw = read_password("Enter password to encrypt your key: ")?;
    anyhow::ensure!(!pw.is_empty(), "Password cannot be empty");
    let confirm = read_password("Confirm password: ")?;
    anyhow::ensure!(pw == confirm, "Passwords do not match");
    Ok(pw)
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

/// Load config from disk. Returns `Ok(None)` if no config file exists,
/// or `Err` if the file exists but can't be read or parsed.
pub fn load_config() -> Result<Option<Config>> {
    let path = config_path()?;
    let data = match fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(anyhow::anyhow!(e).context(format!("Failed to read {}", path.display())));
        }
    };
    let config = serde_json::from_str(&data)
        .context(format!("Invalid JSON in config file {}", path.display()))?;
    Ok(Some(config))
}

/// Priority: CLI flag > env var > config file > default ("proxy").
pub fn resolve_signature_type(cli_flag: Option<&str>) -> Result<String> {
    if let Some(st) = cli_flag {
        return Ok(st.to_string());
    }
    if let Ok(st) = std::env::var(SIG_TYPE_ENV_VAR)
        && !st.is_empty()
    {
        return Ok(st);
    }
    if let Some(config) = load_config()? {
        return Ok(config.signature_type);
    }
    Ok(DEFAULT_SIGNATURE_TYPE.to_string())
}

fn write_config_file(config: &Config) -> Result<()> {
    forget_unlocked_key();

    let dir = config_dir()?;
    fs::create_dir_all(&dir).context("Failed to create config directory")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    }

    let json = serde_json::to_string_pretty(config)?;
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

/// Save wallet as plaintext (legacy, used only when `--no-password` is specified).
pub fn save_wallet(key: &str, chain_id: u64, signature_type: &str) -> Result<()> {
    write_config_file(&Config {
        private_key: key.to_string(),
        chain_id,
        signature_type: signature_type.to_string(),
        encrypted: false,
    })
}

/// Save wallet as encrypted keystore. The keystore file must already be
/// written to `keystore_dir()` by the caller.
pub fn save_wallet_encrypted(chain_id: u64, signature_type: &str) -> Result<()> {
    write_config_file(&Config {
        private_key: String::new(),
        chain_id,
        signature_type: signature_type.to_string(),
        encrypted: true,
    })
}

/// Priority: CLI flag > env var > config file.
/// If the config file uses encrypted keystore, prompts for password.
pub fn resolve_key(cli_flag: Option<&str>) -> Result<(Option<String>, KeySource)> {
    if let Some(key) = cli_flag {
        return Ok((Some(key.to_string()), KeySource::Flag));
    }
    if let Ok(key) = std::env::var(ENV_VAR)
        && !key.is_empty()
    {
        return Ok((Some(key), KeySource::EnvVar));
    }
    if let Some(config) = load_config()? {
        if config.encrypted {
            let key = unlocked_key()?;
            return Ok((Some(key), KeySource::ConfigFile));
        }
        return Ok((Some(config.private_key), KeySource::ConfigFile));
    }
    Ok((None, KeySource::None))
}

/// Drop the decrypted key, so the next command has to unlock the wallet again.
pub fn forget_unlocked_key() {
    *UNLOCKED_KEY.lock().unwrap_or_else(PoisonError::into_inner) = None;
}

fn unlocked_key() -> Result<String> {
    let mut unlocked = UNLOCKED_KEY.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(key) = unlocked.as_ref() {
        return Ok(key.clone());
    }
    let key = decrypt_keystore()?;
    *unlocked = Some(key.clone());
    Ok(key)
}

/// Decrypt the keystore file and return the private key hex.
fn decrypt_keystore() -> Result<String> {
    use polymarket_client_sdk_v2::auth::LocalSigner;

    let path = keystore_dir()?.join(KEYSTORE_FILE);
    anyhow::ensure!(
        path.is_file(),
        "No keystore file found. Config says encrypted but keystore is missing."
    );

    let password = read_password("Enter password to unlock wallet: ")?;
    let signer = LocalSigner::decrypt_keystore(path, password)
        .context("Failed to decrypt keystore. Wrong password?")?;
    Ok(format!("{:#x}", signer.to_bytes()))
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
        let (key, source) = resolve_key(Some("flag_key")).unwrap();
        assert_eq!(key.unwrap(), "flag_key");
        assert!(matches!(source, KeySource::Flag));
        unsafe { unset(ENV_VAR) };
    }

    #[test]
    fn resolve_key_env_var_returns_env_value() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(ENV_VAR, "env_key_value") };
        let (key, source) = resolve_key(None).unwrap();
        assert_eq!(key.unwrap(), "env_key_value");
        assert!(matches!(source, KeySource::EnvVar));
        unsafe { unset(ENV_VAR) };
    }

    #[test]
    fn resolve_key_skips_empty_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(ENV_VAR, "") };
        let (_, source) = resolve_key(None).unwrap();
        assert!(!matches!(source, KeySource::EnvVar));
        unsafe { unset(ENV_VAR) };
    }

    #[test]
    fn resolve_sig_type_flag_overrides_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(SIG_TYPE_ENV_VAR, "eoa") };
        assert_eq!(
            resolve_signature_type(Some("gnosis-safe")).unwrap(),
            "gnosis-safe"
        );
        unsafe { unset(SIG_TYPE_ENV_VAR) };
    }

    #[test]
    fn resolve_sig_type_env_var_returns_env_value() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(SIG_TYPE_ENV_VAR, "eoa") };
        assert_eq!(resolve_signature_type(None).unwrap(), "eoa");
        unsafe { unset(SIG_TYPE_ENV_VAR) };
    }

    #[test]
    fn resolve_sig_type_without_env_returns_nonempty() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { unset(SIG_TYPE_ENV_VAR) };
        let result = resolve_signature_type(None).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn encrypted_wallet_unlocks_once_per_command() {
        use alloy::signers::local::PrivateKeySigner;

        let _lock = ENV_LOCK.lock().unwrap();
        let original_home = std::env::var_os("HOME");
        let home = std::env::temp_dir().join(format!("polymarket-unlock-{}", std::process::id()));
        unsafe {
            set("HOME", home.to_str().unwrap());
            unset(ENV_VAR);
            set(PASSWORD_ENV_VAR, "right");
        }
        forget_unlocked_key();

        let ks_dir = keystore_dir().unwrap();
        fs::create_dir_all(&ks_dir).unwrap();
        let key = [7u8; 32];
        PrivateKeySigner::encrypt_keystore(
            &ks_dir,
            &mut rand::thread_rng(),
            key,
            "right",
            Some(KEYSTORE_FILE),
        )
        .unwrap();
        save_wallet_encrypted(137, DEFAULT_SIGNATURE_TYPE).unwrap();

        let (first, _) = resolve_key(None).unwrap();

        // A second resolve must not reach the keystore, so a wrong password
        // cannot fail it. After the key is forgotten, the keystore is used again.
        unsafe { set(PASSWORD_ENV_VAR, "wrong") };
        let (second, _) = resolve_key(None).unwrap();
        forget_unlocked_key();
        let relocked = resolve_key(None);

        unsafe { unset(PASSWORD_ENV_VAR) };
        match original_home {
            Some(value) => unsafe { std::env::set_var("HOME", value) },
            None => unsafe { unset("HOME") },
        }
        let _ = fs::remove_dir_all(&home);

        assert_eq!(first, second);
        assert!(first.unwrap().ends_with(&"07".repeat(32)));
        assert!(relocked.is_err());
    }
}
