use dotenv::dotenv;
use std::env;

#[derive(Clone, Debug)]
pub struct RithmicCredentials {
    pub user: String,
    pub password: String,
    pub system_name: String,
    pub gateway_name: String,
    pub direct_gateway_url: Option<String>, // Bypass discovery if set
}

impl RithmicCredentials {
    /// Create new credentials manually.
    pub fn new(user: impl Into<String>, password: impl Into<String>, system_name: impl Into<String>, gateway_name: impl Into<String>) -> Self {
        Self {
            user: user.into(),
            password: password.into(),
            system_name: system_name.into(),
            gateway_name: gateway_name.into(),
            direct_gateway_url: None,
        }
    }

    /// Set a direct gateway URL to bypass the discovery process.
    pub fn with_direct_url(mut self, url: impl Into<String>) -> Self {
        self.direct_gateway_url = Some(url.into());
        self
    }
}

/// Helper to load credentials from standard environment variables via `.env`.
///
/// Takes an optional `env_suffix`.
/// - If `None`, looks for `RITHMIC_USER`, `RITHMIC_PASSWORD`, etc.
/// - If `Some("TEST")`, looks for `RITHMIC_USER_TEST`, `RITHMIC_PASSWORD_TEST`, etc.
pub fn get_credentials_from_env(env_suffix: Option<&str>) -> Result<RithmicCredentials, env::VarError> {
    dotenv().ok();

    let get_key = |base: &str| -> String {
        match env_suffix {
            Some(suffix) if !suffix.is_empty() => format!("{}_{}", base, suffix.to_uppercase()),
            _ => base.to_string(),
        }
    };

    let user = env::var(get_key("RITHMIC_USER"))?;
    let password = env::var(get_key("RITHMIC_PASSWORD"))?;
    let system_name = env::var(get_key("RITHMIC_SYSTEM_NAME"))?;
    let gateway_name = env::var(get_key("RITHMIC_GATEWAY_NAME"))?;
    let direct_gateway_url = env::var(get_key("RITHMIC_DIRECT_URL")).ok();

    Ok(RithmicCredentials {
        user,
        password,
        system_name,
        gateway_name,
        direct_gateway_url,
    })
}

#[derive(Clone, Debug)]
pub struct AccountInfo {
    pub account_id: String,
    pub fcm_id: String,
    pub ib_id: String,
    pub user_type: i32,
}

impl Default for AccountInfo {
    fn default() -> Self {
        AccountInfo {
            account_id: "".to_string(),
            fcm_id: "".to_string(),
            ib_id: "".to_string(),
            user_type: 3,
        }
    }
}

pub const BOOTSTRAP_URL: &str = "wss://rprotocol.rithmic.com:443";