use rti_api_rs::{RithmicClient, connection_info::get_credentials_from_env, RithmicMessage};
use dotenv::dotenv;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    // 1. Get credentials from env (User, Pass, System, Gateway)
    let credentials = get_credentials_from_env(None)?;

    println!("Connecting to Rithmic via {}...", credentials.gateway_name);
    let mut client = RithmicClient::new(credentials);

    // 2. List Systems (Optional, for UI demo)
    println!("Fetching available Rithmic Systems...");
    match client.list_systems().await {
        Ok(systems) => {
            println!("--- Available Systems ---");
            for sys in systems {
                println!(" - {}", sys);
            }
            println!("-------------------------");
        }
        Err(e) => println!("Failed to list systems (non-fatal): {}", e),
    }
    
    // 2. Connect (Discovery -> Login)
    let mut event_rx = client.connect().await?;
    
    println!("Connected! Subscribing to ESZ5 (CME)...");
    
    // 3. Subscribe
    client.subscribe_market_data("ESZ5", "CME", None).await?;
    
    println!("Subscribed. Waiting for data (Press Ctrl+C to stop)...");

    loop {
        tokio::select! {
            msg = event_rx.recv() => {
                match msg {
                    Some(response) => {
                        match response.message {
                            RithmicMessage::LastTrade(trade) => {
                                println!("TRADE: {} @ {} (Vol: {})", trade.symbol(), trade.trade_price(), trade.trade_size());
                            }
                            RithmicMessage::BestBidOffer(bbo) => {
                                println!("BBO: {} Bid: {} Ask: {}", bbo.symbol(), bbo.bid_price(), bbo.ask_price());
                            }
                            _ => {
                                // println!("Other message: {:?}", response.message);
                            }
                        }
                    }
                    None => {
                        println!("Event stream closed.");
                        break;
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!("Stopping...");
                break;
            }
        }
    }

    Ok(())
}