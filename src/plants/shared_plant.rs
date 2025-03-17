use anyhow::anyhow;
use tracing::{event, Level};

use crate::{api::{
    receiver_api::{RithmicReceiverApi, RithmicResponse},
    sender_api::RithmicSenderApi,
}, connection_info, connection_info::RithmicConnectionInfo, request_handler::{RithmicRequest, RithmicRequestHandler}, ws::{get_heartbeat_interval, PlantActor, RithmicStream, connect}};

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};

use tokio_tungstenite::{
    tungstenite::{Error, Message},
    MaybeTlsStream,
    WebSocketStream,
};

use tokio::{
    net::TcpStream,
    sync::{broadcast::Sender, oneshot},
    time::Interval,
};

use crate::{
    connection_info::{AccountInfo, RithmicConnectionSystem},
    rti::{
        ResponseRithmicSystemGatewayInfo,
        ResponseRithmicSystemInfo,
        messages::RithmicMessage,
    },
    ws::connect_with_retry,
};

pub enum SharedPlantCommand {
    RithmicSystemInfo,
    RithmicSystemGatewayInfo {
        system_name: String
    },
}

pub struct RithmicSharedPlant {
    rithmic_sender_api: RithmicSenderApi,
    rithmic_receiver_api: RithmicReceiverApi,
}

impl RithmicSharedPlant {
    pub fn new() -> RithmicSharedPlant {
        let rithmic_sender_api = RithmicSenderApi::new(&AccountInfo::default());
        let rithmic_receiver_api = RithmicReceiverApi {
            source: "shared_plant".to_string(),
        };

        RithmicSharedPlant {
            rithmic_sender_api,
            rithmic_receiver_api
        }
    }

    pub async fn rithmic_system_info(&mut self) -> Result<ResponseRithmicSystemInfo, anyhow::Error> {
        let config = connection_info::get_config(&RithmicConnectionSystem::Live);

        let ws_stream = connect_with_retry(&config.url, &config.beta_url, 15)
            .await
            .expect("failed to connect to order plant");

        let (rithmic_sender, mut rithmic_reader) = ws_stream.split();

        let command = SharedPlantCommand::RithmicSystemInfo {};
        self.handle_command(rithmic_sender, command).await;
        if let Some(message) = rithmic_reader.next().await {
            if let Ok(Message::Binary(data)) = message {
                if let RithmicMessage::ResponseRithmicSystemInfo(msg) = self.rithmic_receiver_api.buf_to_message(data).unwrap().message {
                    Ok(msg)
                } else {
                    Err(anyhow!("message is not a rithmic system info"))
                }
            } else {
                Err(anyhow!("rithmic message error"))
            }
        } else {
            Err(anyhow!("rithmic message error"))
        }
    }

    pub async fn rithmic_system_gateway_info(&mut self, system_name: String
    ) -> Result<ResponseRithmicSystemGatewayInfo, anyhow::Error> {
        let config = connection_info::get_config(&RithmicConnectionSystem::Live);

        let ws_stream = connect_with_retry(&config.url, &config.beta_url, 15)
            .await
            .expect("failed to connect to order plant");

        let (rithmic_sender, mut rithmic_reader) = ws_stream.split();

        let command = SharedPlantCommand::RithmicSystemGatewayInfo {
            system_name,
        };
        self.handle_command(rithmic_sender, command).await;
        if let Some(message) = rithmic_reader.next().await {
            if let Ok(Message::Binary(data)) = message {
                if let RithmicMessage::ResponseRithmicSystemGatewayInfo(msg) = self.rithmic_receiver_api.buf_to_message(data).unwrap().message {
                    Ok(msg)
                } else {
                    Err(anyhow!("message is not a rithmic system gateway info"))
                }
            } else {
                Err(anyhow!("rithmic message error"))
            }
        } else {
            Err(anyhow!("rithmic message error"))
        }
    }

    async fn handle_command(
        &mut self,
        mut rithmic_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message,>,
        command: SharedPlantCommand) {
        match command {
            SharedPlantCommand::RithmicSystemInfo => {
                let (request_buf, id) = self.rithmic_sender_api.request_rithmic_system_info();

                event!(Level::INFO, "shared_plant: sending system info request {}", id);

                rithmic_sender
                    .send(Message::Binary(request_buf.into()))
                    .await
                    .unwrap();
            }
            SharedPlantCommand::RithmicSystemGatewayInfo { system_name } => {
                let (request_buf, id) = self.rithmic_sender_api.request_rithmic_system_gateway_info(
                    system_name
                );

                event!(Level::INFO, "shared_plant: sending system gateway info request {}", id);

                rithmic_sender
                    .send(Message::Binary(request_buf.into()))
                    .await
                    .unwrap();
            }
        }
    }
}