use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::WebSocketStream;
use async_tungstenite::tungstenite::Message;
use tokio::time::{Duration, Instant};

use std::sync::Arc;
use tokio::sync::RwLock;
use futures_util::stream::StreamExt;

use super::Error;

use super::structs::RosuValues;

/// A struct that connects to the Gosumemory websocket and keeps the `gosu_json` data
/// up-to-date.
pub struct Listener
{
    temp_rosu_value: Option<RosuValues>,
    rosu_value: Arc<RwLock<RosuValues>>,
    last_value_recieved: Instant,
    ws: WebSocketStream<ConnectStream>
}

impl Listener 
{
    pub async fn new(ip: url::Url, rosu_value: Arc<RwLock<RosuValues>>) -> Result<Self, async_tungstenite::tungstenite::Error> {
        let (ws, _response) = connect_async(ip).await?;
        Ok(Self { 
            temp_rosu_value: None,
            rosu_value,
            last_value_recieved: Instant::now(),
            ws,
        })
    }

    pub async fn listen(mut self) -> Result<(), Error> {
        loop {
            if let Ok(Some(message)) = tokio::time::timeout(Duration::from_millis(500), self.ws.next()).await {
                if let Message::Text(rosu_text) = message? { 
                    log::trace!("Recieved text from websocket.");
                    self.temp_rosu_value = Some(serde_json::from_str(&rosu_text)?);
                    self.last_value_recieved = Instant::now();
                }
            }

            if self.temp_rosu_value.is_some() {
                let rosu_value_write_result = self.rosu_value.try_write();

                match rosu_value_write_result {
                    Ok(mut rosu_value_write) => {
                        // afaik this shouldn't fail.
                        *rosu_value_write = self.temp_rosu_value.take().unwrap();
                    },
                    Err(_e) => () // at some point prob should check for poisoned or something
                }
            }

            let since_last_value = Instant::now().duration_since(self.last_value_recieved);
            log::trace!("{since_last_value:?}");
            if since_last_value >= Duration::from_secs(30) {
                log::error!("It's been {since_last_value:?} since we last recieved data from the rosu-memory websocket, assuming it timed out for some reason.");
                return Err(Error::TimedOut);
            }
        }
    }
}

