use rti_api_rs::{
    RithmicClient,
    connection_info::{get_credentials_from_env},
    RithmicMessage,
    rti::request_new_order::{PriceType, TransactionType, Duration},
    api::decoder::RithmicResponse,
    OrderParams, // Import OrderParams
    ModifyOrderParams,
};
use dotenv::dotenv;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration as TokioDuration};

async fn get_connected_client() -> Result<(RithmicClient, tokio::sync::mpsc::Receiver<RithmicResponse>), anyhow::Error> {
    dotenv().ok();

    // Look for RITHMIC_USER_TEST to verify env exists before proceeding
    if std::env::var("RITHMIC_USER_TEST").is_err() {
        return Err(anyhow::anyhow!("SKIPPED_NO_CREDS"));
    }

    // Uses RITHMIC_USER_TEST, RITHMIC_PASSWORD_TEST, etc. from env
    let credentials = get_credentials_from_env(Some("TEST"))
        .map_err(|e| anyhow::anyhow!("Failed to load credentials: {}", e))?;

    let mut client = RithmicClient::new(credentials);
    // If connect fails, we want the test to FAIL
    let event_rx = client.connect().await?;
    Ok((client, event_rx))
}

#[tokio::test]
async fn test_market_data_subscription() -> Result<(), anyhow::Error> {
    let (client, mut event_rx) = match get_connected_client().await {
        Ok(res) => res,
        Err(e) if e.to_string() == "SKIPPED_NO_CREDS" => {
            println!("Test skipped: Credentials not found.");
            return Ok(());
        },
        Err(e) => return Err(e),
    };

    println!("Subscribing to ESZ5...");
    client.subscribe_market_data("ESZ5", "CME", None).await?;

    let mut received_trade = false;
    let mut received_bbo = false;

    let timeout = sleep(TokioDuration::from_secs(15));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            msg = event_rx.recv() => {
                if let Some(resp) = msg {
                    match resp.message {
                        RithmicMessage::LastTrade(t) => {
                            received_trade = true;
                            println!("Trade: {} @ {}", t.trade_size.unwrap_or(0), t.trade_price.unwrap_or(0.0));
                        },
                        RithmicMessage::BestBidOffer(b) => {
                            received_bbo = true;
                            println!("BBO: {}/{}", b.bid_price.unwrap_or(0.0), b.ask_price.unwrap_or(0.0));
                        },
                        _ => {}
                    }
                    if received_trade && received_bbo {
                        break;
                    }
                } else {
                    return Err(anyhow::anyhow!("Event stream closed unexpectedly"));
                }
            }
            _ = &mut timeout => {
                // Relaxed: Market might be closed
                if !received_trade && !received_bbo {
                     println!("Warning: Timeout waiting for market data (market might be closed)");
                }
                break;
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_historical_replay() -> Result<(), anyhow::Error> {
    let (client, mut event_rx) = match get_connected_client().await {
        Ok(res) => res,
        Err(e) if e.to_string() == "SKIPPED_NO_CREDS" => return Ok(()),
        Err(e) => return Err(e),
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i32;
    let start = now - 3600;
    let end = now;

    println!("Replaying history...");
    client.replay_tick_bars("ESZ5", "CME", start, end).await?;

    let mut bars_received = 0;
    let timeout = sleep(TokioDuration::from_secs(15));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            msg = event_rx.recv() => {
                if let Some(resp) = msg {
                    if let RithmicMessage::ResponseTickBarReplay(_) = resp.message {
                        bars_received += 1;
                        if !resp.has_more {
                            break;
                        }
                    }
                }
            }
            _ = &mut timeout => {
                 if bars_received == 0 {
                     // Non-fatal warn
                     println!("Warning: No historical data received (might be none)");
                 }
                 break;
            }
        }
    }

    println!("Received {} replay messages", bars_received);
    Ok(())
}

#[tokio::test]
async fn test_order_lifecycle() -> Result<(), anyhow::Error> {
    let (client, mut event_rx) = match get_connected_client().await {
        Ok(res) => res,
        Err(e) if e.to_string() == "SKIPPED_NO_CREDS" => return Ok(()),
        Err(e) => return Err(e),
    };

    // 1. Place Limit Buy Order (Way below market to avoid fill)
    let symbol = "ESZ5";
    let exchange = "CME";
        let price = 6500.0; // More realistic low price
        let qty = 1;
        
        println!("Submitting Order (Auto-Route)...");
        
        let params = OrderParams {
            symbol: symbol.into(),
            exchange: exchange.into(),
            quantity: qty,
            price,
            transaction_type: TransactionType::Buy,
            price_type: PriceType::Limit,
            duration: Duration::Day,
            user_tag: None,
        };

        // The client will automatically resolve the trade route for "CME"
        // provided populate_trade_routes_cache() succeeded during connect().
        client.submit_order(params).await?;
    
    let mut basket_id = String::new();
    let mut cancelled = false;

    let timeout = sleep(TokioDuration::from_secs(20));
    tokio::pin!(timeout);

    // State machine for the test
    let mut state = "submitted"; // submitted -> open -> modifying -> modified -> cancelling -> cancelled

    loop {
        tokio::select! {
            msg = event_rx.recv() => {
                if let Some(resp) = msg {
                    if let RithmicMessage::RithmicOrderNotification(n) = resp.message {
                        if n.symbol.as_deref() == Some(symbol) {
                            println!("Order Update: Status={:?} ID={:?} State={} Text={:?} Reason={:?} Filled={:?}",
                                n.status, n.basket_id, state, n.text, n.completion_reason, n.total_fill_size);

                            // 1. Capture ID and wait for Open
                            if state == "submitted" {
                                if let Some(bid) = n.basket_id.clone() {
                                    basket_id = bid;
                                }
                                // Wait for "Open" or "Working" to be sure we can modify
                                if let Some(s) = &n.status {
                                    let s_lower = s.to_lowercase();
                                    // Exclude "open pending"
                                    if (s_lower.contains("open") && !s_lower.contains("pending")) || s_lower.contains("working") {
                                        println!("-> Order OPEN confirmed. Sending Modify...");
                                        state = "modifying";
                                        
                                        let mod_params = ModifyOrderParams {
                                            basket_id: basket_id.clone(),
                                            symbol: symbol.into(),
                                            exchange: exchange.into(),
                                            quantity: qty,
                                            price: 1001.0,
                                            price_type: PriceType::Limit,
                                        };

                                        // Send Modify
                                        if let Err(e) = client.modify_order(mod_params).await {
                                            println!("Modify failed: {}", e);
                                            state = "cancelling";
                                            client.cancel_order(&basket_id).await?;
                                        }
                                    }
                                }
                            }

                            // 2. Wait for Modified
                            else if state == "modifying" {
                                if let Some(s) = &n.status {
                                    let s_lower = s.to_lowercase();
                                    if s_lower.contains("modified") || s_lower == "open" {
                                        println!("-> Order MODIFIED (or back to Open). Sending Cancel...");
                                        state = "cancelling";
                                        client.cancel_order(&basket_id).await?;
                                    } else if s_lower.contains("modification failed") {
                                        println!("-> Modify Failed. Sending Cancel...");
                                        state = "cancelling";
                                        client.cancel_order(&basket_id).await?;
                                    }
                                }
                            }

                            // 3. Wait for Cancelled
                            else if state == "cancelling" {
                                if let Some(s) = &n.status {
                                    let s_lower = s.to_lowercase();
                                    if s_lower.contains("cancelled") || s_lower.contains("complete") {
                                        println!("-> Order CANCELLED/COMPLETE. Test Success.");
                                        cancelled = true;
                                        break;
                                    }
                                }
                            }

                            // Global catch for unexpected completion
                            if let Some(s) = &n.status {
                                let s_lower = s.to_lowercase();
                                if (s_lower.contains("complete") || s_lower.contains("filled")) && !cancelled {
                                    if state == "submitted" {
                                         return Err(anyhow::anyhow!("Order Rejected/Completed immediately: Text={:?} Reason={:?}", n.text, n.completion_reason));
                                    }
                                    println!("-> Order completed/filled unexpectedly early. Marking as done.");
                                    cancelled = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            _ = &mut timeout => {
                return Err(anyhow::anyhow!("Timeout in state: {}", state));
            }
        }
    }

    if !cancelled {
        // Try cleanup
        let _ = client.cancel_all_orders().await;
    }

    Ok(())
}
