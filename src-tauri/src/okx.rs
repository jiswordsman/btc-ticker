use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

const OKX_WS_URL: &str = "wss://ws.okx.com:8443/ws/v5/public";
const OKX_FUNDING_RATE_URL: &str = "https://www.okx.com/api/v5/public/funding-rate";
const INST_ID: &str = "BTC-USDT-SWAP";

// Global proxy address
static PROXY_ADDRESS: std::sync::OnceLock<Arc<tokio::sync::RwLock<String>>> = std::sync::OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TickerData {
    pub last: f64,
    pub open24h: f64,
    pub high24h: f64,
    pub low24h: f64,
    pub vol24h: f64,
    #[serde(rename = "volCcy24h")]
    pub vol_ccy24h: f64,
    pub funding_rate: f64,
    pub change: f64,
    pub change_percent: f64,
    pub ts: u64,
}

#[derive(Debug, Deserialize)]
struct WsResponse {
    #[serde(default)]
    data: Option<Vec<RawTicker>>,
    #[serde(default)]
    event: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTicker {
    last: String,
    open24h: String,
    high24h: String,
    low24h: String,
    vol24h: String,
    volCcy24h: String,
    ts: String,
}

#[derive(Debug, Deserialize)]
struct FundingRateResponse {
    #[serde(default)]
    data: Vec<RawFundingRate>,
}

#[derive(Debug, Deserialize)]
struct RawFundingRate {
    #[serde(rename = "fundingRate")]
    funding_rate: String,
}

impl RawTicker {
    fn to_ticker_data(&self, funding_rate: f64) -> Option<TickerData> {
        let last = self.last.parse::<f64>().ok()?;
        let open24h = self.open24h.parse::<f64>().ok()?;
        let high24h = self.high24h.parse::<f64>().ok()?;
        let low24h = self.low24h.parse::<f64>().ok()?;
        let vol24h = self.vol24h.parse::<f64>().ok()?;
        let vol_ccy24h = self.volCcy24h.parse::<f64>().ok()?;
        let ts = self.ts.parse::<u64>().ok()?;

        let change = last - open24h;
        let change_percent = if open24h != 0.0 {
            (change / open24h) * 100.0
        } else {
            0.0
        };

        Some(TickerData {
            last,
            open24h,
            high24h,
            low24h,
            vol24h,
            vol_ccy24h,
            funding_rate,
            change,
            change_percent,
            ts,
        })
    }
}

pub fn start_ws_client(tx: watch::Sender<TickerData>) {
    let funding_rate = Arc::new(tokio::sync::RwLock::new(0.0));

    // Proxy storage should already be initialized in lib.rs
    
    start_funding_rate_poll(tx.clone(), funding_rate.clone());

    tauri::async_runtime::spawn(async move {
        loop {
            info!("Connecting to OKX WebSocket...");
            match connect_and_subscribe(tx.clone(), funding_rate.clone()).await {
                Ok(_) => {
                    warn!("WebSocket connection closed, reconnecting in 5s...");
                }
                Err(e) => {
                    error!("WebSocket error: {}, reconnecting in 5s...", e);
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
    });
}

fn start_funding_rate_poll(
    tx: watch::Sender<TickerData>,
    funding_rate: Arc<tokio::sync::RwLock<f64>>,
) {
    tauri::async_runtime::spawn(async move {
        // Build HTTP client with optional proxy
        let client = build_http_client().await;

        loop {
            match fetch_funding_rate(&client).await {
                Some(rate) => {
                    {
                        let mut latest_rate = funding_rate.write().await;
                        *latest_rate = rate;
                    }

                    let mut snapshot = tx.borrow().clone();
                    if snapshot.last > 0.0 {
                        snapshot.funding_rate = rate;
                        let _ = tx.send(snapshot);
                    }
                }
                None => warn!("Failed to refresh funding rate from OKX"),
            }

            sleep(Duration::from_secs(60)).await;
        }
    });
}

async fn connect_and_subscribe(
    tx: watch::Sender<TickerData>,
    funding_rate: Arc<tokio::sync::RwLock<f64>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get current proxy address (just like reading Java's proxyInstance)
    let proxy_addr = if let Some(proxy_lock) = PROXY_ADDRESS.get() {
        proxy_lock.read().await.clone()
    } else {
        String::new()
    };

    // Connect with or without proxy based on configuration
    let (ws_stream, _) = if proxy_addr.is_empty() {
        // No proxy configured - direct connection
        info!("Connecting to OKX WebSocket (direct)...");
        connect_async(OKX_WS_URL).await?
    } else {
        // Proxy configured - connect through proxy
        info!("Connecting to OKX WebSocket via proxy: {}...", proxy_addr);
        connect_async_with_proxy(OKX_WS_URL, &proxy_addr).await?
    };

    info!("WebSocket connected to OKX");

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to BTC-USDT-SWAP tickers
    let subscribe_msg = serde_json::json!({
        "op": "subscribe",
        "args": [{
            "channel": "tickers",
            "instId": INST_ID
        }]
    });
    write
        .send(Message::Text(subscribe_msg.to_string().into()))
        .await?;
    info!("Subscribed to {} tickers", INST_ID);

    // Spawn ping task
    let ping_write = Arc::new(tokio::sync::Mutex::new(write));
    let ping_writer = ping_write.clone();
    let ping_task = tauri::async_runtime::spawn(async move {
        loop {
            sleep(Duration::from_secs(25)).await;
            let mut w = ping_writer.lock().await;
            if w.send(Message::Text("ping".into())).await.is_err() {
                break;
            }
        }
    });

    // Read messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Skip pong responses
                if text.as_str() == "pong" {
                    continue;
                }

                if let Ok(response) = serde_json::from_str::<WsResponse>(&text) {
                    // Handle subscription confirmation
                    if let Some(event) = &response.event {
                        info!("WS event: {}", event);
                        continue;
                    }

                    // Handle ticker data
                    if let Some(data) = response.data {
                        if let Some(raw) = data.first() {
                            let latest_funding_rate = *funding_rate.read().await;
                            if let Some(ticker) = raw.to_ticker_data(latest_funding_rate) {
                                let _ = tx.send(ticker);
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket closed by server");
                break;
            }
            Err(e) => {
                error!("WebSocket read error: {}", e);
                break;
            }
            _ => {}
        }
    }

    ping_task.abort();
    Ok(())
}

async fn fetch_funding_rate(client: &reqwest::Client) -> Option<f64> {
    let response = client
        .get(OKX_FUNDING_RATE_URL)
        .query(&[("instId", INST_ID)])
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        warn!("Funding rate request failed with status {}", response.status());
        return None;
    }

    let payload = response.json::<FundingRateResponse>().await.ok()?;
    let raw_rate = payload.data.first()?.funding_rate.parse::<f64>().ok()?;

    Some(raw_rate)
}

/// Build HTTP client with optional proxy
async fn build_http_client() -> reqwest::Client {
    let proxy_addr = if let Some(proxy_lock) = PROXY_ADDRESS.get() {
        proxy_lock.read().await.clone()
    } else {
        String::new()
    };

    let mut builder = reqwest::Client::builder()
        .user_agent("btc-ticker/0.1");

    if !proxy_addr.is_empty() {
        // Configure proxy like Java's Proxy.Type.HTTP
        let proxy_url = format!("http://{}", proxy_addr);
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy);
            info!("HTTP client configured with proxy: {}", proxy_addr);
        }
    }

    builder.build().expect("failed to build HTTP client")
}

/// Connect to WebSocket via HTTP proxy (like Java's URL.openConnection(proxy))
async fn connect_async_with_proxy(
    url: &str,
    proxy_addr: &str,
) -> Result<
    (
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::handshake::client::Response,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use url::Url;

    // Parse proxy address (format: host:port, e.g., 127.0.0.1:7890)
    let proxy_parts: Vec<&str> = proxy_addr.split(':').collect();
    if proxy_parts.len() != 2 {
        return Err(format!("Invalid proxy format: {}. Expected: host:port", proxy_addr).into());
    }

    let proxy_host = proxy_parts[0];
    let proxy_port: u16 = proxy_parts[1]
        .parse()
        .map_err(|e| format!("Invalid proxy port: {}", e))?;

    // Parse WebSocket URL to get target host
    let ws_url = Url::parse(url)?;
    let target_host = ws_url.host_str().ok_or("No host in URL")?;
    let target_port = ws_url.port_or_known_default().ok_or("No port in URL")?;

    // Step 1: Connect to proxy server (like Java's socket connection to proxy)
    info!("Connecting to proxy server: {}:{}", proxy_host, proxy_port);
    let mut stream = TcpStream::connect((proxy_host, proxy_port)).await?;

    // Step 2: Send HTTP CONNECT request (establish tunnel)
    let connect_request = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
        target_host, target_port, target_host, target_port
    );
    
    info!("Sending CONNECT request to establish tunnel...");
    stream.write_all(connect_request.as_bytes()).await?;

    // Step 3: Read CONNECT response
    let mut response_buf = [0u8; 1024];
    let n = stream.read(&mut response_buf).await?;
    let response_str = String::from_utf8_lossy(&response_buf[..n]);
    
    if !response_str.contains("200") {
        return Err(format!("Proxy CONNECT failed: {}", response_str).into());
    }
    
    info!("Proxy tunnel established successfully");

    // Step 4: Perform WebSocket handshake through the tunnel
    // This is like Java's WebSocket handshake over the proxied connection
    let request = url.into_client_request()?;
    let (ws_stream, response) = tokio_tungstenite::client_async_tls(request, stream).await?;

    Ok((ws_stream, response))
}

/// Initialize proxy storage (call this before set_proxy_address)
pub fn init_proxy_storage() {
    PROXY_ADDRESS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(String::new())));
    info!("Proxy storage initialized");
}

/// Set proxy address (like Java's Proxy initialization)
pub fn set_proxy_address(proxy_addr: String) {
    if let Some(proxy_lock) = PROXY_ADDRESS.get() {
        let mut current_proxy = proxy_lock.blocking_write();
        *current_proxy = proxy_addr.clone();
        info!("Proxy address set to: {} (like Java Proxy.Type.HTTP)", proxy_addr);
    } else {
        warn!("Proxy storage not initialized");
    }
}
