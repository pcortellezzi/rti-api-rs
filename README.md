# Rithmic-rs

A Rust client for the Rithmic R | Protocol API, providing **100% protocol coverage for all API message templates** to build robust algo trading systems. High-level client abstractions for all exposed APIs are currently under active development to simplify integration.

## Features

-   **Comprehensive API Coverage**: Full implementation of all Rithmic API message templates (requests and responses) as per the official Reference Guide.
-   **Extensible Client Abstractions**: Foundational work for higher-level client functions is in place, with ongoing development to simplify interaction with the comprehensive API.
-   Connect to Rithmic's ticker, history, order, and PNL (Profit and Loss) plants.
-   State management for connections (`Disconnected`, `Connected`, `Authenticated`).
-   Asynchronous API using `tokio`.
-   Protobuf-based communication with the Rithmic API.

## Getting Started

Add `rti-api-rs` to your `Cargo.toml`:

```toml
[dependencies]
rithmic-rs = "0.4.2"
```

### Example Usage

Here's a quick example demonstrating how to connect, subscribe to market data, and listen for incoming messages:

```rust
use rithmic_rs::{
    RithmicClient, RithmicMessage, 
    MarketDataField, 
    connection_info::RithmicCredentials,
};
use std::env;
use tracing::{Level, event};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    // 1. Setup Rithmic Credentials from environment variables
    let credentials = RithmicCredentials::new(
        env::var("RITHMIC_USER").expect("RITHMIC_USER must be set"),
        env::var("RITHMIC_PASSWORD").expect("RITHMIC_PASSWORD must be set"),
        env::var("RITHMIC_SYSTEM_NAME").unwrap_or("Rithmic Test".to_string()),
        env::var("RITHMIC_GATEWAY_NAME").unwrap_or("Test Gateway".to_string())
    );

    event!(Level::INFO, "Connecting to {} as {}", credentials.system_name, credentials.user);

    // 2. Initialize Rithmic Client
    let mut client = RithmicClient::new(credentials);

    // 3. Connect to the Rithmic system
    let mut event_rx = client.connect().await?;
    event!(Level::INFO, "Connected! Account: {:?}", client.account_info);

    // 4. Spawn a task to handle incoming asynchronous events (Market Data, Order Updates, etc.)
    tokio::spawn(async move {
        while let Some(response) = event_rx.recv().await {
            match response.message {
                RithmicMessage::LastTrade(trade) => {
                    event!(Level::INFO, "Trade: {} @ {}", 
                        trade.symbol.unwrap_or_default(), 
                        trade.trade_price.unwrap_or(0.0)
                    );
                },
                RithmicMessage::RithmicOrderNotification(notify) => {
                    event!(Level::INFO, "Order Update: {:?} Status: {:?}", 
                        notify.basket_id.unwrap_or_default(), 
                        notify.status.map(|s| format!("{:?}", s)).unwrap_or_default()
                    );
                },
                _ => {
                    // Handle other messages as needed
                    event!(Level::DEBUG, "Received message: {:?}", response.message);
                }
            }
        }
    });

    // 5. Subscribe to Market Data for ESZ5 on CME
    client.subscribe_market_data(
        "ESZ5", 
        "CME", 
        Some(vec![MarketDataField::LastTrade, MarketDataField::Bbo])
    ).await?;
    event!(Level::INFO, "Subscribed to ESZ5 market data");

    // Keep the main task alive to allow the spawned task to receive events
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    
    event!(Level::INFO, "Disconnecting from Rithmic");
    client.disconnect().await?; // Disconnect when done

    Ok(())
}
```

## How it Works

The library is built around the `RithmicConnector`, which is the main entry point for interacting with the Rithmic API. The connector manages the connection state and provides access to different "plants".

### Plants

A "plant" is a connection to a specific part of the Rithmic API. There are four types of plants:

-   `TickerPlant`: For receiving real-time market data.
-   `HistoryPlant`: For retrieving historical market data.
-   `OrderPlant`: For managing orders.
-   `PnlPlant`: For retrieving profit and loss information.

You can connect to each plant individually using the `RithmicConnector`. Each plant has a corresponding "handle" (e.g., `RithmicTickerPlantHandle`) which you can use to send commands and subscribe to data streams.

### Connection Flow

1.  Create a `RithmicConnector` with your `AccountInfo`.
2.  Call `connect()` to establish a connection to the Rithmic system.
3.  Call `authenticate()` to authenticate the connection.
4.  Connect to the desired plants using `connect_ticker()`, `connect_history()`, `connect_order()`, and `connect_pnl()`.
5.  Login to each plant separately using `login_ticker()`, `login_history()`, etc.
6.  Get a handle to the plant to send commands and receive data.
7.  Call `disconnect()` to close all connections.

