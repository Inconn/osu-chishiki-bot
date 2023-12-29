use twitch_irc::{TwitchIRCClient, ClientConfig};
use twitch_irc::message::{ServerMessage, PrivmsgMessage};
use twitch_irc::transport::tcp::SecureTCPTransport;
use twitch_irc::login::StaticLoginCredentials;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedReceiver;

use gosumemory_helper::Gosumemory;
use super::bot_config::BotConfig;

//mod auth;

pub struct Client
{
    client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>,
    twitch_rx: UnboundedReceiver<ServerMessage>,
    gosu_json: Arc<RwLock<Box<Gosumemory>>>,
    bot_config: Arc<BotConfig>
}

impl Client
{
    pub fn new(bot_config: Arc<BotConfig>, gosu_json: Arc<RwLock<Box<Gosumemory>>>) -> Self {
        let config = ClientConfig::new_simple(
            StaticLoginCredentials::new(bot_config.name.clone(), Some(bot_config.token.trim_start_matches("oauth:").to_string())));

        let (twitch_rx, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        client.join(bot_config.channel.clone()).unwrap();
        Self { client: Arc::new(client), twitch_rx, bot_config, gosu_json }
    }

    pub async fn listen(mut self) -> Result<(), String> {
        while let Some(message) = self.twitch_rx.recv().await {
            log::trace!("got message from twitch!");
            match message {
                ServerMessage::Privmsg(message) => {
                    tokio::spawn(Self::process_message(self.client.clone(), message, self.gosu_json.clone(), self.bot_config.clone()));
                },
                ServerMessage::Join(join) => {
                    log::info!("Successfully joined channel {} with account {}", join.channel_login, join.user_login);
                },
                _ => ()
            }
        }
        Ok(())
   }

    async fn process_message(client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>, message: PrivmsgMessage, gosu_json: Arc<RwLock<Box<Gosumemory>>>, bot_config: Arc<BotConfig>) {
        log::trace!("got privmsgmessage that reads \"{}\", id: {}", message.message_text, message.message_id);
        if !message.is_action {
            if let Some(query) = message.message_text.strip_prefix(&bot_config.prefix) {
                let mut queries = query.split_whitespace();

                if let Some(command_name) = queries.next() {
                    let message_to_send = match command_name.to_lowercase().as_str() {
                        "np" => {
                            let gosu_json_read = gosu_json.read().await;
                            format!("osu.ppy.sh/b/{} {} - {} [{}] + {} {}★",
                                    gosu_json_read.menu.bm.id,
                                    gosu_json_read.menu.bm.metadata.artist,
                                    gosu_json_read.menu.bm.metadata.title,
                                    gosu_json_read.menu.bm.metadata.difficulty,
                                    gosu_json_read.menu.mods.str,
                                    gosu_json_read.menu.bm.stats.full_sr
                                    )
                        }
                        "nppp" => {
                            let gosu_json_read = gosu_json.read().await;
                            format!("osu.ppy.sh/b/{} {} - {} [{}] + {} {}★ | 100%: {}pp | 99%: {}pp | 98%: {}pp | 97%: {}pp | 96%: {}pp | 95%: {}pp",
                                    gosu_json_read.menu.bm.id,
                                    gosu_json_read.menu.bm.metadata.artist,
                                    gosu_json_read.menu.bm.metadata.title,
                                    gosu_json_read.menu.bm.metadata.difficulty,
                                    gosu_json_read.menu.mods.str,
                                    gosu_json_read.menu.bm.stats.full_sr,
                                    gosu_json_read.menu.pp.n100,
                                    gosu_json_read.menu.pp.n99,
                                    gosu_json_read.menu.pp.n98,
                                    gosu_json_read.menu.pp.n97,
                                    gosu_json_read.menu.pp.n96,
                                    gosu_json_read.menu.pp.n95
                                    )

                        }
                        "ping" => "Pong!".to_string(),
                        _ => String::new()
                    };
                    
                    if !message_to_send.is_empty() {
                        log::trace!("replying to message {} with message \"{}\"", message.message_id, message_to_send);
                        let _ = client.say_in_reply_to(&message, message_to_send).await;
                    }
                }
            }
        }
    }
}
