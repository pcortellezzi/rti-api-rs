use eyre::{eyre, Report, Result};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{Duration, timeout};
use tracing::info;

use crate::api::receiver_api::RithmicResponse;
use crate::plants::worker::WorkerCommand;
use crate::types::{
    InstrumentType, MarketDataField, MarketDataRequestType, RithmicMessage, SearchPattern,
};

impl super::RithmicClient {
    /// Subscribes to market data updates for a given symbol.
    pub async fn subscribe_market_data(
        &self,
        symbol: &str,
        exchange: &str,
        fields: Option<Vec<MarketDataField>>,
    ) -> Result<(), Report> {
        info!(
            "Subscribing to market data for {}/{} with fields {:?}",
            symbol, exchange, fields
        );
        let mut sender = self.sender_api.lock().await;
        // Adjusted to use correct MarketDataField variants
        let sub_fields =
            fields.unwrap_or_else(|| vec![MarketDataField::LastTrade, MarketDataField::Bbo]);
        let (buf, req_id) = sender.request_market_data_update(
            symbol,
            exchange,
            sub_fields,
            MarketDataRequestType::Subscribe,
        );
        drop(sender);
        self.send_single_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id)
            .await
    }

    /// Retrieves the front month contract for a given symbol base (e.g., "ES", "NQ").
    pub async fn get_front_month_contract(
        &self,
        symbol: &str,
        exchange: &str,
    ) -> Result<crate::rti::ResponseFrontMonthContract, Report> {
        info!(
            "Requesting front month contract for {}/{}",
            symbol, exchange
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_front_month_contract(symbol, exchange, false); // false for no continuous updates
        drop(sender);

        let (reply_tx, reply_rx) = oneshot::channel();

        if let Some(tx) = &self.ticker_tx {
            tx.send(WorkerCommand {
                payload: buf,
                request_id: req_id,
                reply_tx: Some(reply_tx),
                stream_tx: None,
            })
            .await
            .map_err(|_| eyre!("Ticker plant unreachable"))?;

            match timeout(Duration::from_secs(10), reply_rx).await {
                Ok(Ok(res)) => match res {
                    Ok(resp) => {
                        if let RithmicMessage::ResponseFrontMonthContract(data) = resp.message {
                            Ok(data)
                        } else {
                            Err(eyre!("Unexpected response type: {:?}", resp.message))
                        }
                    }
                    Err(e) => Err(eyre!("Rithmic Error: {}", e)),
                },
                Ok(Err(e)) => Err(eyre!("Request failed: {}", e)),
                Err(_) => Err(eyre!("Timeout waiting for front month contract")),
            }
        } else {
            Err(eyre!("Ticker plant not connected"))
        }
    }

    /// Retrieves instrument information by underlying symbol.
    pub async fn get_instrument_by_underlying(
        &self,
        underlying_symbol: &str,
        exchange: &str,
        expiration_date: Option<&str>,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Requesting instrument by underlying: {} {} {:?}",
            underlying_symbol, exchange, expiration_date
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_get_instrument_by_underlying(
            underlying_symbol,
            exchange,
            expiration_date,
        );
        drop(sender);

        self.send_stream_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id)
            .await
    }

    /// Subscribes to market data updates for a given underlying symbol.
    pub async fn subscribe_market_data_by_underlying(
        &self,
        underlying_symbol: &str,
        exchange: &str,
        expiration_date: Option<&str>,
        fields: Vec<MarketDataField>,
        request_type: MarketDataRequestType,
    ) -> Result<(), Report> {
        info!(
            "Subscribing to market data by underlying: {} {} {:?} with request type {:?}",
            underlying_symbol, exchange, expiration_date, request_type
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_market_data_update_by_underlying(
            underlying_symbol,
            exchange,
            expiration_date,
            fields,
            request_type,
        );
        drop(sender);
        self.send_single_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id)
            .await
    }

    /// Retrieves the tick size type table.
    pub async fn get_tick_size_type_table(
        &self,
        tick_size_type: Option<&str>,
    ) -> Result<crate::rti::ResponseGiveTickSizeTypeTable, Report> {
        info!("Requesting tick size type table for: {:?}", tick_size_type);
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_give_tick_size_type_table(tick_size_type);
        drop(sender);

        let (reply_tx, reply_rx) = oneshot::channel();

        if let Some(tx) = &self.ticker_tx {
            tx.send(WorkerCommand {
                payload: buf,
                request_id: req_id,
                reply_tx: Some(reply_tx),
                stream_tx: None,
            })
            .await
            .map_err(|_| eyre!("Ticker plant unreachable"))?;

            match timeout(Duration::from_secs(10), reply_rx).await {
                Ok(Ok(res)) => match res {
                    Ok(resp) => {
                        if let RithmicMessage::ResponseGiveTickSizeTypeTable(data) = resp.message {
                            Ok(data)
                        } else {
                            Err(eyre!("Unexpected response type: {:?}", resp.message))
                        }
                    }
                    Err(e) => Err(eyre!("Rithmic Error: {}", e)),
                },
                Ok(Err(e)) => Err(eyre!("Request failed: {}", e)),
                Err(_) => Err(eyre!("Timeout waiting for tick size type table")),
            }
        } else {
            Err(eyre!("Ticker plant not connected"))
        }
    }

    /// Retrieves a list of product codes.
    pub async fn get_product_codes(
        &self,
        exchange: Option<&str>,
        give_toi_products_only: Option<bool>,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Requesting product codes for exchange: {:?}, TOI only: {:?}",
            exchange, give_toi_products_only
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_product_codes(exchange, give_toi_products_only);
        drop(sender);
        self.send_stream_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id)
            .await
    }

    /// Retrieves a snapshot of market depth by order.
    pub async fn get_depth_by_order_snapshot(
        &self,
        symbol: &str,
        exchange: &str,
        depth_price: Option<f64>,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Requesting depth by order snapshot for {} {} at price {:?}",
            symbol, exchange, depth_price
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_depth_by_order_snapshot(symbol, exchange, depth_price);
        drop(sender);
        self.send_stream_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id)
            .await
    }

    /// Subscribes to market depth by order updates.
    pub async fn subscribe_depth_by_order_updates(
        &self,
        request_type: crate::rti::request_depth_by_order_updates::Request,
        symbol: &str,
        exchange: &str,
        depth_price: Option<f64>,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Subscribing to depth by order updates for {} {} at price {:?} with request type {:?}",
            symbol, exchange, depth_price, request_type
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) =
            sender.request_depth_by_order_updates(request_type, symbol, exchange, depth_price);
        drop(sender);
        self.send_stream_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id)
            .await
    }

    /// Retrieves volume at price data for a given symbol and exchange.
    pub async fn get_volume_at_price(
        &self,
        symbol: &str,
        exchange: &str,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!("Requesting volume at price for {} {}", symbol, exchange);
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_get_volume_at_price(symbol, exchange);
        drop(sender);
        self.send_stream_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id)
            .await
    }

    /// Retrieves auxiliary reference data for a given symbol and exchange.
    pub async fn get_auxilliary_reference_data(
        &self,
        symbol: &str,
        exchange: &str,
    ) -> Result<crate::rti::ResponseAuxilliaryReferenceData, Report> {
        info!(
            "Requesting auxiliary reference data for {} {}",
            symbol, exchange
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_auxilliary_reference_data(symbol, exchange);
        drop(sender);

        let (reply_tx, reply_rx) = oneshot::channel();

        if let Some(tx) = &self.ticker_tx {
            tx.send(WorkerCommand {
                payload: buf,
                request_id: req_id,
                reply_tx: Some(reply_tx),
                stream_tx: None,
            })
            .await
            .map_err(|_| eyre!("Ticker plant unreachable"))?;

            match timeout(Duration::from_secs(10), reply_rx).await {
                Ok(Ok(res)) => match res {
                    Ok(resp) => {
                        if let RithmicMessage::ResponseAuxilliaryReferenceData(data) = resp.message
                        {
                            Ok(data)
                        } else {
                            Err(eyre!("Unexpected response type: {:?}", resp.message))
                        }
                    }
                    Err(e) => Err(eyre!("Rithmic Error: {}", e)),
                },
                Ok(Err(e)) => Err(eyre!("Request failed: {}", e)),
                Err(_) => Err(eyre!("Timeout waiting for auxiliary reference data")),
            }
        } else {
            Err(eyre!("Ticker plant not connected"))
        }
    }

    /// Searches for symbols matching the given criteria.
    pub async fn search_symbols(
        &self,
        search_text: &str,
        exchange: &str,
        product_code: &str,
        instrument_type: Option<InstrumentType>,
        pattern: Option<SearchPattern>,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed return type
        info!(
            "Searching symbols for '{}' with instrument type {:?} and pattern {:?}",
            search_text, instrument_type, pattern
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_search_symbols(search_text, exchange, product_code, instrument_type, pattern);
        drop(sender);

        self.send_stream_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id)
            .await
    }

    /// Retrieves reference data for a given symbol and exchange.
    pub async fn get_reference_data(
        &self,
        symbol: &str,
        exchange: &str,
    ) -> Result<crate::rti::ResponseReferenceData, Report> {
        info!("Requesting reference data for {}/{}", symbol, exchange);
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_reference_data(symbol, exchange);
        drop(sender);

        let (reply_tx, reply_rx) = oneshot::channel();

        // Send to Ticker Plant
        if let Some(tx) = &self.ticker_tx {
            tx.send(WorkerCommand {
                payload: buf,
                request_id: req_id,
                reply_tx: Some(reply_tx),
                stream_tx: None,
            })
            .await
            .map_err(|_| eyre!("Ticker plant unreachable"))?;

            match timeout(Duration::from_secs(10), reply_rx).await {
                Ok(Ok(res)) => match res {
                    Ok(resp) => {
                        if let RithmicMessage::ResponseReferenceData(data) = resp.message {
                            Ok(data)
                        } else {
                            Err(eyre!("Unexpected response type: {:?}", resp.message))
                        }
                    }
                    Err(e) => Err(eyre!("Rithmic Error: {}", e)),
                },
                Ok(Err(e)) => Err(eyre!("Request failed: {}", e)),
                Err(_) => Err(eyre!("Timeout waiting for reference data")),
            }
        } else {
            Err(eyre!("Ticker plant not connected"))
        }
    }
}
