pub mod listener;
pub mod structs;

pub use listener::Listener;

#[derive(Debug)]
pub enum Error {
    Websocket(async_tungstenite::tungstenite::Error),
    Parse(serde_json::Error),
    TimedOut
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Websocket(e) => write!(f, "Gosumemory listener failed due to websocket error: {e}"),
            Self::Parse(e) => write!(f, "Gosumemory listener failed due to parsing error: {e}"),
            Self::TimedOut => write!(f, "Gosumemory websocket timed out")
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Websocket(ref e) => Some(e),
            Self::Parse(ref e) => Some(e),
            Self::TimedOut => None
        }
    }
}

impl From<async_tungstenite::tungstenite::Error> for Error {
    fn from(item: async_tungstenite::tungstenite::Error) -> Self {
        Self::Websocket(item)
    }
}

impl From<serde_json::Error> for Error {
    fn from(item: serde_json::Error) -> Self {
        Self::Parse(item)
    }
}
