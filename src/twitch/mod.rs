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

pub struct TwitchClient
{
    client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>,
    twitch_rx: UnboundedReceiver<ServerMessage>,
    gosu_json: Arc<RwLock<Gosumemory>>,
    bot_config: Arc<BotConfig>
}

impl TwitchClient
{
    pub fn new(bot_config: Arc<BotConfig>, gosu_json: Arc<RwLock<Gosumemory>>) -> TwitchClient {
        let config = ClientConfig::new_simple(
            StaticLoginCredentials::new(bot_config.name.to_owned(), Some(bot_config.token.trim_start_matches("oauth:").to_owned())));

        let (twitch_rx, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        client.join(bot_config.channel.to_owned()).unwrap();
        TwitchClient { client: Arc::new(client), twitch_rx, bot_config, gosu_json }
    }

    pub async fn listen(mut self) {
        while let Some(message) = self.twitch_rx.recv().await {
            log::trace!("got message from twitch!");
            match message {
                ServerMessage::Privmsg(message) => {
                    tokio::spawn(TwitchClient::process_message(self.client.clone(), message, self.gosu_json.clone(), self.bot_config.clone()));
                },
                ServerMessage::Join(join) => {
                    log::info!("Successfully joined channel {} with account {}", join.channel_login, join.user_login);
                },
                _ => ()
            }
        }
   }

    async fn process_message(client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>, message: PrivmsgMessage, _gosu_json: Arc<RwLock<Gosumemory>>, bot_config: Arc<BotConfig>) {
        log::trace!("got privmsgmessage that reads \"{}\", id: {}", message.message_text, message.message_id);
        if !message.is_action {
            if let Some(query) = message.message_text.strip_prefix(&bot_config.prefix) {
                let mut queries = query.split_whitespace();

                if let Some(command_name) = queries.next() {
                    let message_to_send = match command_name.to_lowercase().as_str() {
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
