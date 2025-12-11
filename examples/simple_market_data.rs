use dotenv::dotenv;
use eyre::{Report, Result};
use rti_api_rs::{RithmicClient, RithmicMessage, connection_info::get_credentials_from_env};
use tokio::signal;
use tracing::{debug, info, warn};

#[tokio::main]
async fn main() -> Result<(), Report> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    // 1. Get credentials from env (User, Pass, System, Gateway)
    let credentials = get_credentials_from_env(None)?;

    info!("Connecting to Rithmic via {}...", credentials.gateway_name);
    let mut client = RithmicClient::new(credentials);

    // 2. List Systems (Optional, for UI demo)
    info!("Fetching available Rithmic Systems...");
    match client.list_systems().await {
        Ok(systems) => {
            info!("--- Available Systems ---");
            for sys in systems {
                info!(" - {}", sys);
            }
            info!("-------------------------");
        }
        Err(e) => warn!("Failed to list systems (non-fatal): {:?}", e),
    }

    // 2. Connect (Discovery -> Login)
    let mut event_rx = client.connect().await?;

    info!("Connected! Subscribing to ESZ5 (CME)...");

    // 3. Subscribe
    client.subscribe_market_data("ESZ5", "CME", None).await?;

    info!("Subscribed. Waiting for data (Press Ctrl+C to stop)...");

    loop {
        tokio::select! {
            msg = event_rx.recv() => {
                match msg {
                    Some(response) => {
                        match response.message {
                            RithmicMessage::LastTrade(trade) => {
                                info!("TRADE: {} @ {} (Vol: {})", trade.symbol(), trade.trade_price(), trade.trade_size());
                            }
                            RithmicMessage::BestBidOffer(bbo) => {
                                info!("BBO: {} Bid: {} Ask: {}", bbo.symbol(), bbo.bid_price(), bbo.ask_price());
                            }
                            _ => {
                                debug!("Other message: {:?}", response.message);
                            }
                        }
                    }
                    None => {
                        info!("Event stream closed.");
                        break;
                    }
                }
            }
            _ = signal::ctrl_c() => {
                info!("Stopping...");
                break;
            }
        }
    }

    Ok(())
}
