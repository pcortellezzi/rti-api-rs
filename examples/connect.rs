//! Example: Connect to the RithmicTickerPlant
use std::env;
use tracing::{Level, event};

use rithmic_rs::{
    RithmicTickerPlant,
    connection_info::{AccountInfo, RithmicConnectionSystem},
    ws::RithmicStream,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Before running this example, copy .env.blank to .env
    // and fill in RITHMIC_ACCOUNT_ID, FCM_ID, and IB_ID
    dotenv::dotenv().ok();

    tracing_subscriber::fmt().init();

    let account_id = env::var("RITHMIC_ACCOUNT_ID")
        .expect("RITHMIC_ACCOUNT_ID must be set in environment variables");

    let fcm_id = env::var("FCM_ID").expect("RITHMIC_FCM_ID must be set in environment variables");
    let ib_id = env::var("IB_ID").expect("RITHMIC_IB_ID must be set in environment variables");

    let account_info = AccountInfo {
        account_id,
        env: RithmicConnectionSystem::Demo,
        fcm_id,
        ib_id,
    };

    let ticker_plant = RithmicTickerPlant::new(&account_info).await;
    let ticker_plant_handle = ticker_plant.get_handle();

    let resp = ticker_plant_handle.login().await;

    event!(Level::INFO, "Login response: {:#?}", resp);

    ticker_plant_handle.disconnect().await?;

    event!(Level::INFO, "Disconnected from Rithmic");

    Ok(())
}
