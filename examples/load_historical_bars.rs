use rithmic_rs::{RithmicClient, connection_info::{RithmicConnectionSystem, get_credentials_from_env}, RithmicMessage};
use dotenv::dotenv;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let env_type = RithmicConnectionSystem::Demo;
    let credentials = get_credentials_from_env(&env_type);

    println!("Connecting to Rithmic...");
    let mut client = RithmicClient::new(credentials);
    let mut event_rx = client.connect().await?;
    
    let symbol = "ESZ5"; // Adjust if needed
    let exchange = "CME";
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i32;
    let start_time = now - 3600; // 1 hour ago
    let end_time = now;

    println!("Requesting Tick Bar Replay for {} from {} to {}", symbol, start_time, end_time);
    
    match client.replay_tick_bars(symbol, exchange, start_time, end_time).await {
        Ok(_) => println!("Replay request sent."),
        Err(e) => {
            println!("Failed to send replay request: {}", e);
            return Ok(());
        }
    }

    println!("Waiting for replay data...");
    let mut count = 0;
    
    loop {
        let msg = event_rx.recv().await;
        match msg {
            Some(response) => {
                match response.message {
                    RithmicMessage::ResponseTickBarReplay(replay) => {
                        // Inspect the data
                        println!("Bar: Time={:?} Close={:?} Vol={:?}", 
                            replay.data_bar_ssboe.first(), // Just peek first timestamp if any
                            replay.close_price,
                            replay.volume
                        );
                        count += 1;
                        
                        if !response.has_more {
                            println!("Replay End detected (has_more=false). Total bars: {}", count);
                            break;
                        }
                    },
                    _ => {}
                }
            }
            None => break,
        }
    }

    Ok(())
}
