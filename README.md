## Rust Rithmic R | Protocol API client

Unofficial rust client for connecting to Rithmic's R | Protocal API.

https://www.rithmic.com/apis

As is, use at your own risk.

Not all functionality has been implemented, but this is being used to trade live capital through Rithmic.

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

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Contributions encouraged!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.