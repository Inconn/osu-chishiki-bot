mod twitch;
mod bot_config;
mod gosu;

use std::sync::Arc;
use tokio::sync::RwLock;

use gosumemory_helper::Gosumemory;

#[tokio::main]
async fn main() {
    env_logger::init();

    let bot_config = Arc::new(bot_config::BotConfig::read_file("gosubot_config.json").await.expect("couldn't read bot config"));

    // temporary: use file to provide gosu_json
    //let gosu_file = tokio::fs::read("gosu_test.json").await.unwrap();
    //let gosu_text = String::from_utf8_lossy(&gosu_file);
    //let gosu_json = Arc::new(RwLock::new(serde_json::from_str(&gosu_text).unwrap()));
    
    let gosu_json = Arc::new(RwLock::new(Gosumemory::default()));

    let gosu_ws_url = "ws://127.0.0.1:24050/ws".parse().unwrap();
    let gosu = gosu::GosuListener::new(gosu_ws_url, gosu_json.clone()).await
        .expect("Failed to connect to the gosumemory websocket. Please make sure both gosumemory AND osu! are open.");
    let gosu_handle = tokio::spawn(gosu.listen());

    let twitch_client = twitch::TwitchClient::new(bot_config, gosu_json);
    let twitch_handle = tokio::spawn(twitch_client.listen());

    let res = tokio::try_join!(gosu_handle, twitch_handle);

    match res {
        Ok((_gosu, _twitch)) => (),
        Err(_err) => todo!()
    }
}
