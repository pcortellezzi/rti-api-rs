use eyre::{eyre, Report, Result};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{Duration, timeout};
use tracing::{info, warn};

use crate::api::receiver_api::RithmicResponse;
use crate::client::TradeRouteInfo;
use crate::plants::worker::WorkerCommand;
use crate::types::{
    AccountRmsUpdateBits, BracketOrderParams, EasyToBorrowListRequestType,
    ModifyOrderParams, OcoOrderParams, OrderParams, RithmicMessage,
};

impl super::RithmicClient {
    pub(super) async fn fetch_accounts(&mut self) -> Result<(), Report> {
        if let Some(tx) = &self.order_tx {
            let mut sender = self.sender_api.lock().await;
            let (buf, req_id) = sender.request_account_list(&self.account_info);
            drop(sender);

            let (reply_tx, reply_rx) = oneshot::channel();
            tx.send(WorkerCommand {
                payload: buf,
                request_id: req_id,
                reply_tx: Some(reply_tx),
                stream_tx: None, // This is a single response
            })
            .await
            .map_err(|_| eyre!("Order worker unreachable"))?;

            match timeout(Duration::from_secs(10), reply_rx).await {
                Ok(Ok(res)) => {
                    // Changed from Ok(Ok(Ok(resp))) to Ok(Ok(res))
                    match res {
                        // New match arm
                        Ok(resp) => {
                            if let RithmicMessage::ResponseAccountList(list) = resp.message {
                                if let Some(acc_id) = list.account_id {
                                    self.account_info.account_id = acc_id;
                                    info!("Account ID set to: {}", self.account_info.account_id);
                                } else {
                                    warn!("ResponseAccountList received but no account_id found!");
                                }
                            }
                            Ok(())
                        }
                        Err(e) => Err(eyre!("Fetch Accounts failed: {}", e)),
                    }
                }
                Ok(Err(_)) => Err(eyre!("Fetch Accounts worker error")),
                Err(_) => Err(eyre!("Fetch Accounts timeout")),
            }
        } else {
            Err(eyre!("Order Plant not connected, cannot fetch accounts"))
        }
    }

