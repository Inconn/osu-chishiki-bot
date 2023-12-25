use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::WebSocketStream;
use async_tungstenite::tungstenite::Message;
use std::time::{Duration, SystemTime};

use std::sync::Arc;
use tokio::sync::RwLock;
use futures::stream::StreamExt;

use gosumemory_helper::Gosumemory;

/// A struct that connects to the Gosumemory websocket and keeps the `gosu_json` data
/// up-to-date.
pub struct Listener
{
    temp_gosu_json: Option<Box<Gosumemory>>,
    gosu_json: Arc<RwLock<Box<Gosumemory>>>,
    last_json_recieved: SystemTime,
    ws: WebSocketStream<ConnectStream>
}

impl Listener 
{
    pub async fn new(ip: url::Url, gosu_json: Arc<RwLock<Box<Gosumemory>>>) -> Result<Self, async_tungstenite::tungstenite::Error> {
        let (ws, _response) = connect_async(ip).await?;
        Ok(Self { 
            temp_gosu_json: None,
            gosu_json,
            last_json_recieved: SystemTime::now(),
            ws,
        })
    }

    pub async fn listen(mut self) -> Result<(), Error> {
        loop {
            if let Ok(Some(message)) = tokio::time::timeout(Duration::from_millis(500), self.ws.next()).await {
                if let Message::Text(gosu_text) = message? { 
                    log::trace!("Recieved text from websocket.");
                    self.temp_gosu_json = Some(serde_json::from_str(&gosu_text)?);
                    self.last_json_recieved = SystemTime::now();
                }
            }

            if let Some(gosu_json) = self.temp_gosu_json.take() {
                let gosu_json_write_result = self.gosu_json.try_write();

                match gosu_json_write_result {
                    Ok(mut gosu_json_write) => {
                        // afaik this shouldn't fail. hopefully
                        *gosu_json_write = gosu_json;
                    },
                    Err(_e) => {
                        // we failed to get a write lock, put value back
                        // there's gotta be a better way but i'm not figuring it out rn
                        self.temp_gosu_json = Some(gosu_json);
                    }
                }
            }

            log::trace!("{:?}", SystemTime::now().duration_since(self.last_json_recieved).unwrap());
            if SystemTime::now().duration_since(self.last_json_recieved).unwrap() >= Duration::from_secs(30) {
                log::error!("It's been 30 seconds since we last recieved data from the gosumemory websocket, assuming it timed out for some reason.");
                return Err(Error::TimedOut);
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Websocket(async_tungstenite::tungstenite::Error),
    Parse(serde_json::Error),
    TimedOut
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Websocket(e) => write!(f, "Gosumemory listener failed due to websocket error: {e}"),
            Self::Parse(e) => write!(f, "Gosumemory listener failed due to parsing error: {e}"),
            Self::TimedOut => write!(f, "Gosumemory websocket timed out")
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Websocket(ref e) => Some(e),
            Self::Parse(ref e) => Some(e),
            Self::TimedOut => None
        }
    }
}

impl From<async_tungstenite::tungstenite::Error> for Error {
    fn from(item: async_tungstenite::tungstenite::Error) -> Self {
        Self::Websocket(item)
    }
}

impl From<serde_json::Error> for Error {
    fn from(item: serde_json::Error) -> Self {
        Self::Parse(item)
    }
}
