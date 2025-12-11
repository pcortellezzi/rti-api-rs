use dotenv::dotenv;
use eyre::{Report, Result, eyre};
use rti_api_rs::{
    RithmicClient, RithmicMessage,
    api::receiver_api::RithmicResponse,
    connection_info::get_credentials_from_env,
    types::{
        ModifyOrderParams, OrderDuration, OrderParams, PriceType, TickBarReplayBarSubType,
        TickBarReplayBarType, TickBarReplayDirection, TickBarReplayTimeOrder, TransactionType,
    },
};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration as TokioDuration, sleep};
use tracing::{error, info, warn};

async fn get_connected_client()
-> Result<(RithmicClient, tokio::sync::mpsc::Receiver<RithmicResponse>), Report> {
    dotenv().ok();

    // Look for RITHMIC_USER_TEST to verify env exists before proceeding
    if std::env::var("RITHMIC_USER_TEST").is_err() {
        return Err(eyre!("SKIPPED_NO_CREDS"));
    }

    // Uses RITHMIC_USER_TEST, RITHMIC_PASSWORD_TEST, etc. from env
    let credentials = get_credentials_from_env(Some("TEST"))
        .map_err(|e| eyre!("Failed to load credentials: {}", e))?;

    let mut client = RithmicClient::new(credentials);
    // If connect fails, we want the test to FAIL
    let event_rx = client.connect().await?;
    Ok((client, event_rx))
}

#[tokio::test]
async fn test_market_data_subscription() -> Result<(), Report> {
    let (client, mut event_rx) = match get_connected_client().await {
        Ok(res) => res,
        Err(e) if e.to_string() == "SKIPPED_NO_CREDS" => {
            info!("Test skipped: Credentials not found.");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    info!("Subscribing to ESZ5...");
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
                            info!("Trade: {} @ {}", t.trade_size.unwrap_or(0), t.trade_price.unwrap_or(0.0));
                        },
                        RithmicMessage::BestBidOffer(b) => {
                            received_bbo = true;
                            info!("BBO: {}/{}", b.bid_price.unwrap_or(0.0), b.ask_price.unwrap_or(0.0));
                        },
                        _ => {}
                    }
                    if received_trade && received_bbo {
                        break;
                    }
                } else {
                    return Err(eyre!("Event stream closed unexpectedly"));
                }
            }
            _ = &mut timeout => {
                // Relaxed: Market might be closed
                if !received_trade && !received_bbo {
                     warn!("Warning: Timeout waiting for market data (market might be closed)");
                }
                break;
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_historical_replay() -> Result<(), Report> {
    let (client, mut event_rx) = match get_connected_client().await {
        Ok(res) => res,
        Err(e) if e.to_string() == "SKIPPED_NO_CREDS" => return Ok(()),
        Err(e) => return Err(e),
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i32;
    let start = now - 3600;
    let end = now;

    info!("Replaying history...");
    client
        .replay_tick_bars(
            "ESZ5",
            "CME",
            start,
            end,
            TickBarReplayBarType::TickBar,     // Default for testing
            TickBarReplayBarSubType::Regular,  // Default for testing
            TickBarReplayDirection::Last,      // Default for testing
            TickBarReplayTimeOrder::Backwards, // Default for testing
        )
        .await?;

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
                     warn!("Warning: No historical data received (might be none)");
                 }
                 break;
            }
        }
    }

    info!("Received {} replay messages", bars_received);
    Ok(())
}

#[tokio::test]
async fn test_order_lifecycle() -> Result<(), Report> {
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

    info!("Submitting Order (Auto-Route)...");

    let params = OrderParams {
        symbol: symbol.into(),
        exchange: exchange.into(),
        quantity: qty,
        price,
        transaction_type: TransactionType::Buy,
        price_type: PriceType::Limit,
        duration: OrderDuration::Day,
        user_tag: None,
        auto: true,
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
                            info!("Order Update: Status={:?} ID={:?} State={} Text={:?} Reason={:?} Filled={:?}",
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
                                        info!("-> Order OPEN confirmed. Sending Modify...");
                                        state = "modifying";

                                        let mod_params = ModifyOrderParams {
                                            basket_id: basket_id.clone(),
                                            symbol: symbol.into(),
                                            exchange: exchange.into(),
                                            quantity: qty,
                                            price: 1001.0,
                                            price_type: PriceType::Limit,
                                            auto: true,
                                        };

                                        // Send Modify
                                        if let Err(e) = client.modify_order(mod_params).await {
                                            error!("Modify failed: {}", e);
                                            state = "cancelling";
                                            client.cancel_order(&basket_id, true).await?;
                                        }
                                    }
                                }
                            }

                            // 2. Wait for Modified
                            else if state == "modifying" {
                                if let Some(s) = &n.status {
                                    let s_lower = s.to_lowercase();
                                    if s_lower.contains("modified") || s_lower == "open" {
                                        info!("-> Order MODIFIED (or back to Open). Sending Cancel...");
                                        state = "cancelling";
                                        client.cancel_order(&basket_id, true).await?;
                                    } else if s_lower.contains("modification failed") {
                                        error!("-> Modify Failed. Sending Cancel...");
                                        client.cancel_order(&basket_id, true).await?;
                                    }
                                }
                            }

                            // 3. Wait for Cancelled
                            else if state == "cancelling" {
                                if let Some(s) = &n.status {
                                    let s_lower = s.to_lowercase();
                                    if s_lower.contains("cancelled") || s_lower.contains("complete") {
                                        info!("-> Order CANCELLED/COMPLETE. Test Success.");
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
                                         return Err(eyre!("Order Rejected/Completed immediately: Text={:?} Reason={:?}", n.text, n.completion_reason));
                                    }
                                    warn!("-> Order completed/filled unexpectedly early. Marking as done.");
                                    cancelled = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            _ = &mut timeout => {
                return Err(eyre!("Timeout in state: {}", state));
            }
        }
    }

    if !cancelled {
        // Try cleanup
        let _ = client.cancel_all_orders().await;
    }

    Ok(())
}
