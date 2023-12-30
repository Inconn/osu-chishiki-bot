use irc::client::{Client, data::Config};

use super::bot_config::OsuConfig;

pub struct IrcClient {
    client: Client,
    player: String
}

impl IrcClient {
    pub async fn new(osu_config: Option<OsuConfig>) -> Result<Self, Error> {
        match osu_config {
            None => Err(Error::NoConfig),
            Some(osu_config) => {
                if osu_config.beatmap_requests == false {
                    Err(Error::RequestsDisabled)
                }
                else if osu_config.name.is_none() || osu_config.password.is_none() {
                    Err(Error::NoLogin)
                }
                else {
                    let player = osu_config.player.unwrap_or_else(|| osu_config.name.clone().unwrap());
                    let config = Config {
                        nickname: osu_config.name,
                        server: Some(osu_config.server),
                        port: Some(6667),
                        password: osu_config.password,
                        use_tls: Some(false),
                        ..Default::default()
                    };
                    let client = Client::from_config(config).await?;
                    
                    Ok(Self {
                        client,
                        player
                    })
                }
            }
        }
    }
    
    pub async fn send_request<'a>(&self, beatmap_id: &'a str, beatmap_name: &String, beatmap_info: &'a str) -> Result<(), Error> {
        let message_to_send = format!("[https://osu.ppy.sh/b/{beatmap_id} {beatmap_name}] {beatmap_info}");
        Ok(self.client.send_privmsg(self.player.clone(), message_to_send)?)
    }
}

#[derive(Debug)]
pub enum Error {
    NoConfig,
    RequestsDisabled,
    NoLogin,
    IrcError(irc::error::Error)
}

impl From<irc::error::Error> for Error {
    fn from(item: irc::error::Error) -> Self {
        Self::IrcError(item)
    }
}
