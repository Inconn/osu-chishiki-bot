use twitch_irc::{TwitchIRCClient, ClientConfig};
use twitch_irc::message::{ServerMessage, PrivmsgMessage};
use twitch_irc::transport::tcp::SecureTCPTransport;
use twitch_irc::login::StaticLoginCredentials;

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedReceiver;

use rosu_pp::{Beatmap, BeatmapExt};

use super::bot_config::TwitchConfig;
use super::rosu::structs::RosuValues;
use super::bancho;

//mod auth;

pub struct Client
{
    client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>,
    bancho_client: Arc<Option<bancho::IrcClient>>,
    twitch_rx: UnboundedReceiver<ServerMessage>,
    rosu_value: Arc<RwLock<RosuValues>>,
    twitch_config: Arc<TwitchConfig>
}

impl Client
{
    pub fn new(twitch_config: TwitchConfig, rosu_value: Arc<RwLock<RosuValues>>, bancho_client: Option<bancho::IrcClient>) -> Self {
        let config = ClientConfig::new_simple(
            StaticLoginCredentials::new(twitch_config.name.clone(), Some(twitch_config.token.trim_start_matches("oauth:").to_string())));

        let (twitch_rx, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        client.join(twitch_config.channel.clone()).unwrap();
        Self {
            client: Arc::new(client),
            bancho_client: Arc::new(bancho_client),
            twitch_rx,
            twitch_config: Arc::new(twitch_config),
            rosu_value
        }
    }

    pub async fn listen(mut self) -> Result<(), String> {
        while let Some(message) = self.twitch_rx.recv().await {
            log::trace!("got message from twitch!");
            match message {
                ServerMessage::Privmsg(message) => {
                    tokio::spawn(Self::process_message(self.client.clone(), self.bancho_client.clone(), message, self.rosu_value.clone(), self.twitch_config.clone()));
                },
                ServerMessage::Join(join) => {
                    log::info!("Successfully joined channel {} with account {}", join.channel_login, join.user_login);
                },
                _ => ()
            }
        }
        Ok(())
   }

    async fn process_message(client: Arc<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>, bancho_client: Arc<Option<bancho::IrcClient>>, message: PrivmsgMessage, rosu_value: Arc<RwLock<RosuValues>>, twitch_config: Arc<TwitchConfig>) {
        log::trace!("got privmsgmessage that reads \"{}\", id: {}", message.message_text, message.message_id);
        if !message.is_action {
            if let Some(query) = message.message_text.strip_prefix(&twitch_config.prefix) {
                let mut queries = query.split_whitespace();

                if let Some(command_name) = queries.next() {
                    let message_to_send = match command_name.to_lowercase().as_str() {
                        "np" | "nowplaying" | "nowplay" | "map" => {
                            let rosu_value_read = rosu_value.read().await;
                            let mods_str: String = rosu_value_read.mods_str.iter().map(|item: &String| item.as_str()).collect();
                            format!("osu.ppy.sh/b/{} {} - {} [{}] + {} {:.2}★",
                                    rosu_value_read.map_id,
                                    rosu_value_read.artist,
                                    rosu_value_read.title,
                                    rosu_value_read.difficulty,
                                    mods_str,
                                    rosu_value_read.stars_mods
                                    )
                        }
                        "nppp" | "nowplayingpp" | "nowplaypp" | "mappp" => {
                            let rosu_value_read = rosu_value.read().await;
                            let mods_str: String = rosu_value_read.mods_str.iter().map(|item: &String| item.as_str()).collect();

                            let beatmap = Beatmap::from_path(rosu_value_read.beatmap_full_path.clone()).await.unwrap().convert_mode(rosu_value_read.mode.into()).into_owned();
                            // TODO: simply this, it feels too big
                            let mut pp = [0.0; 5];
                            pp[0] = rosu_value_read.ss_pp;
                            let attr = if pp[0] == 0.0 {
                                let attr_temp = beatmap.max_pp(rosu_value_read.menu_mods.bits());
                                pp[0] = attr_temp.pp();
                                pp[1] = beatmap.pp()
                                    .attributes(attr_temp.clone())
                                    .accuracy(99.)
                                    .calculate()
                                    .pp();

                                Some(attr_temp)
                            }
                            else {
                                let attr_temp = beatmap.pp()
                                    .mods(rosu_value_read.menu_mods.bits())
                                    .accuracy(99.)
                                    .calculate();

                                pp[1] = attr_temp.pp();
                                Some(attr_temp)
                            };

                            let attr = attr.unwrap();
                            for i in 2..pp.len() {
                                pp[i] = beatmap.pp()
                                    .attributes(attr.clone())
                                    .accuracy((100 - i) as f64)
                                    .calculate()
                                    .pp();
                            }

                            format!("osu.ppy.sh/b/{} {} - {} [{}] + {} {:.2}★ | 100%: {:.0}pp | 99%: {:.0}pp | 98%: {:.0}pp | 97%: {:.0}pp | 96%: {:.0}pp | 95%: {:.0}pp",
                                    rosu_value_read.map_id,
                                    rosu_value_read.artist,
                                    rosu_value_read.title,
                                    rosu_value_read.difficulty,
                                    mods_str,
                                    rosu_value_read.stars_mods,
                                    pp[0],
                                    pp[1],
                                    pp[2],
                                    pp[3],
                                    pp[4],
                                    pp[5]
                                    )
                        }
                        "rq" | "req" | "request" => {
                            if let Some(ref bancho_client) =  *bancho_client {
                                let rosu_value_read = rosu_value.read().await;
                                let beatmap_id = queries.next().unwrap();
                                // this is wrong, TODO: Fix this
                                // requires using osu api
                                let beatmap_name = format!("{} - {} [{}] {:.2}★",
                                                           rosu_value_read.artist,
                                                           rosu_value_read.title,
                                                           rosu_value_read.difficulty,
                                                           rosu_value_read.stars_mods
                                                           );
                                let _ = bancho_client.send_request(
                                    beatmap_id,
                                    &beatmap_name,
                                    queries.next().unwrap_or_default()
                                    ).await;
                                
                                format!("Added request {beatmap_name} osu.ppy.sh/b/{}", rosu_value_read.map_id)
                            }
                            else {
                                String::new()
                            }
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
