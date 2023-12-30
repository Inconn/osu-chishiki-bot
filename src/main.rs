#![forbid(unsafe_code)]

mod bancho;
mod bot_config;
mod gosu;
mod twitch;

use std::sync::Arc;
use tokio::sync::RwLock;
use gosumemory_helper::Gosumemory;

#[tokio::main]
async fn main() -> Result<(), HandleError> {
    env_logger::init();

    let bot_config = bot_config::BotConfig::read_file("bot_config.toml").await.expect("couldn't read bot config");
    
    // temporary: use file to provide gosu_json
    //let gosu_file = tokio::fs::read("gosu_test.json").await.unwrap();
    //let gosu_text = String::from_utf8_lossy(&gosu_file);
    //let gosu_json = Arc::new(RwLock::new(serde_json::from_str(&gosu_text).unwrap()));
    
    let gosu_json = Arc::new(RwLock::new(Gosumemory::default()));

    let gosu_ws_url = "ws://127.0.0.1:9001/ws".parse().unwrap();
    let gosu = gosu::Listener::new(gosu_ws_url, gosu_json.clone()).await
        .expect("Failed to connect to the gosumemory websocket. Please make sure both gosumemory AND osu! are open.");
    let gosu_handle = tokio::spawn(gosu.listen());

    // note: OsuConfig in the BotConfig struct is moved here.
    let bancho_client: Option<bancho::IrcClient> = bancho::IrcClient::new(bot_config.osu)
        .await.map_err(|err: bancho::Error| {
            match err {
                bancho::Error::IrcError(_) => return Err(err),
                _ => Err::<bancho::IrcClient, bancho::Error>(err)
            }
        }).map(|client: bancho::IrcClient| Some(client))
        .unwrap_or_default();

    let twitch_client = twitch::Client::new(bot_config.twitch, gosu_json, bancho_client);
    let twitch_handle = tokio::spawn(twitch_client.listen());

    let res = tokio::try_join!(flatten(gosu_handle), flatten(twitch_handle));

    match res {
        Ok((_gosu, _twitch)) => Ok(()),
        Err(err) => Err(err)
    }
}

async fn flatten<T, E>(handle: tokio::task::JoinHandle<Result<T, E>>) -> Result<T, HandleError> 
where
    T: Send,
    E: Into<HandleError> + Send
{
    match handle.await {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(err)) => Err(err.into()),
        Err(err) => panic!("{err}")
    }
}

#[derive(Debug)]
enum HandleError {
    Gosu(gosu::Error),
    BanchoIrc(bancho::Error),
    Twitch(String)
}

impl From<gosu::Error> for HandleError {
    fn from(item: gosu::Error) -> Self {
        Self::Gosu(item)
    }
}

impl From<bancho::Error> for HandleError {
    fn from(item: bancho::Error) -> Self {
        Self::BanchoIrc(item)
    }
}

impl From<String> for HandleError {
    fn from(item: String) -> Self {
        Self::Twitch(item)
    }
}
