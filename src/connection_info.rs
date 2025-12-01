use dotenv::dotenv;
use std::{env, fmt, str::FromStr};

#[derive(Clone, Debug)]
pub struct RithmicCredentials {
    pub user: String,
    pub password: String,
    pub system_name: String,
    pub gateway_name: String,
    pub direct_gateway_url: Option<String>, // Bypass discovery if set
}

#[derive(Clone, Debug)]
pub enum RithmicConnectionSystem {
    Demo,
    Live,
    Test,
}

impl fmt::Display for RithmicConnectionSystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RithmicConnectionSystem::Demo => write!(f, "demo"),
            RithmicConnectionSystem::Live => write!(f, "live"),
            RithmicConnectionSystem::Test => write!(f, "test"),
        }
    }
}

impl FromStr for RithmicConnectionSystem {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "demo" | "development" => Ok(RithmicConnectionSystem::Demo),
            "live" | "production" => Ok(RithmicConnectionSystem::Live),
            "test" => Ok(RithmicConnectionSystem::Test),
            _ => Err(()),
        }
    }
}

pub fn get_credentials_from_env(env_type: &RithmicConnectionSystem) -> RithmicCredentials {
    dotenv().ok();

    match env_type {
        RithmicConnectionSystem::Demo => RithmicCredentials {
            user: env::var("RITHMIC_DEMO_USER").expect("RITHMIC_DEMO_USER not set"),
            password: env::var("RITHMIC_DEMO_PW").expect("RITHMIC_DEMO_PW not set"),
            system_name: "Rithmic Paper Trading".into(),
            gateway_name: "Chicago Area".into(),
            direct_gateway_url: None,
        },
        RithmicConnectionSystem::Live => RithmicCredentials {
            user: env::var("RITHMIC_LIVE_USER").expect("RITHMIC_LIVE_USER not set"),
            password: env::var("RITHMIC_LIVE_PW").expect("RITHMIC_LIVE_PW not set"),
            system_name: "Rithmic 01".into(),
            gateway_name: "Chicago Area".into(),
            direct_gateway_url: None,
        },
        RithmicConnectionSystem::Test => RithmicCredentials {
            user: env::var("RITHMIC_TEST_USER").expect("RITHMIC_TEST_USER not set"),
            password: env::var("RITHMIC_TEST_PW").expect("RITHMIC_TEST_PW not set"),
            system_name: "Rithmic Test".into(),
            gateway_name: "Test Gateway".into(),
            // Hardcoded Test URL as per user request
            direct_gateway_url: Some("wss://rituz00100.rithmic.com:443".into()),
        },
    }
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