use std::path::Path;
use tokio::fs;

use serde::{Serialize, Deserialize};

//use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct BotConfig {
    pub name: String,
    pub token: String, // TODO: make it so token is in better place?
    pub prefix: String,
    pub channel: String,
//    pub commands: HashMap<String, String>
}

impl BotConfig {
    pub async fn read_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let config_file = fs::read(path).await?;
        let config_text = String::from_utf8_lossy(&config_file);
        Ok(serde_json::from_str(&config_text)?)
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseError(serde_json::Error)
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

impl From<serde_json::Error> for Error {
    fn from(item: serde_json::Error) -> Self {
        Self::ParseError(item)
    }
}
