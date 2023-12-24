use async_trait::async_trait;
use twitch_irc::login::{StaticLoginCredentials, RefreshingLoginCredentials, TokenStorage, UserAccessToken};
use twitch_irc::ClientConfig;

use std::collections::HashMap;

use oauth2::*;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::body;
use hyper::Request;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use std::fmt;

use chrono::{DateTime, Utc};

#[derive(Debug)]
struct BotTokenStorage {
    refresh_token: RefreshToken,
    access_token: AccessToken,
    created: DateTime<Utc>,
    expires: Option<DateTime<Utc>>
}

impl BotTokenStorage {
    async fn new() -> Self {
        let client = BasicClient::new(
                ClientId::new("id".to_string()),
                None,
                AuthUrl::new("https://id.twitch.tv/oauth2/authorize".to_string()).unwrap(),
                Some(TokenUrl::new("https://id.twitch.tv/oauth2/token".to_string()).unwrap())
            )
            .set_redirect_uri(RedirectUrl::new("127.0.0.1:3000".to_string()).unwrap());

        // twitch's oauth2 implementation doesn't support pkce afaik, so we can't use it :(

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("chat:edit".to_string()))
            .add_scope(Scope::new("chat:read".to_string()))
            .url();

        open::that(auth_url.as_str()).unwrap();

        if let Ok((auth_code, returned_csrf_token)) = listen_for_auth_code() {
            if returned_csrf_token == csrf_token {
                let token_result = client.exchange_code(auth_code).request_async(async_http_client).await.unwrap();
                let created = chrono::Utc::now();
                let expires = token_result.expires_in()
                    .map_or(None, |exp| Some(created.checked_add_signed(chrono::Duration::from_std(token_result.expires_in().unwrap()).unwrap()).unwrap()));
                Self { refresh_token: token_result.refresh_token().unwrap().clone(), access_token: token_result.access_token().clone(), created, expires }
            }
            else {
                todo!();
            }
        }
        else {
            todo!();
        }

    }
}

#[async_trait]
impl TokenStorage for BotTokenStorage {
    type LoadError = Error;
    type UpdateError = Error;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {

        todo!()
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        todo!()
    }
}

async fn listen_for_auth_code() -> Result<(AuthorizationCode, CsrfToken), GetAuthCodeError> {
    let (tx, rx) = oneshot::channel::<Result<(AuthorizationCode, CsrfToken), GetAuthCodeError>>();
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    let (stream, _) = listener.accept().await.unwrap();
    let io = TokioIo::new(stream);

    http1::Builder::new()
        .serve_connection(io, service_fn(move |req: Request<body::Incoming>| async move {
            if let fragment = req.uri().to_string().rsplit_once('#').unwrap().1 {

                let fields: HashMap<String, String> = fragment.split('&').
                    filter_map(|query| {
                        query.split_once('=')
                            .and_then(|t| Some((t.0.to_owned(), t.1.to_owned())))
                    }).collect();

                if let Some(error) = fields.get("error") {
                    tx.send(Err(GetAuthCodeError::TwitchError(error.to_string(), fields.get("error_description").unwrap_or_else(|| &"no error description?".to_string()).to_owned(), CsrfToken::new(fields.get("state").unwrap().to_string()))));
                }
                else if let Some(access_token) = fields.get("access_token") {
                    tx.send(Ok((AuthorizationCode::new(access_token.to_string()), CsrfToken::new(fields.get("state").unwrap().to_string()))));
                }

                Ok(hyper::Response::new(http_body_util::Empty::<hyper::body::Bytes>::new()))
            }
            else {
                Err("no token string")
            }
        })).await.unwrap();
    let result = match rx.await {
        Ok(res) => res,
        Err(_) => Err(GetAuthCodeError::Dropped)
    };

    result
}

enum GetAuthCodeError {
    TwitchError(String, String, CsrfToken),
    Dropped
}

#[derive(Debug)]
struct Error {
    
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
