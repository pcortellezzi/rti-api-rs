use async_trait::async_trait;
use std::time::Duration;
use tracing::{error, info, warn};
use std::env;
use anyhow::anyhow;

use tokio::{
    net::TcpStream,
    time::{Instant, Interval, interval_at, sleep, timeout},
};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use bytes::Bytes;
use http::{Request, Uri};
use http::header::PROXY_AUTHORIZATION;



use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async_with_config,
    tungstenite::{
        Error, Message,
        client::IntoClientRequest,
    },
};

/// Number of seconds between heartbeats sent to the server.
pub const HEARTBEAT_SECS: u64 = 60;

/// Connection attempt timeout in seconds.
const CONNECT_TIMEOUT_SECS: u64 = 2;

/// Base backoff in milliseconds multiplied by the attempt number.
const BACKOFF_MS_BASE: u64 = 500;

/// A generic stream over the Rithmic connection exposing a handle for external control.
pub trait RithmicStream {
    type Handle;

    fn get_handle(&self) -> Self::Handle;
}

#[async_trait]
pub trait PlantActor {
    type Command;

    async fn run(&mut self);
    async fn handle_command(&mut self, command: Self::Command);
    async fn handle_rithmic_message(&mut self, message: Result<Message, Error>)
    -> Result<bool, ()>;
}

pub fn get_heartbeat_interval(override_secs: Option<u64>) -> Interval {
    let secs = override_secs.unwrap_or(HEARTBEAT_SECS);
    let heartbeat_interval = Duration::from_secs(secs);
    let start_offset = Instant::now() + heartbeat_interval;

    interval_at(start_offset, heartbeat_interval)
}

/// Sometimes the connection gets stuck and retrying seems to help.
///
/// Arguments:
/// * `primary_url`: Primary URL to connect to first
/// * `secondary_url`: Beta URL to alternate with after the first failure
/// * `max_attempts`: Total number of attempts to connect
///
/// Returns:
/// * `Ok`: A WebSocketStream if the connection is successful.
/// * `Err`: An error if the connection fails after the specified number of attempts.
pub async fn connect_with_retry(
    primary_url: &str,
    secondary_url: &str,
    max_attempts: u32,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Error> {
    for attempt in 1..=max_attempts {
        let selected_url = if attempt == 1 {
            primary_url
        } else if attempt % 2 == 0 {
            secondary_url
        } else {
            primary_url
        };

        info!("Attempt {}: connecting to {}", attempt, selected_url);

        match timeout(
            Duration::from_secs(CONNECT_TIMEOUT_SECS),
            connect_async_with_config(selected_url, None, true),
        )
        .await
        {
            Ok(Ok((ws_stream, _))) => return Ok(ws_stream),
            Ok(Err(e)) => warn!("connect_async failed for {}: {:?}", selected_url, e),
            Err(e) => warn!("connect_async to {} timed out: {:?}", selected_url, e),
        }

        if attempt < max_attempts {
            let backoff_ms: u64 = BACKOFF_MS_BASE * attempt as u64;

            info!("Backing off for {}ms before retry", backoff_ms,);

            sleep(Duration::from_millis(backoff_ms)).await;
        }
    }

    error!("max connection attempts reached");

    Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "max connection attempts reached",
    )))
}

pub async fn connect(url: &str) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, anyhow::Error> {
    let ws_uri: Uri = url.parse()?;

    if let Ok(proxy_url_str) = env::var("HTTPS_PROXY") {
        let proxy_uri: hyper::Uri = proxy_url_str.parse()?;

        // Établir une connexion TCP avec le proxy
        let stream = hyper_util::rt::TokioIo::new(TcpStream::connect(format!("{}:{}", proxy_uri.host().unwrap_or_default(), proxy_uri.port_u16().unwrap_or(80))).await?);

        let (mut request_sender, conn) = hyper::client::conn::http1::handshake(stream).await?;
        let conn = tokio::spawn(conn.without_shutdown());

        let mut request_builder = Request::connect(format!("{}:{}", ws_uri.host().unwrap_or_default(), ws_uri.port_u16().unwrap_or(443)));
        // Ajoute l'authentification si présente dans l'URL du proxy
        if let Some(auth) = proxy_uri.authority() {
            if let Some((username, password)) = auth.as_str().split_once('@') {
                let credentials = format!("{}:{}", username, password.splitn(2, ':').next().unwrap_or(""));
                let auth = format!("Basic {}", BASE64_STANDARD.encode(credentials));
                request_builder = request_builder.header(PROXY_AUTHORIZATION, auth);
            }
        }
        let request = request_builder.body(http_body_util::Empty::<Bytes>::new())?;

        let res = request_sender.send_request(request).await?;

        if !res.status().is_success() {
            return Err(anyhow!(
                "The proxy server returned an error response: status code: {}, body: {:#?}",
                res.status(),
                res.body()
            ));
        }

        let tcp = conn.await
            .map_err(|e| anyhow!(e))??
            .io
            .into_inner();

        // CryptoProvider::install_default();
        let ws_stream = tokio_tungstenite::client_async_tls(ws_uri.into_client_request()?, tcp).await?.0;
        Ok(ws_stream)
    } else {
        let ws_stream = tokio_tungstenite::connect_async(ws_uri.into_client_request()?).await?.0;
        Ok(ws_stream)
    }
}