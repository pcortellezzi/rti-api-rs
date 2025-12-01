# Rithmic-rs

Rust client for the Rithmic R | Protocol API to build algo trading systems.

This library provides a high-level connector for interacting with the Rithmic API, with a focus on ease of use and proper state management. It handles the connection and authentication flow, and provides access to different "plants" for interacting with various aspects of the Rithmic API.

## Features

-   Connect to Rithmic's ticker, history, order, and PNL (Profit and Loss) plants.
-   State management for connections (`Disconnected`, `Connected`, `Authenticated`).
-   Asynchronous API using `tokio`.
-   Protobuf-based communication with the Rithmic API.

## Getting Started

Add `rithmic-rs` to your `Cargo.toml`:

```toml
[dependencies]
rithmic-rs = "0.4.2"
```

### Example Usage

Here's a quick example of how to connect to the Rithmic ticker plant:

```rust
use std::env;
use rithmic_rs::{
    RithmicConnector,
    connection_info::{AccountInfo, RithmicConnectionSystem},
};
use tracing::{Level, event};

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
    let fcm_id = env::var("FCM_ID").expect("RITHMIC_FCM_ID must be set in environment variables");
    let ib_id = env::var("IB_ID").expect("RITHMIC_IB_ID must be set in environment variables");

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

    // Connect to the ticker plant
    let ticker_handle = connector.connect_ticker().await?;

    // Login to the ticker plant
    connector.login_ticker().await?;
    event!(Level::INFO, "Logged into ticker plant");

    // The ticker_handle can now be used to send requests to the ticker plant

    // Disconnect from the Rithmic system
    connector.disconnect().await?;
    event!(Level::INFO, "Disconnected from Rithmic");

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

The `examples` directory contains several examples of how to use the library. To run them, you'll need to create a `.env` file in the `examples` directory with your Rithmic credentials. You can copy the `.env.blank` file to get started.

```bash
cp examples/.env.blank examples/.env
```

Then, edit `examples/.env` with your Rithmic account ID, password, FCM ID, and IB ID.

You can run the examples using `cargo run --example <example_name>`. For example:

```bash
cargo run --example connect
```
