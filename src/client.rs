use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};
use tracing::{info, error, debug, warn};

use crate::api::decoder::{decode_message, RithmicResponse};
use crate::api::sender_api::RithmicSenderApi;
use crate::connection_info::{AccountInfo, RithmicCredentials, BOOTSTRAP_URL};
use crate::plants::worker::{start_plant_worker, WorkerCommand};
use crate::rti::request_login::SysInfraType;
use crate::rti::request_market_data_update::{Request, UpdateBits};
use crate::rti::request_tick_bar_replay::BarType; 
use crate::rti::request_time_bar_replay;
use crate::rti::request_new_order::{TransactionType, PriceType};
use crate::rti::messages::RithmicMessage;
use crate::ws::{connect, receive_bytes, send_bytes};

/// Information about a trade route
#[derive(Debug, Clone)]
pub struct TradeRouteInfo {
    pub trade_route: String,
    pub exchange: String,
}

/// Main client to interact with Rithmic
pub struct RithmicClient {
    credentials: RithmicCredentials,
    pub account_info: AccountInfo, // Public to access fcm_id etc.
    sender_api: Arc<Mutex<RithmicSenderApi>>,
    
    // Command channels for each plant
    ticker_tx: Option<mpsc::Sender<WorkerCommand>>,
    history_tx: Option<mpsc::Sender<WorkerCommand>>,
    order_tx: Option<mpsc::Sender<WorkerCommand>>,
    
    // Background handles
    handles: Vec<JoinHandle<()>>,
}

impl RithmicClient {
    pub fn new(credentials: RithmicCredentials) -> Self {
        let sender_api = RithmicSenderApi::new();
        Self {
            credentials,
            account_info: AccountInfo::default(),
            sender_api: Arc::new(Mutex::new(sender_api)),
            ticker_tx: None,
            history_tx: None,
            order_tx: None,
            handles: Vec::new(),
        }
    }

    /// Connect to Rithmic System.
    pub async fn connect(&mut self) -> Result<mpsc::Receiver<RithmicResponse>, anyhow::Error> {
        info!("Starting Rithmic Connection Sequence");

        // 1. Determine Gateway URI
        let gateway_uri = if let Some(direct_url) = &self.credentials.direct_gateway_url {
            info!("Using Direct Gateway URL: {}", direct_url);
            direct_url.clone()
        } else {
            info!("Starting Discovery Process...");
            let uri = self.discover_gateway().await?;
            info!("Discovered Gateway: {}", uri);
            uri
        };

        // Create global event channel
        let (event_tx, event_rx) = mpsc::channel(10000);
        
        // 2. Connect Ticker Plant (Market Data)
        let ticker_tx = self.spawn_plant(
            "Ticker Plant",
            &gateway_uri, 
            SysInfraType::TickerPlant, 
            event_tx.clone()
        ).await?;
        self.ticker_tx = Some(ticker_tx);

        // 3. Connect History Plant
        let history_tx = self.spawn_plant(
            "History Plant",
            &gateway_uri,
            SysInfraType::HistoryPlant,
            event_tx.clone()
        ).await?;
        self.history_tx = Some(history_tx);

        // 4. Connect Order Plant
        let order_tx = self.spawn_plant(
            "Order Plant",
            &gateway_uri,
            SysInfraType::OrderPlant,
            event_tx.clone()
        ).await?;
        self.order_tx = Some(order_tx);

        // 5. Fetch Account ID (Crucial for other requests)
        info!("Fetching Account List to determine Account ID...");
        self.fetch_accounts().await?;

        // Subscribe to Order Updates automatically
        self.subscribe_order_updates().await?;

        Ok(event_rx)
    }

