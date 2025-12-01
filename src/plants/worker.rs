use std::collections::HashMap;
use prost::Message; // Trait required for encode
use tokio::sync::{mpsc, oneshot};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

use crate::api::decoder::{decode_message, RithmicResponse};
use crate::ws::{connect, receive_bytes, send_bytes};
use crate::rti::{RequestHeartbeat, messages::RithmicMessage, ResponseLogin};

/// Command sent to the worker
pub struct WorkerCommand {
    pub payload: Vec<u8>,
    pub request_id: String,
    // For single, immediate response (e.g., login, submit order ACK)
    pub reply_tx: Option<oneshot::Sender<Result<RithmicResponse, String>>>,
    // For streaming/multi-part responses (e.g., historical data, account lists)
    pub stream_tx: Option<mpsc::Sender<Result<RithmicResponse, String>>>, 
}

/// Main loop for a plant connection
pub async fn start_plant_worker(
    url: String,
    login_req: (Vec<u8>, String), // (payload, request_id)
    mut command_rx: mpsc::Receiver<WorkerCommand>,
    event_tx: mpsc::Sender<RithmicResponse>,
    login_result_tx: oneshot::Sender<Result<ResponseLogin, String>>,
) -> Result<(), anyhow::Error> {
    
    info!("Starting worker for {}", url);
    let mut login_result_tx = Some(login_result_tx);
    
    // 1. Connect
    let mut stream = match connect(&url).await {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("Failed to connect to {}: {}", url, e);
            error!("{}", msg);
            if let Some(tx) = login_result_tx.take() {
                let _ = tx.send(Err(msg));
            }
            return Err(e);
        }
    };
    
    // 2. Login
    let (login_payload, login_id) = login_req;
    // DEBUG: Hex dump login payload
    let hex_string: String = login_payload.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    debug!("Login Payload ({} bytes): {}", login_payload.len(), hex_string);

    if let Err(e) = send_bytes(&mut stream, login_payload).await {
        let msg = format!("Failed to send login request: {}", e);
        error!("{}", msg);
        if let Some(tx) = login_result_tx.take() {
            let _ = tx.send(Err(msg));
        }
        return Err(e);
    }
    
    // Wait for login response
    let mut logged_in = false;
    let mut heartbeat_secs: f64 = 30.0; // Default fallback

    while !logged_in {
        match receive_bytes(&mut stream).await {
            Ok(Some(bytes)) => {
                 match decode_message(&bytes) {
                     Ok(resp) => {
                         if resp.request_id == login_id {
                             if let RithmicMessage::ResponseLogin(login_resp) = resp.message {
                                 // Extract heartbeat info
                                 if let Some(hb) = login_resp.heartbeat_interval {
                                     heartbeat_secs = hb;
                                     debug!("Heartbeat interval set to {}s", heartbeat_secs);
                                 }
                                 
                                 info!("Login successful for {}", url);
                                 // Notify success with full login response
                                 if let Some(tx) = login_result_tx.take() {
                                     let _ = tx.send(Ok(login_resp));
                                 }
                                 logged_in = true;
                             } else {
                                 if let Some(tx) = login_result_tx.take() {
                                     let _ = tx.send(Err("Unexpected response type during login".into()));
                                 }
                                 return Err(anyhow::anyhow!("Unexpected response type during login"));
                             }
                         } else {
                             // Ignore other messages during login or handle them?
                             debug!("Received message during login: {:?}", resp.message);
                         }
                     }
                     Err(e) => {
                         let err_msg = e.error.clone().unwrap_or_else(|| "Unknown login error".to_string());
                         error!("Login failed/rejected: {}", err_msg);
                         if let Some(tx) = login_result_tx.take() {
                             let _ = tx.send(Err(err_msg.clone()));
                         }
                         return Err(anyhow::anyhow!("Login failed: {}", err_msg));
                     }
                 }
            }
            Ok(None) => {
                let msg = "Connection closed by server during login";
                error!("{}", msg);
                if let Some(tx) = login_result_tx.take() {
                    let _ = tx.send(Err(msg.into()));
                }
                return Err(anyhow::anyhow!(msg));
            }
            Err(e) => {
                let msg = format!("Socket error during login: {}", e);
                error!("{}", msg);
                if let Some(tx) = login_result_tx.take() {
                    let _ = tx.send(Err(msg));
                }
                return Err(e);
            }
        }
    }
    
    // 3. Main Loop
    // Map to hold pending single-response commands
    let mut pending_replies: HashMap<String, oneshot::Sender<Result<RithmicResponse, String>>> = HashMap::new();
    // Map to hold pending multi-response (stream) commands
    let mut pending_streams: HashMap<String, mpsc::Sender<Result<RithmicResponse, String>>> = HashMap::new();

    let hb_period = if heartbeat_secs > 5.0 { heartbeat_secs - 2.0 } else { heartbeat_secs };
    let mut heartbeat_interval = interval(Duration::from_secs(hb_period as u64));
    
    loop {
        tokio::select! {
            // Heartbeat
            _ = heartbeat_interval.tick() => {
                let req = RequestHeartbeat {
                    template_id: 18,
                    user_msg: vec!["hb".to_string()],
                    ..RequestHeartbeat::default()
                };
                
                let mut buf = Vec::new();
                let len = req.encoded_len() as u32;
                let header = len.to_be_bytes();
                buf.reserve((len + 4) as usize);
                buf.extend_from_slice(&header);
                req.encode(&mut buf).unwrap();
                
                if let Err(e) = send_bytes(&mut stream, buf).await {
                     error!("Failed to send heartbeat: {}", e);
                     break;
                }
            }

            // Incoming Commands
            cmd_opt = command_rx.recv() => {
                match cmd_opt {
                    Some(cmd) => {
                        if let Some(reply_tx) = cmd.reply_tx {
                            pending_replies.insert(cmd.request_id.clone(), reply_tx);
                        }
                        if let Some(stream_tx) = cmd.stream_tx {
                            pending_streams.insert(cmd.request_id.clone(), stream_tx);
                        }

                        if let Err(e) = send_bytes(&mut stream, cmd.payload).await {
                            error!("Failed to send command: {}", e);
                            // Notify all pending channels for this request_id
                            if let Some(tx) = pending_replies.remove(&cmd.request_id) {
                                let _ = tx.send(Err(e.to_string()));
                            }
                            if let Some(tx) = pending_streams.remove(&cmd.request_id) {
                                let _ = tx.send(Err(e.to_string())).await;
                            }
                            break; 
                        }
                    }
                    None => {
                        info!("Command channel closed, stopping worker {}", url);
                        break;
                    }
                }
            }

            // Incoming WS Messages
            res = receive_bytes(&mut stream) => {
                match res {
                    Ok(Some(bytes)) => {
                        match decode_message(&bytes) {
                            Ok(resp) => { // resp is not mut anymore
                                // Handle Heartbeat Response (silent)
                                if let RithmicMessage::ResponseHeartbeat(_) = resp.message {
                                    continue;
                                }

                                // Check pending single-replies first (they take priority)
                                if let Some(tx) = pending_replies.remove(&resp.request_id) {
                                    let _ = tx.send(Ok(resp));
                                    // If has_more, it should go to stream_tx for multi-response
                                    // This means single-reply should ONLY be for non-multi-response.
                                    // Or the first message of a multi-response.
                                    // For simplicity: `reply_tx` is for non-multi. `stream_tx` is for multi.
                                    // This means `WorkerCommand` needs to be more explicit.
                                    // For now, if a reply_tx is present, it consumes the first message.
                                    // If it's a stream, it needs `stream_tx`.

                                } else if let Some(tx) = pending_streams.get_mut(&resp.request_id) {
                                    // Send to stream channel
                                    if let Err(e) = tx.send(Ok(resp.clone())).await {
                                        error!("Failed to send to stream_tx: {}", e);
                                        pending_streams.remove(&resp.request_id); // Drop if channel broken
                                    }
                                    // If stream is finished, remove it
                                    if !resp.has_more {
                                        pending_streams.remove(&resp.request_id);
                                    }

                                } else {
                                    // Unsolicited or no specific handler, send to global event_tx
                                    if let Err(e) = event_tx.send(resp).await {
                                        error!("Event channel closed: {}", e);
                                        break;
                                    }
                                }
                            }
                            Err(resp_err) => {
                                // Error response can also be for pending replies or streams
                                if let Some(tx) = pending_replies.remove(&resp_err.request_id) {
                                    let _ = tx.send(Err(resp_err.error.clone().unwrap_or("Unknown error".into())));
                                }
                                if let Some(tx) = pending_streams.get_mut(&resp_err.request_id) {
                                     let _ = tx.send(Err(resp_err.error.clone().unwrap_or("Unknown error".into()))).await;
                                     pending_streams.remove(&resp_err.request_id); // Error terminates stream
                                } else {
                                    error!("Received error for unknown/expired request {}: {:?}", resp_err.request_id, resp_err.error);
                                    // If it's a global error, send to event_tx? No, if no specific listener, just log.
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        info!("Connection closed by server in main loop");
                        break;
                    }
                    Err(e) => {
                        error!("WS Error: {}", e);
                        break;
                    }
                }
            }
        }
    }
    
    info!("Worker {} stopped", url);
    Ok(())
}
