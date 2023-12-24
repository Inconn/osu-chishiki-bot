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

    let gosu_json = Arc::new(RwLock::new(Gosumemory::default()));

    // temp commented out since it fails when you're not running gosu
    //let gosu_ws_url = "wss://127.0.0.1:24050/ws".parse().unwrap();
    //let gosu = gosu::GosuListener::new(gosu_ws_url, gosu_json.clone()).await.unwrap();
    //let gosu_handle = tokio::spawn(gosu.listen());

    let twitch_client = twitch::TwitchClient::new(bot_config, gosu_json);
    let twitch_handle = tokio::spawn(twitch_client.listen());

    let res = tokio::try_join!(/*gosu_handle,*/ twitch_handle);

    match res {
        Ok((first)) => (),
        Err(err) => todo!()
    }
}