    async fn fetch_accounts(&mut self) -> Result<(), anyhow::Error> {
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
             }).await.map_err(|_| anyhow::anyhow!("Order worker unreachable"))?;
             
             match timeout(Duration::from_secs(10), reply_rx).await {
                 Ok(Ok(Ok(resp))) => {
                     if let RithmicMessage::ResponseAccountList(list) = resp.message {
                         if let Some(acc_id) = list.account_id {
                             self.account_info.account_id = acc_id;
                             info!("Account ID set to: {}", self.account_info.account_id);
                         } else {
                             warn!("ResponseAccountList received but no account_id found!");
                         }
                     }
                     Ok(())
                 },
                 Ok(Ok(Err(e))) => Err(anyhow::anyhow!("Fetch Accounts failed: {}", e)),
                 Ok(Err(_)) => Err(anyhow::anyhow!("Fetch Accounts worker error")),
                 Err(_) => Err(anyhow::anyhow!("Fetch Accounts timeout")),
             }
        } else {
            Err(anyhow::anyhow!("Order Plant not connected, cannot fetch accounts"))
        }
    }

    // --- Market Data ---
    
    pub async fn subscribe_market_data(
        &self, 
        symbol: &str, 
        exchange: &str,
        fields: Option<Vec<UpdateBits>>
    ) -> Result<(), anyhow::Error> {
        let mut sender = self.sender_api.lock().await;
        let sub_fields = fields.unwrap_or_else(|| vec![UpdateBits::LastTrade, UpdateBits::Bbo]);
        let (buf, req_id) = sender.request_market_data_update(symbol, exchange, sub_fields, Request::Subscribe);
        drop(sender);
        self.send_single_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id).await
    }

    // --- History ---

    pub async fn replay_tick_bars(
        &self,
        symbol: &str,
        exchange: &str,
        start_time: i32,
        end_time: i32
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, anyhow::Error> { // Now returns a receiver
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_tick_bar_replay(exchange.to_string(), symbol.to_string(), start_time, end_time);
        drop(sender);
        self.send_stream_command_to_plant(&self.history_tx, "History", buf, req_id).await
    }

    pub async fn replay_time_bars(
        &self,
        symbol: &str,
        exchange: &str,
        bar_type: request_time_bar_replay::BarType,
        period: i32,
        start_time: i32,
        end_time: i32
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, anyhow::Error> { // Now returns a receiver
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_time_bar_replay(exchange.to_string(), symbol.to_string(), bar_type, period, start_time, end_time);
        drop(sender);
        self.send_stream_command_to_plant(&self.history_tx, "History", buf, req_id).await
    }

    // --- Orders ---

    async fn subscribe_order_updates(&self) -> Result<(), anyhow::Error> {
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_subscribe_for_order_updates(&self.account_info);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id).await
    }

    pub async fn submit_order(
        &self,
        symbol: &str,
        exchange: &str,
        qty: i32,
        price: f64,
        side: TransactionType,
        order_type: PriceType,
        duration: crate::rti::request_new_order::Duration,
        trade_route: &str, // e.g. "simulator"
    ) -> Result<(), anyhow::Error> {
        let mut sender = self.sender_api.lock().await;
        let local_id = uuid::Uuid::new_v4().to_string();
        
        let (buf, req_id) = sender.request_new_order(
            &self.account_info,
            exchange,
            symbol,
            qty,
            price,
            side,
            order_type,
            duration,
            &local_id,
            trade_route
        );
        drop(sender);
        
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id).await
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<(), anyhow::Error> {
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_cancel_order(&self.account_info, order_id);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id).await
    }

    pub async fn cancel_all_orders(&self) -> Result<(), anyhow::Error> {
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_cancel_all_orders(&self.account_info);
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id).await
    }

    pub async fn list_orders(&self) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, anyhow::Error> {
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_show_orders(&self.account_info);
        drop(sender);
        self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id).await
    }

    pub async fn modify_order(
        &self,
        basket_id: &str,
        exchange: &str,
        symbol: &str,
        qty: i32,
        price: f64,
        order_type: PriceType
    ) -> Result<(), anyhow::Error> {
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_modify_order(
            &self.account_info,
            basket_id,
            exchange,
            symbol,
            qty,
            price,
            order_type
        );
        drop(sender);
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id).await
    }

    pub async fn list_trade_routes(&self) -> Result<Vec<TradeRouteInfo>, anyhow::Error> {
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_trade_routes();
        drop(sender);

        let mut routes = Vec::new();
        let mut stream_rx = self.send_stream_command_to_plant(&self.order_tx, "Order", buf, req_id).await?;

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
                },
                Err(e) => return Err(anyhow::anyhow!("Error receiving trade route: {}", e)),
            }
        }
        Ok(routes)
    }

    pub async fn list_systems(&self) -> Result<Vec<String>, anyhow::Error> {
        info!("Listing Systems from bootstrap URL: {}", BOOTSTRAP_URL);
        let mut stream = connect(BOOTSTRAP_URL).await?;
        let mut sender = self.sender_api.lock().await;
        
        let (req, req_id) = sender.request_rithmic_system_info();
        send_bytes(&mut stream, req).await?;
        drop(sender);

        let mut systems = Vec::new();
        
        let result = timeout(Duration::from_secs(10), async {
            loop {
                 match receive_bytes(&mut stream).await {
                     Ok(Some(bytes)) => {
                         match decode_message(&bytes) {
                             Ok(resp) => {
                                 if resp.request_id == req_id {
                                     if let RithmicMessage::ResponseRithmicSystemInfo(info) = resp.message {
                                         for name in info.system_name {
                                             systems.push(name);
                                         }
                                     }
                                     if !resp.has_more {
                                         break;
                                     }
                                 }
                             }
                             Err(e) => error!("Error decoding system info: {:?}", e),
                         }
                     }
                     Ok(None) => break,
                     Err(e) => return Err(anyhow::anyhow!("Socket error: {}", e)),
                 }
            }
            Ok(())
        }).await;

        match result {
            Ok(Ok(_)) => Ok(systems),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                if !systems.is_empty() {
                    Ok(systems)
                } else {
                    Err(anyhow::anyhow!("Timeout listing systems"))
                }
            }
        }
    }

    async fn discover_gateway(&self) -> Result<String, anyhow::Error> {
        info!("Discovery: Connecting to bootstrap URL: {}", BOOTSTRAP_URL);
        let mut stream = connect(BOOTSTRAP_URL).await?;

        let mut sender = self.sender_api.lock().await;
        
        info!("Discovery: Sending RequestRithmicSystemGatewayInfo for '{}'", self.credentials.system_name);
        let (gw_req, gw_id) = sender.request_rithmic_system_gateway_info(self.credentials.system_name.clone());
        send_bytes(&mut stream, gw_req).await?;
        drop(sender);

        info!("Discovery: Waiting for response (timeout 10s)...");
        
        let loop_result = timeout(Duration::from_secs(10), async {
            loop {
                match receive_bytes(&mut stream).await {
                    Ok(Some(bytes)) => {
                        match decode_message(&bytes) {
                            Ok(resp) => {
                                if resp.request_id == gw_id {
                                    if let RithmicMessage::ResponseRithmicSystemGatewayInfo(info) = resp.message {
                                        let server = info.gateway_uri.first().cloned().unwrap_or_default();
                                        if server.is_empty() {
                                            return Err(anyhow::anyhow!("Empty server name in gateway info"));
                                        }
                                        let uri = if server.starts_with("wss://") || server.starts_with("ws://") {
                                            server
                                        } else {
                                            format!("wss://{}", server)
                                        };
                                        return Ok(uri);
                                    }
                                }
                                if let Some(err) = resp.error {
                                    if resp.request_id == gw_id {
                                        return Err(anyhow::anyhow!("Gateway Request Rejected: {}", err));
                                    }
                                }
                            }
                            Err(e) => {
                                if e.request_id == gw_id {
                                    return Err(anyhow::anyhow!("Discovery request failed: {:?}", e.error));
                                }
                            }
                        }
                    }
                    Ok(None) => return Err(anyhow::anyhow!("Connection closed by server during discovery")),
                    Err(e) => return Err(e),
                }
            }
        }).await;

        match loop_result {
            Ok(res) => res,
            Err(_) => Err(anyhow::anyhow!("Discovery timed out after 10s")),
        }
    }

    async fn spawn_plant(
        &mut self,
        name: &str,
        url: &str,
        infra_type: SysInfraType,
        event_tx: mpsc::Sender<RithmicResponse>,
    ) -> Result<mpsc::Sender<WorkerCommand>, anyhow::Error> {
        info!("Initializing {}", name);
        
        let mut sender = self.sender_api.lock().await;
        let (login_buf, login_id) = sender.request_login(
            &self.credentials.system_name, 
            infra_type, 
            &self.credentials.user, 
            &self.credentials.password
        );
        drop(sender);

        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (login_tx, login_rx) = oneshot::channel();

        let url_clone = url.to_string();
        let name_clone = name.to_string();
        
        let handle = tokio::spawn(async move {
            if let Err(e) = start_plant_worker(url_clone, (login_buf, login_id), cmd_rx, event_tx, login_tx).await {
                error!("{} worker failed: {}", name_clone, e);
            }
        });
        
        self.handles.push(handle);
        
        match timeout(Duration::from_secs(10), login_rx).await {
            Ok(Ok(Ok(login_resp))) => {
                if !login_resp.fcm_id.as_deref().unwrap_or_default().is_empty() {
                    self.account_info.fcm_id = login_resp.fcm_id.unwrap_or_default();
                }
                if !login_resp.ib_id.as_deref().unwrap_or_default().is_empty() {
                    self.account_info.ib_id = login_resp.ib_id.unwrap_or_default();
                }
                
                debug!("Login confirmed for {}. Info: {:?}", name, self.account_info);
            },
            Ok(Ok(Err(e))) => return Err(anyhow::anyhow!("Login refused by Rithmic: {}", e)),
            Ok(Err(_)) => return Err(anyhow::anyhow!("Worker failed to send login status")),
            Err(_) => return Err(anyhow::anyhow!("Login status timed out")),
        }
        
        Ok(cmd_tx)
    }

    async fn send_single_command_to_plant(&self, tx_option: &Option<mpsc::Sender<WorkerCommand>>, plant_name: &str, payload: Vec<u8>, request_id: String) -> Result<(), anyhow::Error> {
        if let Some(tx) = tx_option {
            let (reply_tx, reply_rx) = oneshot::channel();
            
            tx.send(WorkerCommand {
                payload,
                request_id,
                reply_tx: Some(reply_tx),
                stream_tx: None,
            }).await.map_err(|_| anyhow::anyhow!("{} worker unreachable", plant_name))?;

            match reply_rx.await {
                Ok(Ok(resp)) => {
                     if let Some(err) = resp.error {
                         return Err(anyhow::anyhow!("Rithmic Error: {}", err));
                     }
                     Ok(())
                },
                Ok(Err(e)) => Err(anyhow::anyhow!("Request failed: {}", e)),
                Err(_) => Err(anyhow::anyhow!("Worker dropped the request")),
            }
        } else {
            Err(anyhow::anyhow!("{} plant not connected", plant_name))
        }
    }

    // New method for streaming responses
    async fn send_stream_command_to_plant(&self, tx_option: &Option<mpsc::Sender<WorkerCommand>>, plant_name: &str, payload: Vec<u8>, request_id: String) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, anyhow::Error> {
        if let Some(tx) = tx_option {
            let (stream_tx, stream_rx) = mpsc::channel(1000); // Buffer for stream
            
            tx.send(WorkerCommand {
                payload,
                request_id,
                reply_tx: None, // No single reply expected
                stream_tx: Some(stream_tx),
            }).await.map_err(|_| anyhow::anyhow!("{} worker unreachable", plant_name))?;

            Ok(stream_rx)
        } else {
            Err(anyhow::anyhow!("{} plant not connected", plant_name))
        }
    }
}