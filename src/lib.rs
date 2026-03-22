mod security;

use std::collections::HashMap;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub type EventData = HashMap<String, serde_json::Value>;
pub type EventHandler = Box<dyn Fn(EventData) + Send + Sync>;

pub struct TikSync {
    unique_id: String,
    api_key: String,
    api_url: String,
    handlers: HashMap<String, Vec<EventHandler>>,
    security: security::SecurityCore,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawEvent {
    #[serde(rename = "type")]
    event_type: Option<String>,
    data: Option<serde_json::Value>,
}

impl TikSync {
    pub fn new(unique_id: &str, api_key: &str) -> Self {
        Self {
            unique_id: unique_id.replace('@', ""),
            api_key: api_key.to_string(),
            api_url: "wss://api.tiksync.com".to_string(),
            handlers: HashMap::new(),
            security: security::SecurityCore::new(api_key),
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.api_url = url.replace("https://", "wss://").replace("http://", "ws://");
        self
    }

    pub fn on<F>(mut self, event: &str, handler: F) -> Self
    where
        F: Fn(EventData) + Send + Sync + 'static,
    {
        self.handlers
            .entry(event.to_string())
            .or_default()
            .push(Box::new(handler));
        self
    }

    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/v1/connect?uniqueId={}",
            self.api_url,
            urlencoding::encode(&self.unique_id)
        );

        let sec_headers = self.security.get_headers();
        let mut builder = http::Request::builder()
            .uri(&url)
            .header("x-api-key", &self.api_key)
            .header("Host", url::Url::parse(&url)?.host_str().unwrap_or(""))
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key());

        for (k, v) in &sec_headers {
            builder = builder.header(k.as_str(), v.as_str());
        }

        let request = builder.body(())?;

        let (ws, _) = connect_async(request).await?;
        let (mut _write, mut read) = ws.split();

        self.emit("connected", HashMap::new());

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(event) = serde_json::from_str::<RawEvent>(&text) {
                        let event_type = event.event_type.unwrap_or_default();
                        let data: EventData = match event.data {
                            Some(serde_json::Value::Object(map)) => {
                                map.into_iter().collect()
                            }
                            _ => HashMap::new(),
                        };
                        self.emit(&event_type, data);
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    let mut err = HashMap::new();
                    err.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                    self.emit("error", err);
                    break;
                }
                _ => {}
            }
        }

        self.emit("disconnected", HashMap::new());
        Ok(())
    }

    fn emit(&self, event: &str, data: EventData) {
        if let Some(handlers) = self.handlers.get(event) {
            for handler in handlers {
                handler(data.clone());
            }
        }
    }
}
