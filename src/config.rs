use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use polymarket_client_sdk_v2::types::Address;
use serde::{Deserialize, Serialize};

const ENV_VAR: &str = "POLYMARKET_PRIVATE_KEY";
const SIG_TYPE_ENV_VAR: &str = "POLYMARKET_SIGNATURE_TYPE";
const FUNDER_ENV_VAR: &str = "POLYMARKET_FUNDER";
pub(crate) const DEFAULT_SIGNATURE_TYPE: &str = "proxy";
pub(crate) const POLY_1271_SIGNATURE_TYPE: &str = "poly-1271";

pub(crate) const NO_WALLET_MSG: &str =
    "No wallet configured. Run `polymarket wallet create` or `polymarket wallet import <key>`";

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub private_key: String,
    pub chain_id: u64,
    #[serde(default = "default_signature_type")]
    pub signature_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funder: Option<String>,
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

pub fn normalize_signature_type(signature_type: &str) -> Result<&'static str> {
    match signature_type.to_ascii_lowercase().as_str() {
        "eoa" | "0" => Ok("eoa"),
        "proxy" | "1" => Ok(DEFAULT_SIGNATURE_TYPE),
        "gnosis-safe" | "gnosis_safe" | "2" => Ok("gnosis-safe"),
        "poly-1271" | "poly1271" | "poly_1271" | "3" => Ok(POLY_1271_SIGNATURE_TYPE),
        _ => bail!(
            "Invalid signature type '{signature_type}'. Expected eoa, proxy, gnosis-safe, or poly-1271"
        ),
    }
}

/// Priority: CLI flag > env var > config file > default ("proxy").
pub fn resolve_signature_type(cli_flag: Option<&str>) -> Result<String> {
    if let Some(st) = cli_flag {
        return Ok(normalize_signature_type(st)?.to_string());
    }
    if let Ok(st) = std::env::var(SIG_TYPE_ENV_VAR)
        && !st.is_empty()
    {
        return Ok(normalize_signature_type(&st)?.to_string());
    }
    if let Some(config) = load_config()? {
        return Ok(normalize_signature_type(&config.signature_type)?.to_string());
    }
    Ok(DEFAULT_SIGNATURE_TYPE.to_string())
}

fn parse_funder(funder: &str) -> Result<Address> {
    Address::from_str(funder).with_context(|| format!("Invalid funder address: {funder}"))
}

/// Resolve a funder supplied for a new wallet, without inheriting an existing config value.
/// Priority: CLI flag > env var.
pub fn resolve_funder_input(cli_flag: Option<&str>) -> Result<Option<Address>> {
    if let Some(funder) = cli_flag {
        return parse_funder(funder).map(Some);
    }
    if let Ok(funder) = std::env::var(FUNDER_ENV_VAR)
        && !funder.is_empty()
    {
        return parse_funder(&funder).map(Some);
    }
    Ok(None)
}

/// Resolve a funder for runtime commands. Priority: CLI flag > env var > config file.
pub fn resolve_funder(cli_flag: Option<&str>) -> Result<Option<Address>> {
    if let Some(funder) = resolve_funder_input(cli_flag)? {
        return Ok(Some(funder));
    }

    load_config()?
        .and_then(|config| config.funder)
        .map(|funder| parse_funder(&funder))
        .transpose()
}

pub fn validate_funder_for_signature_type(
    signature_type: &str,
    funder: Option<Address>,
) -> Result<Option<Address>> {
    let signature_type = normalize_signature_type(signature_type)?;
    match (signature_type, funder) {
        (POLY_1271_SIGNATURE_TYPE, None) => {
            bail!("--funder is required when using signature type poly-1271")
        }
        (POLY_1271_SIGNATURE_TYPE, Some(Address::ZERO)) => {
            bail!("The funder address cannot be zero for signature type poly-1271")
        }
        (POLY_1271_SIGNATURE_TYPE, Some(funder)) => Ok(Some(funder)),
        (_, Some(_)) => bail!(
            "A funder address is only valid with signature type poly-1271; unset --funder, POLYMARKET_FUNDER, or the config funder"
        ),
        (_, None) => Ok(None),
    }
}

pub fn save_wallet(
    key: &str,
    chain_id: u64,
    signature_type: &str,
    funder: Option<&str>,
) -> Result<()> {
    let signature_type = normalize_signature_type(signature_type)?;
    let funder = funder.map(parse_funder).transpose()?;
    let funder = validate_funder_for_signature_type(signature_type, funder)?
        .map(|address| address.to_string());

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
        funder,
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
        return Ok((Some(config.private_key), KeySource::ConfigFile));
    }
    Ok((None, KeySource::None))
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
    fn normalize_signature_type_accepts_poly_1271_aliases() {
        for alias in ["poly-1271", "poly1271", "POLY_1271", "3"] {
            assert_eq!(
                normalize_signature_type(alias).unwrap(),
                POLY_1271_SIGNATURE_TYPE
            );
        }
    }

    #[test]
    fn normalize_signature_type_rejects_unknown_values() {
        assert!(normalize_signature_type("unknown").is_err());
    }

    #[test]
    fn poly_1271_wallet_requires_funder_before_writing_config() {
        let error = save_wallet("unused", 137, POLY_1271_SIGNATURE_TYPE, None).unwrap_err();
        assert!(error.to_string().contains("--funder is required"));
    }

    #[test]
    fn non_poly_wallet_rejects_funder_before_writing_config() {
        let error = save_wallet(
            "unused",
            137,
            DEFAULT_SIGNATURE_TYPE,
            Some("0xd1615A7B6146cDbA40a559eC876A3bcca4050890"),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("only valid with signature type poly-1271")
        );
    }

    #[test]
    fn poly_1271_rejects_zero_funder() {
        let error =
            validate_funder_for_signature_type(POLY_1271_SIGNATURE_TYPE, Some(Address::ZERO))
                .unwrap_err();
        assert!(error.to_string().contains("cannot be zero"));
    }

    #[test]
    fn poly_1271_accepts_nonzero_funder() {
        let funder = Address::from_str("0xd1615A7B6146cDbA40a559eC876A3bcca4050890").unwrap();
        assert_eq!(
            validate_funder_for_signature_type(POLY_1271_SIGNATURE_TYPE, Some(funder)).unwrap(),
            Some(funder)
        );
    }

    #[test]
    fn resolve_funder_input_reads_env_address() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(FUNDER_ENV_VAR, "0xd1615A7B6146cDbA40a559eC876A3bcca4050890") };
        let funder = resolve_funder_input(None).unwrap().unwrap();
        assert_eq!(
            funder,
            Address::from_str("0xd1615A7B6146cDbA40a559eC876A3bcca4050890").unwrap()
        );
        unsafe { unset(FUNDER_ENV_VAR) };
    }

    #[test]
    fn resolve_funder_input_flag_overrides_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe { set(FUNDER_ENV_VAR, "0xd1615A7B6146cDbA40a559eC876A3bcca4050890") };
        let funder = resolve_funder_input(Some("0x0000000000000000000000000000000000000001"))
            .unwrap()
            .unwrap();
        assert_eq!(
            funder,
            Address::from_str("0x0000000000000000000000000000000000000001").unwrap()
        );
        unsafe { unset(FUNDER_ENV_VAR) };
    }
}
