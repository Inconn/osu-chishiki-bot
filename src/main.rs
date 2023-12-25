mod twitch;
mod bot_config;
mod gosu;

use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    env_logger::init();

    let bot_config = Arc::new(bot_config::BotConfig::read_file("gosubot_config.json").await.expect("couldn't read bot config"));

    // temporary: use file to provide gosu_json
    //let gosu_file = tokio::fs::read("gosu_test.json").await.unwrap();
    //let gosu_text = String::from_utf8_lossy(&gosu_file);
    //let gosu_json = Arc::new(RwLock::new(serde_json::from_str(&gosu_text).unwrap()));
    
    let gosu_json = Arc::new(RwLock::new(Box::default()));

    let gosu_ws_url = "ws://127.0.0.1:24050/ws".parse().unwrap();
    let gosu = gosu::Listener::new(gosu_ws_url, gosu_json.clone()).await
        .expect("Failed to connect to the gosumemory websocket. Please make sure both gosumemory AND osu! are open.");
    let gosu_handle = tokio::spawn(gosu.listen());

    let twitch_client = twitch::Client::new(bot_config, gosu_json);
    let twitch_handle = tokio::spawn(twitch_client.listen());

    let res = tokio::try_join!(flatten(gosu_handle), flatten(twitch_handle));

    match res {
        Ok((_gosu, _twitch)) => (),
        Err(e) => panic!("one branch failed: {e:?}")
    }
}

async fn flatten<T, E>(handle: tokio::task::JoinHandle<Result<T, E>>) -> Result<T, HandleError> 
where
    E: Into<HandleError>
{
    match handle.await {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(err)) => Err(err.into()),
        Err(_err) => panic!()
    }
}

#[derive(Debug)]
enum HandleError {
    Gosu(gosu::Error),
    Twitch(String)
}

impl From<gosu::Error> for HandleError {
    fn from(item: gosu::Error) -> Self {
        Self::Gosu(item)
    }
}

impl From<String> for HandleError {
    fn from(item: String) -> Self {
        Self::Twitch(item)
    }
}
