use async_tungstenite::tokio::{connect_async, ConnectStream};
use async_tungstenite::WebSocketStream;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

use serde_json::Value;
use gosumemory_helper::Gosumemory;
use serde::Deserialize;

/// A simple struct that connects to the Gosumemory websocket and keeps the gosu_json data
/// up-to-date.
pub struct GosuListener
{
    temp_gosu_json: Value,
    gosu_json: Arc<RwLock<Gosumemory>>,
    ws: WebSocketStream<ConnectStream>
}

impl GosuListener 
{
    pub async fn new(ip: url::Url, gosu_json: Arc<RwLock<Gosumemory>>) -> Result<GosuListener, async_tungstenite::tungstenite::Error> {
        let (ws, _response) = connect_async(ip).await?;
        Ok(GosuListener { temp_gosu_json: Value::Null, ws, gosu_json })
    }

    pub async fn listen(mut self) {
        loop {
            if let Some(Ok(gosu_text)) = self.ws.next().await {
                self.temp_gosu_json = serde_json::from_str(gosu_text.to_text().unwrap()).unwrap();
            }

            if !self.temp_gosu_json.is_null() {
                let gosu_json_write_result = self.gosu_json.try_write();

                match gosu_json_write_result {
                    Ok(mut gosu_json_write) => {
                        // afaik this shouldn't fail. hopefully
                        *gosu_json_write = Gosumemory::deserialize(self.temp_gosu_json).unwrap();
                        self.temp_gosu_json = Value::Null;
                    },
                    _ => ()
                }
            }
        }
    }
}
