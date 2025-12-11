use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{Duration, timeout};
use tracing::{debug, error, info, warn};

use crate::api::receiver_api::{RithmicResponse, decode_message};
use crate::api::sender_api::RithmicSenderApi;
use crate::connection_info::{AccountInfo, BOOTSTRAP_URL, RithmicCredentials};
use crate::plants::worker::{WorkerCommand, start_plant_worker};
use crate::types::SysInfraType;
use dashmap::DashMap; // Added DashMap import
use eyre::{Report, Result, eyre}; // Import from types

pub mod agreements;
pub mod history;
pub mod market_data;
pub mod order_management;
pub mod pnl;

/// Information about a trade route
#[derive(Debug, Clone)]
pub struct TradeRouteInfo {
    pub trade_route: String,
    pub exchange: String,
}

/// Main client to interact with Rithmic
pub struct RithmicClient {
    pub credentials: RithmicCredentials,
    pub account_info: AccountInfo,
    sender_api: Arc<Mutex<RithmicSenderApi>>,

    // Command channels for each plant
    ticker_tx: Option<mpsc::Sender<WorkerCommand>>,
    history_tx: Option<mpsc::Sender<WorkerCommand>>,
    order_tx: Option<mpsc::Sender<WorkerCommand>>,
    pnl_tx: Option<mpsc::Sender<WorkerCommand>>,

    // Cache
    trade_routes_cache: Arc<DashMap<String, String>>,

    // Background handles
    handles: Vec<JoinHandle<()>>,
}

impl RithmicClient {
    /// Create a new RithmicClient with the provided credentials.
    pub fn new(credentials: RithmicCredentials) -> Self {
        let sender_api = RithmicSenderApi::new();
        Self {
            credentials,
            account_info: AccountInfo::default(),
            sender_api: Arc::new(Mutex::new(sender_api)),
            ticker_tx: None,
            history_tx: None,
            order_tx: None,
            pnl_tx: None,
            trade_routes_cache: Arc::new(DashMap::new()),
            handles: Vec::new(),
        }
    }

    /// Connect to Rithmic System.
    pub async fn connect(&mut self) -> Result<mpsc::Receiver<RithmicResponse>, Report> {
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
        let ticker_tx = self
            .spawn_plant(
                "Ticker Plant",
                &gateway_uri,
                SysInfraType::TickerPlant,
                event_tx.clone(),
            )
            .await?;
        self.ticker_tx = Some(ticker_tx);

        // 3. Connect History Plant
        let history_tx = self
            .spawn_plant(
                "History Plant",
                &gateway_uri,
                SysInfraType::HistoryPlant, // Reverted back to HistoryPlant
                event_tx.clone(),
            )
            .await?;
        self.history_tx = Some(history_tx);

        // 4. Connect Order Plant
        let order_tx = self
            .spawn_plant(
                "Order Plant",
                &gateway_uri,
                SysInfraType::OrderPlant,
                event_tx.clone(),
            )
            .await?;
        self.order_tx = Some(order_tx);

        // 5. Connect PnL Plant
        let pnl_tx = self
            .spawn_plant(
                "PnL Plant",
                &gateway_uri,
                SysInfraType::PnlPlant,
                event_tx.clone(),
            )
            .await?;
        self.pnl_tx = Some(pnl_tx);

        // 6. Fetch Account ID (Crucial for other requests)
        info!("Fetching Account List to determine Account ID...");
        self.fetch_accounts().await?;

        // Subscribe to Order Updates automatically
        self.subscribe_order_updates().await?;

        // Populate Trade Routes Cache
        if let Err(e) = self.populate_trade_routes_cache().await {
            warn!("Failed to populate trade routes cache: {}", e);
        }

        Ok(event_rx)
    }

