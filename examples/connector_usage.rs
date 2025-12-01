//! Example: RithmicConnector usage for all plants
use std::env;
use rithmic_rs::{
    RithmicConnector,
    connection_info::{AccountInfo, RithmicConnectionSystem},
};
use tracing::{Level, event};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Before running this example, copy .env.blank to .env
    // and fill in RITHMIC_ACCOUNT_ID, RITHMIC_PASSWORD, FCM_ID, and IB_ID
    dotenv::dotenv().ok();

    tracing_subscriber::fmt().init();

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
    event!(Level::INFO, "Connected and Authenticated with Rithmic");

    // Connect to all plants
    let ticker_handle = connector.connect_ticker().await?;
    event!(Level::INFO, "Connected to Ticker Plant");
    let history_handle = connector.connect_history().await?;
    event!(Level::INFO, "Connected to History Plant");
    let order_handle = connector.connect_order().await?;
    event!(Level::INFO, "Connected to Order Plant");
    let pnl_handle = connector.connect_pnl().await?;
    event!(Level::INFO, "Connected to PnL Plant");

    // Login to all plants
    ticker_handle.login().await?;
    event!(Level::INFO, "Logged into Ticker Plant");
    history_handle.login().await?;
    event!(Level::INFO, "Logged into History Plant");
    order_handle.login().await?;
    event!(Level::INFO, "Logged into Order Plant");
    pnl_handle.login().await?;
    event!(Level::INFO, "Logged into PnL Plant");

    // Here you would typically use the handles to send requests and subscribe to data
    // For this example, we'll just disconnect after successful login
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Disconnect from the Rithmic system
    connector.disconnect().await?;
    event!(Level::INFO, "Disconnected from Rithmic");

    Ok(())
}