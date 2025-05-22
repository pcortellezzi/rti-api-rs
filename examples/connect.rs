//! Example: Connect to the RithmicTickerPlant and stream ticks
use tracing::{Level, event};

use rithmic_rs::{
    RithmicTickerPlant,
    connection_info::{AccountInfo, RithmicConnectionSystem},
    ws::RithmicStream,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load credentials from environment variables (or .env)
    dotenv::dotenv().ok();

    tracing_subscriber::fmt().init();

    let account_info = AccountInfo {
        account_id: "XXXXXXXX".to_string(),
        env: RithmicConnectionSystem::Demo,
        fcm_id: "XXXXXX".to_string(),
        ib_id: "XXXXXX".to_string(),
    };

    let ticker_plant = RithmicTickerPlant::new(&account_info).await;
    let ticker_plant_handle = ticker_plant.get_handle();

    let resp = ticker_plant_handle.login().await;

    event!(Level::INFO, "Login response: {:#?}", resp);

    ticker_plant_handle.disconnect().await?;

    event!(Level::INFO, "Disconnected from Rithmic");

    Ok(())
}