    /// Subscribes to order updates. This is typically called automatically during connection.
    pub async fn subscribe_order_updates(&self) -> Result<(), Report> {
        info!("Subscribing to order updates.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_subscribe_for_order_updates(&self.account_info);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves login information.
    pub async fn get_login_info(&self) -> Result<crate::rti::ResponseLoginInfo, Report> {
        info!("Requesting login info.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_login_info();
        drop(sender);

        let (reply_tx, reply_rx) = oneshot::channel();

        if let Some(tx) = &self.order_tx {
            tx.send(WorkerCommand {
                payload: buf,
                request_id: req_id,
                reply_tx: Some(reply_tx),
                stream_tx: None,
            })
            .await
            .map_err(|_| eyre!("Order plant unreachable"))?;

            match timeout(Duration::from_secs(10), reply_rx).await {
                Ok(Ok(res)) => match res {
                    Ok(resp) => {
                        if let RithmicMessage::ResponseLoginInfo(data) = resp.message {
                            Ok(data)
                        } else {
                            Err(eyre!("Unexpected response type: {:?}", resp.message))
                        }
                    }
                    Err(e) => Err(eyre!("Rithmic Error: {}", e)),
                },
                Ok(Err(e)) => Err(eyre!("Request failed: {}", e)),
                Err(_) => Err(eyre!("Timeout waiting for login info")),
            }
        } else {
            Err(eyre!("Order plant not connected"))
        }
    }

    /// Retrieves Account RMS (Risk Management System) information.
    pub async fn get_account_rms_info(
        &self,
        user_type: crate::rti::request_account_rms_info::UserType, // Keep RTI type for now, consider abstracting if needed
    ) -> Result<crate::rti::ResponseAccountRmsInfo, Report> {
        info!("Requesting Account RMS Info for user type {:?}", user_type);
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_account_rms_info(&self.account_info, user_type);
        drop(sender);

        let (reply_tx, reply_rx) = oneshot::channel();

        if let Some(tx) = &self.order_tx {
            tx.send(WorkerCommand {
                payload: buf,
                request_id: req_id,
                reply_tx: Some(reply_tx),
                stream_tx: None,
            })
            .await
            .map_err(|_| eyre!("Order plant unreachable"))?;

            match timeout(Duration::from_secs(10), reply_rx).await {
                Ok(Ok(res)) => match res {
                    Ok(resp) => {
                        if let RithmicMessage::ResponseAccountRmsInfo(data) = resp.message {
                            Ok(data)
                        } else {
                            Err(eyre!("Unexpected response type: {:?}", resp.message))
                        }
                    }
                    Err(e) => Err(eyre!("Rithmic Error: {}", e)),
                },
                Ok(Err(e)) => Err(eyre!("Request failed: {}", e)),
                Err(_) => Err(eyre!("Timeout waiting for account RMS info")),
            }
        } else {
            Err(eyre!("Order plant not connected"))
        }
    }

    /// Retrieves Product RMS (Risk Management System) information.
    pub async fn get_product_rms_info(&self) -> Result<crate::rti::ResponseProductRmsInfo, Report> {
        info!("Requesting Product RMS Info.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_product_rms_info(&self.account_info);
        drop(sender);

        let (reply_tx, reply_rx) = oneshot::channel();

        if let Some(tx) = &self.order_tx {
            tx.send(WorkerCommand {
                payload: buf,
                request_id: req_id,
                reply_tx: Some(reply_tx),
                stream_tx: None,
            })
            .await
            .map_err(|_| eyre!("Order plant unreachable"))?;

            match timeout(Duration::from_secs(10), reply_rx).await {
                Ok(Ok(res)) => match res {
                    Ok(resp) => {
                        if let RithmicMessage::ResponseProductRmsInfo(data) = resp.message {
                            Ok(data)
                        } else {
                            Err(eyre!("Unexpected response type: {:?}", resp.message))
                        }
                    }
                    Err(e) => Err(eyre!("Rithmic Error: {}", e)),
                },
                Ok(Err(e)) => Err(eyre!("Request failed: {}", e)),
                Err(_) => Err(eyre!("Timeout waiting for product RMS info")),
            }
        } else {
            Err(eyre!("Order plant not connected"))
        }
    }

    /// Retrieves a list of dates for which order history is available.
    pub async fn get_order_history_dates(
        &self,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!("Requesting order history dates.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_show_order_history_dates();
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves a summary of the order history for a specific date.
    pub async fn get_order_history_summary(
        &self,
        date: &str,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!("Requesting order history summary for date: {}", date);
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_show_order_history_summary(&self.account_info, date);
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves detailed order history for a specific date and optional basket ID.
    pub async fn get_order_history_detail(
        &self,
        basket_id: Option<&str>,
        date: &str,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Requesting detailed order history for date {} and basket ID {:?}",
            date, basket_id
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) =
            sender.request_show_order_history_detail(&self.account_info, basket_id, date);
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Updates the target bracket level of an existing bracket order.
    pub async fn update_target_bracket_level(
        &self,
        basket_id: &str,
        level: i32,
        target_ticks: i32,
    ) -> Result<(), Report> {
        info!(
            "Updating target bracket level for basket ID {} to level {} with {} ticks.",
            basket_id, level, target_ticks
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_update_target_bracket_level(
            &self.account_info,
            basket_id,
            level,
            target_ticks,
        );
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Updates the stop bracket level of an existing bracket order.
    pub async fn update_stop_bracket_level(
        &self,
        basket_id: &str,
        level: i32,
        stop_ticks: i32,
    ) -> Result<(), Report> {
        info!(
            "Updating stop bracket level for basket ID {} to level {} with {} ticks.",
            basket_id, level, stop_ticks
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_update_stop_bracket_level(
            &self.account_info,
            basket_id,
            level,
            stop_ticks,
        );
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Subscribes to bracket order updates.
    pub async fn subscribe_to_bracket_updates(&self) -> Result<(), Report> {
        info!("Subscribing to bracket updates.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_subscribe_to_bracket_updates(&self.account_info);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves a list of active bracket orders.
    pub async fn show_brackets(
        &self,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!("Requesting active bracket orders.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_show_brackets(&self.account_info);
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves a list of active bracket stop orders.
    pub async fn show_bracket_stops(
        &self,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!("Requesting active bracket stop orders.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_show_bracket_stops(&self.account_info);
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves a list of exchange permissions for the logged-in user.
    pub async fn list_exchange_permissions(
        &self,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!("Requesting list of exchange permissions.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_list_exchange_permissions(None); // Assuming user is None for client call
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Links two existing orders.
    pub async fn link_orders(&self, order_id_1: &str, order_id_2: &str) -> Result<(), Report> {
        info!("Linking orders {} and {}.", order_id_1, order_id_2);
        let mut sender = self.sender_api.lock().await;
        // The sender_api request_link_orders expects Vec<String> for basket_ids
        let basket_ids = vec![order_id_1.to_string(), order_id_2.to_string()];
        let (buf, req_id) = sender.request_link_orders(&self.account_info, basket_ids);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves a list of easy-to-borrow instruments.
    pub async fn get_easy_to_borrow_list(
        &self,
        request_type: EasyToBorrowListRequestType,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Requesting easy-to-borrow list with type {:?}",
            request_type
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_easy_to_borrow_list(request_type);
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Modifies the reference data for an existing order.
    pub async fn modify_order_reference_data(
        &self,
        basket_id: &str,
        user_tag: Option<String>,
    ) -> Result<(), Report> {
        info!(
            "Modifying reference data for basket ID {} to user_tag {:?}.",
            basket_id, user_tag
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) =
            sender.request_modify_order_reference_data(&self.account_info, basket_id, user_tag);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves the order session configuration.
    pub async fn get_order_session_config(
        &self,
        should_defer_request: Option<bool>,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Requesting order session configuration. Should defer: {:?}",
            should_defer_request
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_order_session_config(should_defer_request);
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Exits a position for a given account and instrument.
    pub async fn exit_position(
        &self,
        window_name: Option<&str>,
        symbol: Option<&str>,
        exchange: Option<&str>,
    ) -> Result<(), Report> {
        info!(
            "Exiting position for account: {:?}, symbol: {:?}, exchange: {:?}",
            self.account_info.account_id, symbol, exchange
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_exit_position(
            &self.account_info,
            window_name,
            symbol,
            exchange,
        );
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Replays historical executions for a given account.
    pub async fn replay_executions(
        &self,
        start_index: i32,
        finish_index: i32,
    ) -> Result<(), Report> {
        info!(
            "Replaying historical executions for account {:?} from {} to {}.",
            self.account_info.account_id, start_index, finish_index
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) =
            sender.request_replay_executions(&self.account_info, start_index, finish_index);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Subscribes to account RMS updates.
    pub async fn subscribe_account_rms_updates(
        &self,
        _subscribe: bool,
        update_bits: Vec<AccountRmsUpdateBits>,
    ) -> Result<(), Report> {
        info!(
            "Subscribing to account RMS updates. Subscribe: {}, Update bits: {:?}",
            _subscribe, update_bits
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) =
            sender.request_account_rms_updates(&self.account_info, _subscribe, update_bits);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    pub async fn submit_order(&self, params: OrderParams) -> Result<(), Report> {
        let route = if let Some(r) = self.trade_routes_cache.get(&params.exchange) {
            r.value().clone()
        } else {
            return Err(eyre!(
                "Trade route not found in cache for exchange '{}'.",
                params.exchange
            ));
        };

        info!(
            "Placing order for {}/{} with route {}",
            params.symbol, params.exchange, route
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_new_order(&self.account_info, params, &route);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    pub async fn cancel_order(&self, basket_id: &str, auto: bool) -> Result<(), Report> {
        info!("Cancelling order with basket ID: {}", basket_id);
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_cancel_order(&self.account_info, basket_id, auto);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    pub async fn cancel_all_orders(&self) -> Result<(), Report> {
        info!(
            "Cancelling all orders for account: {:?}",
            self.account_info.account_id
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_cancel_all_orders(&self.account_info);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    pub async fn list_orders(
        &self,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Listing active orders for account: {:?}",
            self.account_info.account_id
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_show_orders(&self.account_info);
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Retrieves the order history for the current session.
    ///
    /// Optional `basket_id` can be used to filter by a specific order/basket ID.
    /// Returns a stream of `RithmicResponse`.
    pub async fn get_order_history(
        &self,
        basket_id: Option<&str>,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        // Changed to RithmicResponse
        info!(
            "Requesting order history for account: {:?} with basket ID: {:?}",
            self.account_info.account_id, basket_id
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_show_order_history(&self.account_info, basket_id);
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    pub async fn modify_order(&self, params: ModifyOrderParams) -> Result<(), Report> {
        info!("Modifying order with basket ID: {}", params.basket_id);
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_modify_order(&self.account_info, params);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    pub async fn place_bracket_order(&self, params: BracketOrderParams) -> Result<(), Report> {
        let route = if let Some(r) = self.trade_routes_cache.get(&params.exchange) {
            r.value().clone()
        } else {
            return Err(eyre!(
                "Trade route not found in cache for exchange '{}'.",
                params.exchange
            ));
        };

        info!(
            "Placing bracket order for {}/{} with route {}",
            params.symbol, params.exchange, route
        );
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_bracket_order(&self.account_info, params, &route);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Place an OCO (One-Cancels-Other) order with two legs.
    pub async fn place_oco_order(&self, params: OcoOrderParams) -> Result<(), Report> {
        // Assuming same exchange for route lookup
        let route = if let Some(r) = self.trade_routes_cache.get(&params.leg1.exchange) {
            r.value().clone()
        } else {
            return Err(eyre!(
                "Trade route not found in cache for exchange '{}'.",
                params.leg1.exchange
            ));
        };
        info!(
            "Placing OCO order for {}/{} with route {}",
            params.leg1.symbol, params.leg1.exchange, route
        );

        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_oco_order(&self.account_info, params, &route);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    pub async fn list_trade_routes(&self) -> Result<Vec<TradeRouteInfo>, Report> {
        info!("Listing trade routes.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_trade_routes();
        drop(sender);

        let mut routes = Vec::new();
        let mut stream_rx = self
            .send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await?;

        while let Some(res) = stream_rx.recv().await {
            match res {
                Ok(resp) => {
                    if let RithmicMessage::ResponseTradeRoutes(r) = resp.message {
                        if let Some(tr) = r.trade_route {
                            routes.push(TradeRouteInfo {
                                trade_route: tr,
                                exchange: r.exchange.unwrap_or_default(),
                            });
                        }
                        if !resp.has_more {
                            break;
                        }
                    }
                }
                Err(e) => return Err(eyre!("Error receiving trade route: {}", e)),
            }
        }
        Ok(routes)
    }
}
