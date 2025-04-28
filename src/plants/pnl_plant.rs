use async_trait::async_trait;
use tracing::{Level, event};

use crate::{
    api::{
        receiver_api::{RithmicReceiverApi, RithmicResponse},
        sender_api::RithmicSenderApi,
    },
    connection_info::{self, AccountInfo},
    request_handler::{RithmicRequest, RithmicRequestHandler},
    rti::{request_login::SysInfraType, request_pn_l_position_updates},
    ws::{PlantActor, RithmicStream, get_heartbeat_interval},
};

use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};

use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc, oneshot},
    time::Interval,
};

use tokio_tungstenite::{
    MaybeTlsStream, connect_async,
    tungstenite::{Error, Message},
};

pub enum PnlPlantCommand {
    Close,
    Login {
        response_sender: oneshot::Sender<Result<Vec<RithmicResponse>, String>>,
    },
    SetLogin,
    Logout {
        response_sender: oneshot::Sender<Result<Vec<RithmicResponse>, String>>,
    },
    PnlPositionSnapshots {
        response_sender: oneshot::Sender<Result<Vec<RithmicResponse>, String>>,
    },
    SendHeartbeat {},
    SubscribePnlUpdates {
        response_sender: oneshot::Sender<Result<Vec<RithmicResponse>, String>>,
    },
}

pub struct RithmicPnlPlant {
    pub connection_handle: tokio::task::JoinHandle<()>,
    sender: mpsc::Sender<PnlPlantCommand>,
    subscription_sender: broadcast::Sender<RithmicResponse>,
}

impl RithmicPnlPlant {
    pub async fn new(account_info: &AccountInfo) -> RithmicPnlPlant {
        let (req_tx, req_rx) = mpsc::channel::<PnlPlantCommand>(32);
        let (sub_tx, _sub_rx) = broadcast::channel(1024);

        let mut pnl_plant = PnlPlant::new(req_rx, sub_tx.clone(), account_info)
            .await
            .unwrap();

        let connection_handle = tokio::spawn(async move {
            pnl_plant.run().await;
        });

        RithmicPnlPlant {
            connection_handle,
            sender: req_tx,
            subscription_sender: sub_tx,
        }
    }
}

impl RithmicStream for RithmicPnlPlant {
    type Handle = RithmicPnlPlantHandle;

    fn get_handle(&self) -> Self::Handle {
        RithmicPnlPlantHandle {
            sender: self.sender.clone(),
            subscription_receiver: self.subscription_sender.subscribe(),
        }
    }
}

#[derive(Debug)]
pub struct PnlPlant {
    config: connection_info::RithmicConnectionInfo,
    interval: Interval,
    logged_in: bool,
    request_handler: RithmicRequestHandler,
    request_receiver: mpsc::Receiver<PnlPlantCommand>,
    rithmic_reader: SplitStream<tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>>,
    rithmic_receiver_api: RithmicReceiverApi,
    rithmic_sender: SplitSink<
        tokio_tungstenite::WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::Message,
    >,
    rithmic_sender_api: RithmicSenderApi,
    subscription_sender: broadcast::Sender<RithmicResponse>,
}

impl PnlPlant {
    async fn new(
        request_receiver: mpsc::Receiver<PnlPlantCommand>,
        subscription_sender: broadcast::Sender<RithmicResponse>,
        account_info: &AccountInfo,
    ) -> Result<PnlPlant, ()> {
        let config = connection_info::get_config(&account_info.env);

        let (ws_stream, _) = connect_async(&config.url).await.expect("Failed to connect");
        let (rithmic_sender, rithmic_reader) = ws_stream.split();

        let rithmic_sender_api = RithmicSenderApi::new(account_info);
        let rithmic_receiver_api = RithmicReceiverApi {
            source: "pnl_plant".to_string(),
        };

        let interval = get_heartbeat_interval();

        Ok(PnlPlant {
            config,
            interval,
            logged_in: false,
            request_handler: RithmicRequestHandler::new(),
            request_receiver,
            rithmic_reader,
            rithmic_receiver_api,
            rithmic_sender_api,
            rithmic_sender,
            subscription_sender,
        })
    }
}

#[async_trait]
impl PlantActor for PnlPlant {
    type Command = PnlPlantCommand;

    async fn run(&mut self) {
        loop {
            tokio::select! {
                _ = self.interval.tick() => {
                    self.handle_command(PnlPlantCommand::SendHeartbeat {}).await;
                }
                Some(message) = self.request_receiver.recv() => {
                    self.handle_command(message).await;
                }
                Some(message) = self.rithmic_reader.next() => {
                    let stop = self.handle_rithmic_message(message).await.unwrap();

                    if stop {
                        break;
                    }
                }
                else => { break; }
            }
        }
    }

    async fn handle_rithmic_message(
        &mut self,
        message: Result<Message, Error>,
    ) -> Result<bool, ()> {
        let mut stop = false;

        match message {
            Ok(Message::Close(frame)) => {
                event!(Level::INFO, "pnl_plant: Received close frame: {:?}", frame);
                stop = true;
            }
            Ok(Message::Binary(data)) => match self.rithmic_receiver_api.buf_to_message(data) {
                Ok(response) => {
                    if response.is_update {
                        match self.subscription_sender.send(response) {
                            Ok(_) => {}
                            Err(e) => {
                                event!(Level::ERROR, "pnl_plant: failed to send response {:?}", e);
                            }
                        };
                    } else {
                        self.request_handler.handle_response(response);
                    }
                }
                Err(err) => {
                    event!(
                        Level::ERROR,
                        "pnl_plant: received an error message {:?}",
                        err
                    );

                    self.request_handler.handle_response(err);
                }
            },
            Err(Error::ConnectionClosed) => {
                event!(Level::INFO, "pnl_plant: Connection closed");
                stop = true;
            }
            _ => {
                event!(Level::WARN, "pnl_plant: Unhandled message: {:?}", message);
            }
        }

        Ok(stop)
    }

