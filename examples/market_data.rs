//! Example: Subscribe to real-time market data (LastTrade and BBO)
use std::env;
use tracing::{Level, event, info};

use rithmic_rs::{
    RithmicConnector,
    connection_info::{AccountInfo, RithmicConnectionSystem},
    api::request_market_data_update::UpdateBits,
    rti::messages::RithmicMessage,
};

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

    // Connect to the ticker plant
    let ticker_handle = connector.connect_ticker().await?;
    event!(Level::INFO, "Connected to Ticker Plant");

    // Login to the ticker plant
    ticker_handle.login().await?;
    event!(Level::INFO, "Logged into Ticker Plant");

    // Symbol and exchange can be customized here
    let symbol = env::var("SYMBOL").unwrap_or_else(|_| "ZNU5".to_string());
    let exchange = env::var("EXCHANGE").unwrap_or_else(|_| "CBOT".to_string());

    info!("Subscribing to market data for {} on {}", symbol, exchange);

    let _ = ticker_handle.subscribe(&symbol, &exchange, vec![UpdateBits::LastTrade, UpdateBits::Bbo]).await?;

    let mut subscription_receiver = ticker_handle.subscribe_updates();

    // Process a handful of updates, then exit
    let mut processed = 0usize;
    let max_updates = 100usize;

    while processed < max_updates {
        match subscription_receiver.recv().await {
            Ok(update) => match update.message {
                RithmicMessage::LastTrade(t) => {
                    let price = t.trade_price.unwrap_or(0.0);
                    let size = t.trade_size.unwrap_or(0);

                    info!("LastTrade {}: {} @ {}", symbol, size, price);

                    processed += 1;
                }
                RithmicMessage::BestBidOffer(b) => {
                    let bid = b.bid_price.unwrap_or(0.0);
                    let bid_sz = b.bid_size.unwrap_or(0);
                    let ask = b.ask_price.unwrap_or(0.0);
                    let ask_sz = b.ask_size.unwrap_or(0);

                    info!(
                        "BBO {}: bid {} x {}, ask {} x {}",
                        symbol, bid, bid_sz, ask, ask_sz
                    );

                    processed += 1;
                }
                _ => {}
            },
            Err(e) => {
                event!(Level::ERROR, "Error receiving update: {}", e);
                break;
            }
        }
    }

    connector.disconnect().await?;

    info!("Disconnected from Rithmic");

    Ok(())
}