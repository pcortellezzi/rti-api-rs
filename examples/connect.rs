//! Example: Connect to the RithmicTickerPlant
use std::env;
use tracing::{Level, event};

use rithmic_rs::{
    RithmicConnector,
    connection_info::{AccountInfo, RithmicConnectionSystem},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Before running this example, copy .env.blank to .env
    // and fill in RITHMIC_ACCOUNT_ID, RITHMIC_PASSWORD, FCM_ID, and IB_ID
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let account_id = env::var("RITHMIC_ACCOUNT_ID")
        .expect("RITHMIC_ACCOUNT_ID must be set in environment variables");
    let password = env::var("RITHMIC_PASSWORD")
        .expect("RITHMIC_PASSWORD must be set in environment variables");
    let fcm_id = env::var("FCM_ID").expect("FCM_ID must be set in environment variables");
    let ib_id = env::var("IB_ID").expect("IB_ID must be set in environment variables");

    let account_info = AccountInfo {
        account_id,
        password,
        env: RithmicConnectionSystem::Demo,
        fcm_id,
        ib_id,
    };

    let connector = RithmicConnector::new(account_info);

    // Connect to the Rithmic system
    connector.connect().await?;
    connector.authenticate().await?;

    // Connect to the ticker plant
    let ticker_handle = connector.connect_ticker().await?;

    // Login to the ticker plant
    let login_resp = ticker_handle.login().await?;
    event!(Level::INFO, "Logged into ticker plant: {:?}", login_resp);

    // Disconnect from the Rithmic system
    connector.disconnect().await?;
    event!(Level::INFO, "Disconnected from Rithmic");

    Ok(())
}