## Running the Examples

The `examples` directory contains several examples of how to use the library, including `full_usage.rs` which demonstrates a comprehensive client workflow.

To run them, you'll need to create a `.env` file in the project root with your Rithmic credentials. A template is provided at `examples/.env.blank`. Copy its content to a new `.env` file in the project root and fill in your credentials.

```bash
cp examples/.env.blank .env
```

Then, edit `.env` with your Rithmic username (`RITHMIC_USER`), password (`RITHMIC_PASSWORD`), and optionally system and gateway names (`RITHMIC_SYSTEM_NAME`, `RITHMIC_GATEWAY_NAME`).

You can run the examples using `cargo run --example <example_name>`. For example:

```bash
cargo run --example full_usage
```

## API Compliance Status

| ID | Template Name | Proto Message | Status |
|---|---|---|---|
| 10 | Login Request | RequestLogin | ✅ |
| 11 | Login Response | ResponseLogin | ✅ |
| 12 | Logout Request | RequestLogout | ✅ |
| 13 | Logout Response | ResponseLogout | ✅ |
| 14 | Reference Data Request | RequestReferenceData | ✅ |
| 15 | Reference Data Response | ResponseReferenceData | ✅ |
| 16 | Rithmic System Info Request | RequestRithmicSystemInfo | ✅ |
| 17 | Rithmic System Info Response | ResponseRithmicSystemInfo | ✅ |
| 18 | Request Heartbeat | RequestHeartbeat | ✅ |
| 19 | Response Heartbeat | ResponseHeartbeat | ✅ |
| 20 | Rithmic System Gateway Info Request | RequestRithmicSystemGatewayInfo | ✅ |
| 21 | Rithmic System Gateway Info Response | ResponseRithmicSystemGatewayInfo | ✅ |
| 75 | Reject | Reject | ✅ |
| 76 | User Account Update | UserAccountUpdate | ✅ |
| 77 | Forced Logout | ForcedLogout | ✅ |
| 100 | Market Data Update Request | RequestMarketDataUpdate | ✅ |
| 101 | Market Data Update Response | ResponseMarketDataUpdate | ✅ |
| 102 | Get Instrument by Underlying Request | RequestGetInstrumentByUnderlying | ✅ |
| 103 | Get Instrument by Underlying Response | ResponseGetInstrumentByUnderlying | ✅ |
| 104 | Get Instrument by Underlying Keys Response | ResponseGetInstrumentByUnderlyingKeys | ✅ |
| 105 | Market Data Update by Underlying Request | RequestMarketDataUpdateByUnderlying | ✅ |
| 106 | Market Data Update by Underlying Response | ResponseMarketDataUpdateByUnderlying | ✅ |
| 107 | Give Tick Size Type Table Request | RequestGiveTickSizeTypeTable | ✅ |
| 108 | Give Tick Size Type Table Response | ResponseGiveTickSizeTypeTable | ✅ |
| 109 | Search Symbols Request | RequestSearchSymbols | ✅ |
| 110 | Search Symbols Response | ResponseSearchSymbols | ✅ |
| 111 | Product Codes Request | RequestProductCodes | ✅ |
| 112 | Product Codes Response | ResponseProductCodes | ✅ |
| 113 | Front Month Contract Request | RequestFrontMonthContract | ✅ |
| 114 | Front Month Contract Response | ResponseFrontMonthContract | ✅ |
| 115 | Depth By Order Snapshot Request | RequestDepthByOrderSnapshot | ✅ |
| 116 | Depth By Order Snapshot Response | ResponseDepthByOrderSnapshot | ✅ |
| 117 | Depth By Order Updates Request | RequestDepthByOrderUpdates | ✅ |
| 118 | Depth By Order Updates Response | ResponseDepthByOrderUpdates | ✅ |
| 119 | Get Volume At Price Request | RequestGetVolumeAtPrice | ✅ |
| 120 | Get Volume At Price Response | ResponseGetVolumeAtPrice | ✅ |
| 121 | Auxilliary Reference Data Request | RequestAuxilliaryReferenceData | ✅ |
| 122 | Auxilliary Reference Data Response | ResponseAuxilliaryReferenceData | ✅ |
| 150 | Last Trade | LastTrade | ✅ |
| 151 | Best Bid Offer | BestBidOffer | ✅ |
| 152 | Trade Statistics | TradeStatistics | ✅ |
| 153 | Quote Statistics | QuoteStatistics | ✅ |
| 154 | Indicator Prices | IndicatorPrices | ✅ |
| 155 | End Of Day Prices | EndOfDayPrices | ✅ |
| 156 | Order Book | OrderBook | ✅ |
| 157 | Market Mode | MarketMode | ✅ |
| 158 | Open Interest | OpenInterest | ✅ |
| 159 | Front Month Contract Update | FrontMonthContractUpdate | ✅ |
| 160 | Depth By Order | DepthByOrder | ✅ |
| 161 | Depth By Order End Event | DepthByOrderEndEvent | ✅ |
| 162 | Symbol Margin Rate | SymbolMarginRate | ✅ |
| 163 | Order Price Limits | OrderPriceLimits | ✅ |
| 200 | Time Bar Update Request | RequestTimeBarUpdate | ✅ |
| 201 | Time Bar Update Response | ResponseTimeBarUpdate | ✅ |
| 202 | Time Bar Replay Request | RequestTimeBarReplay | ✅ |
| 203 | Time Bar Replay Response | ResponseTimeBarReplay | ✅ |
| 204 | Tick Bar Update Request | RequestTickBarUpdate | ✅ |
| 205 | Tick Bar Update Response | ResponseTickBarUpdate | ✅ |
| 206 | Tick Bar Replay Request | RequestTickBarReplay | ✅ |
| 207 | Tick Bar Replay Response | ResponseTickBarReplay | ✅ |
| 208 | Volume Profile Minute Bars Request | RequestVolumeProfileMinuteBars | ✅ |
| 209 | Volume Profile Minute Bars Response | ResponseVolumeProfileMinuteBars | ✅ |
| 210 | Resume Bars Request | RequestResumeBars | ✅ |
| 211 | Resume Bars Response | ResponseResumeBars | ✅ |
| 250 | Time Bar | TimeBar | ✅ |
| 251 | Tick Bar | TickBar | ✅ |
| 300 | Login Info Request | RequestLoginInfo | ✅ |
| 301 | Login Info Response | ResponseLoginInfo | ✅ |
| 302 | Account List Request | RequestAccountList | ✅ |
| 303 | Account List Response | ResponseAccountList | ✅ |
| 304 | Account RMS Info Request | RequestAccountRmsInfo | ✅ |
| 305 | Account RMS Info Response | ResponseAccountRmsInfo | ✅ |
| 306 | Product RMS Info Request | RequestProductRmsInfo | ✅ |
| 307 | Product RMS Info Response | ResponseProductRmsInfo | ✅ |
| 308 | Subscribe For Order Updates Request | RequestSubscribeForOrderUpdates | ✅ |
| 309 | Subscribe For Order Updates Response | ResponseSubscribeForOrderUpdates | ✅ |
| 310 | Trade Routes Request | RequestTradeRoutes | ✅ |
| 311 | Trade Routes Response | ResponseTradeRoutes | ✅ |
| 312 | New Order Request | RequestNewOrder | ✅ |
| 313 | New Order Response | ResponseNewOrder | ✅ |
| 314 | Modify Order Request | RequestModifyOrder | ✅ |
| 315 | Modify Order Response | ResponseModifyOrder | ✅ |
| 316 | Cancel Order Request | RequestCancelOrder | ✅ |
| 317 | Cancel Order Response | ResponseCancelOrder | ✅ |
| 318 | Show Order History Dates Request | RequestShowOrderHistoryDates | ✅ |
| 319 | Show Order History Dates Response | ResponseShowOrderHistoryDates | ✅ |
| 320 | Show Orders Request | RequestShowOrders | ✅ |
| 321 | Show Orders Response | ResponseShowOrders | ✅ |
| 322 | Show Order History Request | RequestShowOrderHistory | ✅ |
| 323 | Show Order History Response | ResponseShowOrderHistory | ✅ |
| 324 | Show Order History Summary Request | RequestShowOrderHistorySummary | ✅ |
| 325 | Show Order History Summary Response | ResponseShowOrderHistorySummary | ✅ |
| 326 | Show Order History Detail Request | RequestShowOrderHistoryDetail | ✅ |
| 327 | Show Order History Detail Response | ResponseShowOrderHistoryDetail | ✅ |
| 328 | OCO Order Request | RequestOCOOrder | ✅ |
| 329 | OCO Order Response | ResponseOCOOrder | ✅ |
| 330 | Bracket Order Request | RequestBracketOrder | ✅ |
| 331 | Bracket Order Response | ResponseBracketOrder | ✅ |
| 332 | Update Target Bracket Level Request | RequestUpdateTargetBracketLevel | ✅ |
| 333 | Update Target Bracket Level Response | ResponseUpdateTargetBracketLevel | ✅ |
| 334 | Update Stop Bracket Level Request | RequestUpdateStopBracketLevel | ✅ |
| 335 | Update Stop Bracket Level Response | ResponseUpdateStopBracketLevel | ✅ |
| 336 | Subscribe To Bracket Updates Request | RequestSubscribeToBracketUpdates | ✅ |
| 337 | Subscribe To Bracket Updates Response | ResponseSubscribeToBracketUpdates | ✅ |
| 338 | Show Brackets Request | RequestShowBrackets | ✅ |
| 339 | Show Brackets Response | ResponseShowBrackets | ✅ |
| 340 | Show Bracket Stops Request | RequestShowBracketStops | ✅ |
| 341 | Show Bracket Stops Response | ResponseShowBracketStops | ✅ |
| 342 | List Exchange Permissions Request | RequestListExchangePermissions | ✅ |
| 343 | List Exchange Permissions Response | ResponseListExchangePermissions | ✅ |
| 344 | Link Orders Request | RequestLinkOrders | ✅ |
| 345 | Link Orders Response | ResponseLinkOrders | ✅ |
| 346 | Cancel All Orders Request | RequestCancelAllOrders | ✅ |
| 347 | Cancel All Orders Response | ResponseCancelAllOrders | ✅ |
| 348 | Easy To Borrow List Request | RequestEasyToBorrowList | ✅ |
| 349 | Easy To Borrow List Response | ResponseEasyToBorrowList | ✅ |
| 350 | Trade Route | TradeRoute | ✅ |
| 351 | Rithmic Order Notification | RithmicOrderNotification | ✅ |
| 352 | Exchange Order Notification | ExchangeOrderNotification | ✅ |
| 353 | Bracket Updates | BracketUpdates | ✅ |
| 355 | Update Easy To Borrow List | UpdateEasyToBorrowList | ✅ |
| 356 | Account RMS Updates | AccountRmsUpdates | ✅ |
| 400 | PnL Position Updates Request | RequestPnLPositionUpdates | ✅ |
| 401 | PnL Position Updates Response | ResponsePnLPositionUpdates | ✅ |
| 402 | PnL Position Snapshot Request | RequestPnLPositionSnapshot | ✅ |
| 403 | PnL Position Snapshot Response | ResponsePnLPositionSnapshot | ✅ |
| 450 | Instrument PnL Position Update | InstrumentPnLPositionUpdate | ✅ |
| 451 | Account PnL Position Update | AccountPnLPositionUpdate | ✅ |
| 500 | List Unaccepted Agreements Request | RequestListUnacceptedAgreements | ✅ |
| 501 | List Unaccepted Agreements Response | ResponseListUnacceptedAgreements | ✅ |
| 502 | List Accepted Agreements Request | RequestListAcceptedAgreements | ✅ |
| 503 | List Accepted Agreements Response | ResponseListAcceptedAgreements | ✅ |
| 506 | Show Agreement Request | RequestShowAgreement | ✅ |
| 507 | Show Agreement Response | ResponseShowAgreement | ✅ |
| 3500 | Modify Order Reference Data Request | RequestModifyOrderReferenceData | ✅ |
| 3501 | Modify Order Reference Data Response | ResponseModifyOrderReferenceData | ✅ |
| 3502 | Order Session Config Request | RequestOrderSessionConfig | ✅ |
| 3503 | Order Session Config Response | ResponseOrderSessionConfig | ✅ |
| 3504 | Exit Position Request | RequestExitPosition | ✅ |
| 3505 | Exit Position Response | ResponseExitPosition | ✅ |
| 3506 | Replay Executions Request | RequestReplayExecutions | ✅ |
| 3507 | Replay Executions Response | ResponseReplayExecutions | ✅ |
| 3508 | Account RMS Updates Request | RequestAccountRmsUpdates | ✅ |
| 3509 | Account RMS Updates Response | ResponseAccountRmsUpdates | ✅ |
| 505 | - | ResponseAcceptAgreement | ⚠️ Extra |
| 509 | - | ResponseSetRithmicMrktDataSelfCertStatus | ⚠️ Extra |
| - | - | RequestAcceptAgreement | ⚠️ Undocumented |
| - | - | RequestSetRithmicMrktDataSelfCertStatus | ⚠️ Undocumented |