    /// Disconnects the Rithmic client by sending a logout request.
    pub async fn logout(&self) -> Result<(), Report> {
        info!("Sending logout request.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_logout();
        drop(sender);
        // Logout request typically goes through the Order Plant.
        self.send_single_command_to_plant(&self.order_tx, "Order", buf, req_id)
            .await
    }

    /// Sends a heartbeat request to the Rithmic system.
    pub async fn heartbeat(&self) -> Result<(), Report> {
        debug!("Sending heartbeat request.");
        let mut sender = self.sender_api.lock().await;
        let (buf, req_id) = sender.request_heartbeat();
        drop(sender);
        // Heartbeat requests usually go through the Ticker Plant.
        self.send_single_command_to_plant(&self.ticker_tx, "Ticker", buf, req_id)
            .await
    }

    async fn populate_trade_routes_cache(&self) -> Result<(), Report> {
        let routes = self.list_trade_routes().await?;
        for r in routes {
            // Store exchange -> trade_route. First one wins.
            if !self.trade_routes_cache.contains_key(&r.exchange) {
                self.trade_routes_cache
                    .insert(r.exchange.clone(), r.trade_route.clone());
            }
        }
        debug!("Trade Routes Cached: {:?}", self.trade_routes_cache);
        Ok(())
    }

    pub async fn list_systems(&self) -> Result<Vec<String>, Report> {
        info!("Listing Systems from bootstrap URL: {}", BOOTSTRAP_URL);
        let mut stream = crate::ws::connect(BOOTSTRAP_URL).await?;
        let mut sender = self.sender_api.lock().await;

        let (req, req_id) = sender.request_rithmic_system_info();
        crate::ws::send_bytes(&mut stream, req).await?;
        drop(sender);

        let mut systems = Vec::new();

        let result = timeout(Duration::from_secs(10), async {
            loop {
                 match crate::ws::receive_bytes(&mut stream).await {
                     Ok(Some(bytes)) => {
                         match decode_message(&bytes) {
                             Ok(resp) => {
                                 if resp.request_id == req_id {
                                     if let crate::rti::messages::RithmicMessage::ResponseRithmicSystemInfo(info) = resp.message {
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
                     Err(e) => return Err(eyre!("Socket error: {}", e)),
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
                    Err(eyre!("Timeout listing systems"))
                }
            }
        }
    }

    #[allow(clippy::collapsible_if)]
    async fn discover_gateway(&self) -> Result<String, Report> {
        // Special handling for Rithmic Test environment
        if self.credentials.system_name == "Rithmic Test" {
            info!("System is 'Rithmic Test', utilizing hardcoded test gateway.");
            return Ok("wss://rituz00100.rithmic.com:443".to_string());
        }

        info!("Discovery: Connecting to bootstrap URL: {}", BOOTSTRAP_URL);
        let mut stream = crate::ws::connect(BOOTSTRAP_URL).await?;

        let mut sender = self.sender_api.lock().await;

        info!(
            "Discovery: Sending RequestRithmicSystemGatewayInfo for '{}'",
            self.credentials.system_name
        );
        let (gw_req, gw_id) =
            sender.request_rithmic_system_gateway_info(self.credentials.system_name.clone());
        crate::ws::send_bytes(&mut stream, gw_req).await?;
        drop(sender);

        info!("Discovery: Waiting for response (timeout 10s)...");

        let loop_result = timeout(Duration::from_secs(10), async {
            loop {
                match crate::ws::receive_bytes(&mut stream).await {
                    Ok(Some(bytes)) => {
                        match decode_message(&bytes) {
                            Ok(resp) => {
                                if resp.request_id == gw_id {
                                    if let crate::rti::messages::RithmicMessage::ResponseRithmicSystemGatewayInfo(info) = resp.message {
                                        let server = info.gateway_uri.first().cloned().unwrap_or_default();
                                        if server.is_empty() {
                                            return Err(eyre!("Empty server name in gateway info"));
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
                                        return Err(eyre!("Gateway Request Rejected: {}", err));
                                    }
                                }
                            }
                            Err(e) => {
                                if e.request_id == gw_id {
                                    return Err(eyre!("Discovery request failed: {:?}", e.error));
                                }
                            }
                        }
                    }
                    Ok(None) => return Err(eyre!("Connection closed by server during discovery")),
                    Err(e) => return Err(e),
                }
            }
        }).await;

        match loop_result {
            Ok(res) => res,
            Err(_) => Err(eyre!("Discovery timed out after 10s")),
        }
    }

    async fn spawn_plant(
        &mut self,
        name: &str,
        url: &str,
        infra_type: SysInfraType,
        event_tx: mpsc::Sender<RithmicResponse>,
    ) -> Result<mpsc::Sender<WorkerCommand>, Report> {
        info!("Initializing {}", name);

        let mut sender = self.sender_api.lock().await;
        let (login_buf, login_id) = sender.request_login(
            &self.credentials.system_name,
            infra_type,
            &self.credentials.user,
            &self.credentials.password,
        );
        drop(sender);

        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (login_tx, login_rx) = oneshot::channel();

        let url_clone = url.to_string();
        let name_clone = name.to_string();

        let handle = tokio::spawn(async move {
            if let Err(e) =
                start_plant_worker(url_clone, (login_buf, login_id), cmd_rx, event_tx, login_tx)
                    .await
            {
                error!("{} worker failed: {}", name_clone, e);
            }
        });

        self.handles.push(handle);

        match timeout(Duration::from_secs(30), login_rx).await {
            // Increased timeout to 30 seconds
            Ok(Ok(Ok(login_resp))) => {
                if !login_resp.fcm_id.as_deref().unwrap_or_default().is_empty() {
                    self.account_info.fcm_id = login_resp.fcm_id.unwrap_or_default();
                }
                if !login_resp.ib_id.as_deref().unwrap_or_default().is_empty() {
                    self.account_info.ib_id = login_resp.ib_id.unwrap_or_default();
                }

                debug!(
                    "Login confirmed for {}. Info: {:?}",
                    name, self.account_info
                );
            }
            Ok(Ok(Err(e))) => return Err(eyre!("Login refused by Rithmic: {}", e)),
            Ok(Err(_)) => return Err(eyre!("Worker failed to send login status")),
            Err(_) => return Err(eyre!("Login status timed out")),
        }

        Ok(cmd_tx)
    }

    async fn send_single_command_to_plant(
        &self,
        tx_option: &Option<mpsc::Sender<WorkerCommand>>,
        plant_name: &str,
        payload: Vec<u8>,
        request_id: String,
    ) -> Result<(), Report> {
        if let Some(tx) = tx_option {
            let (reply_tx, reply_rx) = oneshot::channel();

            tx.send(WorkerCommand {
                payload,
                request_id,
                reply_tx: Some(reply_tx),
                stream_tx: None,
            })
            .await
            .map_err(|_| eyre!("{} worker unreachable", plant_name))?;

            match reply_rx.await {
                Ok(Ok(resp)) => {
                    if let Some(err) = resp.error {
                        return Err(eyre!("Rithmic Error: {}", err));
                    }
                    Ok(())
                }
                Ok(Err(e)) => Err(eyre!("Request failed: {}", e)),
                Err(_) => Err(eyre!("Worker dropped the request")),
            }
        } else {
            Err(eyre!("{} plant not connected", plant_name))
        }
    }

    // New method for streaming responses
    async fn send_stream_command_to_plant(
        &self,
        tx_option: &Option<mpsc::Sender<WorkerCommand>>,
        plant_name: &str,
        payload: Vec<u8>,
        request_id: String,
    ) -> Result<mpsc::Receiver<Result<RithmicResponse, String>>, Report> {
        if let Some(tx) = tx_option {
            let (stream_tx, stream_rx) = mpsc::channel(1000); // Buffer for stream

            tx.send(WorkerCommand {
                payload,
                request_id,
                reply_tx: None, // No single reply expected
                stream_tx: Some(stream_tx),
            })
            .await
            .map_err(|_| eyre!("{} worker unreachable", plant_name))?;

            Ok(stream_rx)
        } else {
            Err(eyre!("{} plant not connected", plant_name))
        }
    }
}
