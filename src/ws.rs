use std::env;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    client_async,
    tungstenite::{Message},
    WebSocketStream,
};
use tokio_native_tls::{TlsConnector, TlsStream};
use native_tls::TlsConnector as NativeTlsConnector;
use tracing::{debug, info, error};
use url::Url;
use bytes::Bytes;

pub type RithmicStream = WebSocketStream<TlsStream<TcpStream>>;

/// Connect to a Rithmic WebSocket endpoint with SSL/TLS, supporting system proxies.
pub async fn connect(url: &str) -> Result<RithmicStream, anyhow::Error> {
    let target_url = Url::parse(url)?;
    let host = target_url.host_str().ok_or_else(|| anyhow::anyhow!("No host in URL"))?;
    let port = target_url.port_or_known_default().ok_or_else(|| anyhow::anyhow!("No port in URL"))?;
    
    // 1. Detect Proxy
    let proxy_url = get_proxy_url();
    
    let tcp_stream = if let Some(proxy) = proxy_url {
        info!("Using Proxy: {}", proxy);
        connect_via_proxy(&proxy, host, port).await?
    } else {
        info!("Connecting directly to {}:{}", host, port);
        TcpStream::connect((host, port)).await?
    };

    // 2. TLS Handshake (Native Certs)
    // NativeTlsConnector::new() uses OS trust store by default (SChannel on Windows)
    let cx = NativeTlsConnector::new()?;
    let cx = TlsConnector::from(cx);

    debug!("Starting TLS Handshake with {}", host);
    // We must provide the domain name for SNI
    let tls_stream = cx.connect(host, tcp_stream).await.map_err(|e| {
        error!("TLS Handshake failed: {}", e);
        anyhow::anyhow!("TLS Handshake failed: {}", e)
    })?;

    // 3. WebSocket Handshake
    debug!("Starting WebSocket Handshake");
    let (ws_stream, _) = client_async(url, tls_stream).await.map_err(|e| {
        error!("WebSocket Handshake failed: {}", e);
        anyhow::anyhow!("WebSocket Handshake failed: {}", e)
    })?;

    info!("Connected to Rithmic WS: {}", url);
    Ok(ws_stream)
}

/// Helper to send bytes
pub async fn send_bytes(stream: &mut RithmicStream, data: Vec<u8>) -> Result<(), anyhow::Error> {
    stream.send(Message::Binary(Bytes::from(data))).await.map_err(|e| e.into())
}

/// Helper to receive bytes
pub async fn receive_bytes(stream: &mut RithmicStream) -> Result<Option<Vec<u8>>, anyhow::Error> {
    match stream.next().await {
        Some(Ok(Message::Binary(data))) => Ok(Some(data.to_vec())),
        Some(Ok(Message::Ping(_))) => {
            debug!("Received Ping");
            Ok(None)
        }
        Some(Ok(Message::Pong(_))) => {
            debug!("Received Pong");
            Ok(None)
        }
        Some(Ok(Message::Close(_))) => {
            info!("Connection closed by server");
            Ok(None) // End of stream
        }
        Some(Err(e)) => Err(e.into()),
        None => Ok(None), // Stream ended
        _ => Ok(None), // Text messages or others we ignore
    }
}

// --- Proxy Logic ---

fn get_proxy_url() -> Option<Url> {
    // Check common env vars
    let vars = ["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"];
    
    for var in vars {
        if let Ok(val) = env::var(var) {
            if !val.is_empty() {
                return Url::parse(&val).ok();
            }
        }
    }
    None
}

async fn connect_via_proxy(proxy_url: &Url, target_host: &str, target_port: u16) -> Result<TcpStream, anyhow::Error> {
    let proxy_host = proxy_url.host_str().ok_or_else(|| anyhow::anyhow!("Invalid proxy host"))?;
    let proxy_port = proxy_url.port_or_known_default().unwrap_or(8080);

    debug!("Connecting to proxy at {}:{}", proxy_host, proxy_port);
    let stream = TcpStream::connect((proxy_host, proxy_port)).await?;
    
    // Use BufReader to read line by line
    let mut reader = BufReader::new(stream);

    // Build CONNECT request
    let mut connect_req = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n",
        target_host, target_port, target_host, target_port
    );

    // Proxy Auth
    if !proxy_url.username().is_empty() {
        let auth = format!("{}:{}", proxy_url.username(), proxy_url.password().unwrap_or(""));
        use base64::{Engine as _, engine::general_purpose};
        let encoded = general_purpose::STANDARD.encode(auth);
        connect_req.push_str(&format!("Proxy-Authorization: Basic {}\
\n", encoded));
    }

    connect_req.push_str("\r\n");

    debug!("Sending CONNECT request to proxy");
    reader.write_all(connect_req.as_bytes()).await?;
    reader.flush().await?;

    // Read Response Headers
    let mut line = String::new();
    
    // Read Status Line
    reader.read_line(&mut line).await?;
    if !line.starts_with("HTTP/1.1 200") && !line.starts_with("HTTP/1.0 200") {
        return Err(anyhow::anyhow!("Proxy handshake failed: {}", line.trim()));
    }
    
    // Read until empty line
    loop {
        line.clear();
        reader.read_line(&mut line).await?;
        if line == "\r\n" || line == "\n" {
            break;
        }
        if line.is_empty() {
            // EOF before end of headers
            return Err(anyhow::anyhow!("Proxy closed connection during handshake"));
        }
    }

    debug!("Proxy tunnel established");
    // Unwrap the stream from BufReader to return raw TcpStream
    Ok(reader.into_inner())
}