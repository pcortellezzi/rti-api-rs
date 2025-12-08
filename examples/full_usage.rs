use rithmic_rs::{
    RithmicClient, RithmicMessage, 
    MarketDataField, OrderDuration,
    OrderParams, BracketOrderParams, BracketType,
    TransactionType, PriceType,
    connection_info::RithmicCredentials,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 1. Setup Logging
    tracing_subscriber::fmt::init();

    // OPTION A: Load from .env (Generic)
    // Expects RITHMIC_USER, RITHMIC_PASSWORD, etc.
    // Pass Some("TEST") to look for RITHMIC_USER_TEST, etc.
    // let credentials = get_credentials_from_env(None)?;

    // OPTION B: Explicit Credentials (For Applications/GUIs)
    let username = env::var("RITHMIC_USER").unwrap_or("YOUR_USER".to_string());
    let password = env::var("RITHMIC_PASSWORD").unwrap_or("YOUR_PASSWORD".to_string());
    let system   = env::var("RITHMIC_SYSTEM_NAME").unwrap_or("Rithmic Test".to_string());
    let gateway  = env::var("RITHMIC_GATEWAY_NAME").unwrap_or("Test Gateway".to_string());
    
    // Manually specifying system and gateway (Flexible)
    let credentials = RithmicCredentials::new(
        username,
        password,
        system,
        gateway
    );
    
    // Optional: Set direct URL if known/needed
    // let credentials = credentials.with_direct_url("wss://rituz00100.rithmic.com:443");

    println!("Connecting to {} as {}", credentials.system_name, credentials.user);

    // 3. Initialize Client
    let mut client = RithmicClient::new(credentials);


    // 4. Connect (Async)
    // Returns a receiver channel for all asynchronous events (Market Data, Order Updates, etc.)
    let mut event_rx = client.connect().await?;
    
    println!("Connected! Account: {:?}", client.account_info);

    // 5. Spawn a task to handle incoming events
    tokio::spawn(async move {
        while let Some(response) = event_rx.recv().await {
            match response.message {
                RithmicMessage::LastTrade(trade) => {
                    println!("Trade: {} @ {}", trade.symbol.unwrap_or_default(), trade.trade_price.unwrap_or(0.0));
                },
                RithmicMessage::RithmicOrderNotification(notify) => {
                    println!("Order Update: {:?} Status: {:?}", notify.basket_id, notify.status);
                },
                _ => {
                    // Handle other messages
                }
            }
        }
    });

    // 6. Subscribe to Market Data
    client.subscribe_market_data(
        "ESZ5", 
        "CME", 
        Some(vec![MarketDataField::LastTrade, MarketDataField::Bbo])
    ).await?;

    // 7. Place a Standard Limit Order using OrderParams
    let order = OrderParams {
        symbol: "ESZ5".to_string(),
        exchange: "CME".to_string(),
        quantity: 1,
        price: 4500.00,
        transaction_type: TransactionType::Buy,
        price_type: PriceType::Limit,
        duration: OrderDuration::Day,
        user_tag: Some("my_bot_v1".to_string()),
    };

    println!("Submitting Order...");
    if let Err(e) = client.submit_order(order).await {
        eprintln!("Order placement failed: {}", e);
    }

    // 8. Place a Bracket Order (Entry + Profit/Stop)
    let bracket = BracketOrderParams {
        symbol: "ESZ5".to_string(),
        exchange: "CME".to_string(),
        quantity: 1,
        price: 4500.00,
        transaction_type: TransactionType::Buy,
        price_type: PriceType::Limit,
        duration: OrderDuration::Day,
        bracket_type: BracketType::TargetAndStop,
        target_ticks: Some(40), // 10 points (ES tick is 0.25)
        stop_ticks: Some(40),
        user_tag: Some("bracket_v1".to_string()),
    };

    println!("Submitting Bracket Order...");
    if let Err(e) = client.place_bracket_order(bracket).await {
        eprintln!("Bracket order failed: {}", e);
    }

    // 9. Retrieve Order History
    println!("Fetching Order History...");
    let mut history_stream = client.get_order_history(None).await?;
    while let Some(Ok(msg)) = history_stream.recv().await {
        if let RithmicMessage::ResponseShowOrderHistory(hist) = msg.message {
            // ResponseShowOrderHistory contains just the header/ack usually, individual items come as RithmicOrderNotification 
            // or ResponseShowOrderHistoryDetail depending on the specific request flow. 
            // But ShowOrderHistory actually streams RithmicOrderNotification or specific history messages.
            // Let's check the proto definition again. 
            // Actually, Rithmic streams `RithmicOrderNotification` or `ResponseShowOrderHistory` which might be just end of stream or ack.
            // The history items are usually `RithmicOrderNotification`.
            println!("History ACK: {:?}", hist.rp_code);
        }
        if !msg.has_more { break; }
    }

    // 10. Keep main alive
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    
    Ok(())
}
