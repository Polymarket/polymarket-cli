use anyhow::{Result, bail};

const PASSWORD_ENV_VAR: &str = "POLYMARKET_PASSWORD";

/// Prompt for password, or read from POLYMARKET_PASSWORD env var.
pub fn prompt_password(prompt_msg: &str) -> Result<String> {
    if let Ok(pw) = std::env::var(PASSWORD_ENV_VAR)
        && !pw.is_empty()
    {
        return Ok(pw);
    }
    rpassword::prompt_password(prompt_msg).map_err(Into::into)
}

/// Prompt for password with confirmation (for create/import).
pub fn prompt_new_password() -> Result<String> {
    let pw = prompt_password("Enter password to encrypt wallet: ")?;
    if pw.is_empty() {
        bail!("Password cannot be empty");
    }
    let confirm = prompt_password("Confirm password: ")?;
    if pw != confirm {
        bail!("Passwords do not match");
    }
    Ok(pw)
}

/// Prompt for password with up to 3 retries, calling `try_fn` each time.
/// Returns the result of the first successful call to `try_fn`.
pub fn prompt_password_with_retries<T, F>(try_fn: F) -> Result<T>
where
    F: Fn(&str) -> Result<T>,
{
    for attempt in 1..=3 {
        let pw = prompt_password("Enter wallet password: ")?;
        match try_fn(&pw) {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempt < 3 {
                    eprintln!("Wrong password. Try again. ({attempt}/3)");
                } else {
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}