    async fn handle_command(&mut self, command: PnlPlantCommand) {
        match command {
            PnlPlantCommand::Close => {
                self.rithmic_sender
                    .send(Message::Close(None))
                    .await
                    .unwrap();
            }
            PnlPlantCommand::Login { response_sender } => {
                let (login_buf, id) = self.rithmic_sender_api.request_login(
                    &self.config.system_name,
                    SysInfraType::PnlPlant,
                    &self.config.user,
                    &self.config.password,
                );

                event!(Level::INFO, "pnl_plant: sending login request {}", id);

                self.request_handler.register_request(RithmicRequest {
                    request_id: id,
                    responder: response_sender,
                });

                self.rithmic_sender
                    .send(Message::Binary(login_buf))
                    .await
                    .unwrap();
            }
            PnlPlantCommand::SetLogin => {
                self.logged_in = true;
            }
            PnlPlantCommand::Logout { response_sender } => {
                let (logout_buf, id) = self.rithmic_sender_api.request_logout();

                self.request_handler.register_request(RithmicRequest {
                    request_id: id,
                    responder: response_sender,
                });

                self.rithmic_sender
                    .send(Message::Binary(logout_buf))
                    .await
                    .unwrap();
            }
            PnlPlantCommand::SendHeartbeat {} => {
                let (heartbeat_buf, _id) = self.rithmic_sender_api.request_heartbeat();

                let _ = self
                    .rithmic_sender
                    .send(Message::Binary(heartbeat_buf))
                    .await;
            }
            PnlPlantCommand::SubscribePnlUpdates { response_sender } => {
                let (subscribe_buf, id) = self.rithmic_sender_api.request_pnl_position_updates(
                    request_pn_l_position_updates::Request::Subscribe,
                );

                self.request_handler.register_request(RithmicRequest {
                    request_id: id,
                    responder: response_sender,
                });

                self.rithmic_sender
                    .send(Message::Binary(subscribe_buf))
                    .await
                    .unwrap();
            }
            PnlPlantCommand::PnlPositionSnapshots { response_sender } => {
                let (snapshot_buf, id) = self.rithmic_sender_api.request_pnl_position_snapshot();

                self.request_handler.register_request(RithmicRequest {
                    request_id: id,
                    responder: response_sender,
                });

                self.rithmic_sender
                    .send(Message::Binary(snapshot_buf))
                    .await
                    .unwrap();
            }
        }
    }
}

pub struct RithmicPnlPlantHandle {
    sender: mpsc::Sender<PnlPlantCommand>,
    pub subscription_receiver: broadcast::Receiver<RithmicResponse>,
}

impl RithmicPnlPlantHandle {
    pub async fn login(&self) -> Result<RithmicResponse, String> {
        event!(Level::INFO, "pnl_plant: logging in");

        let (tx, rx) = oneshot::channel::<Result<Vec<RithmicResponse>, String>>();

        let command = PnlPlantCommand::Login {
            response_sender: tx,
        };

        let _ = self.sender.send(command).await;
        let response = rx.await.unwrap().unwrap().remove(0);

        if response.error.is_none() {
            let _ = self.sender.send(PnlPlantCommand::SetLogin).await;

            event!(Level::INFO, "pnl_plant: logged in");

            Ok(response)
        } else {
            event!(Level::ERROR, "pnl_plant: login failed {:?}", response.error);

            Err(response.error.unwrap())
        }
    }

    pub async fn disconnect(&self) -> Result<RithmicResponse, String> {
        let (tx, rx) = oneshot::channel::<Result<Vec<RithmicResponse>, String>>();

        let command = PnlPlantCommand::Logout {
            response_sender: tx,
        };

        let _ = self.sender.send(command).await;
        let mut r = rx.await.unwrap().unwrap();
        let _ = self.sender.send(PnlPlantCommand::Close).await;

        Ok(r.remove(0))
    }

    pub async fn subscribe_pnl_updates(&self) -> Result<RithmicResponse, String> {
        let (tx, rx) = oneshot::channel::<Result<Vec<RithmicResponse>, String>>();

        let command = PnlPlantCommand::SubscribePnlUpdates {
            response_sender: tx,
        };

        let _ = self.sender.send(command).await;

        Ok(rx.await.unwrap().unwrap().remove(0))
    }

    pub async fn pnl_position_snapshots(&self) -> Result<RithmicResponse, String> {
        let (tx, rx) = oneshot::channel::<Result<Vec<RithmicResponse>, String>>();

        let command = PnlPlantCommand::PnlPositionSnapshots {
            response_sender: tx,
        };

        let _ = self.sender.send(command).await;

        let response = rx.await;

        if let Ok(r) = response {
            if let Ok(mut v) = r {
                Ok(v.remove(0))
            } else {
                Err("error".to_string())
            }
        } else {
            Err("error".to_string())
        }
    }
}
