## Rust Rithmic R | Protocol API client

https://www.rithmic.com/apis


As is, use at your own risk.

Only `order_plant`, `ticker_plant`, and `pnl_plant` are provided. It uses the actor pattern so you'll want to start the plant, and communicate with it using the handle.

### Example Usage:

```rust
pub async fn stream_live_ticks(&self, env: &RithmicConnectionSystem, account_info: &AccountInfo) {
    event!(Level::INFO, "market-data streaming ticks");

    let ticker_plant = RithmicTickerPlant::new(env, account_info).await;
    let ticker_plant_handle = ticker_plant.get_handle();

    let mut min_backoff_wait = 1;

    while let Err(err) = ticker_plant_handle.login().await {
        event!(Level::ERROR, "market-data: login failed: {}", err);

        sleep(Duration::from_secs(min_backoff_wait)).await;

        min_backoff_wait *= 2;

        if min_backoff_wait > 60 {
            event!(Level::ERROR, "market-data: login exceeded max backoff");

            panic!("market-data: login exceeded max backoff")
        }
    }

    for (base_symbol, symbol) in self.base_symbol_to_symbol_map.iter() {
        let _ = ticker_plant_handle
            .subscribe(symbol, base_symbol.get_exchange().as_str())
            .await;
    }
}
```
