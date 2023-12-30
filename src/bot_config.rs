use tokio::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BotConfig {
    pub twitch: TwitchConfig,
    pub osu: OsuConfig
}

#[derive(Debug, Deserialize)]
pub struct TwitchConfig {
    pub name: String,
    pub token: String,
    pub prefix: String,
    pub channel: String
}

/// this is the only way you can set default values in serde
fn default_server() -> String {
    "irc.ppy.sh".to_string()
}

#[derive(Debug, Deserialize)]
pub struct OsuConfig {
    #[serde(default)]
    pub beatmap_requests: bool,
    #[serde(default = "default_server")]
    pub server: String,
    pub name: Option<String>,
    pub player: Option<String>,
    pub password: Option<String>,
}

impl BotConfig {
    pub async fn read_file(path: &str) -> Result<Self, Error> {
        let config_file = fs::read(path).await?;
        let config_text = String::from_utf8_lossy(&config_file);
        Ok(toml::de::from_str(&config_text)?)
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseError(toml::de::Error)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "Failed to read bot config due to I/O error: {e}"),
            Self::ParseError(e) => write!(f, "Failed to read bot config due to parsing error: {e}")
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(ref e) => Some(e),
            Self::ParseError(ref e) => Some(e)
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(item: std::io::Error) -> Self {
        Self::IoError(item)
    }
}

impl From<toml::de::Error> for Error {
    fn from(item: toml::de::Error) -> Self {
        Self::ParseError(item)
    }
}
