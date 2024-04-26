use std::{error::Error, io, process::exit, time::Duration};

use futures_util::future::{join, join_all};
use http_body_util::BodyExt as _;
use hyper_util::rt::TokioExecutor;
use ruma::{
    api::client::{
        filter::FilterDefinition, membership::join_room_by_id, message::send_message_event,
        sync::sync_events,
    },
    assign, client,
    events::{
        room::message::{MessageType, RoomMessageEventContent},
        AnySyncMessageLikeEvent, AnySyncTimelineEvent, SyncMessageLikeEvent,
    },
    presence::PresenceState,
    serde::Raw,
    OwnedRoomId, OwnedUserId, TransactionId, UserId,
};
use serde_json::Value as JsonValue;
use tokio::fs;
use tokio_stream::StreamExt as _;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = run().await {
        eprintln!("{e}");
        exit(1)
    }
    Ok(())
}

type HttpClient = client::http_client::HyperNativeTls;
type MatrixClient = client::Client<client::http_client::HyperNativeTls>;

async fn run() -> Result<(), Box<dyn Error>> {
    let config =
        read_config().await.map_err(|e| format!("configuration in ./config is invalid: {e}"))?;
    let http_client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
        .build(hyper_tls::HttpsConnector::new());
    let matrix_client = if let Some(state) = read_state().await.ok().flatten() {
        ruma::Client::builder()
            .homeserver_url(config.homeserver.clone())
            .access_token(Some(state.access_token))
            .http_client(http_client.clone())
            .await?
    } else if config.password.is_some() {
        let client = create_matrix_session(http_client.clone(), &config).await?;

        if let Err(err) = write_state(&State {
            access_token: client.access_token().expect("Matrix access token is missing"),
        })
        .await
        {
            eprintln!(
                "Failed to persist access token to disk. \
                 Re-authentication will be required on the next startup: {err}",
            );
        }
        client
    } else {
        return Err("No previous session found and no credentials stored in config".into());
    };

    let filter = FilterDefinition::ignore_all().into();
    let initial_sync_response = matrix_client
        .send_request(assign!(sync_events::v3::Request::new(), {
            filter: Some(filter),
        }))
        .await?;
    let user_id = &config.username;
    let not_senders = vec![user_id.clone()];
    let filter = {
        let mut filter = FilterDefinition::empty();
        filter.room.timeline.not_senders = not_senders;
        filter
    }
    .into();

    let mut sync_stream = Box::pin(matrix_client.sync(
        Some(filter),
        initial_sync_response.next_batch,
        PresenceState::Online,
        Some(Duration::from_secs(30)),
    ));

    // Prevent the clients being moved by `async move` blocks
    let http_client = &http_client;
    let matrix_client = &matrix_client;

    println!("Listening...");
    while let Some(response) = sync_stream.try_next().await? {
        let message_futures = response.rooms.join.iter().map(|(room_id, room_info)| async move {
            // Use a regular for loop for the messages within one room to handle them sequentially
            for e in &room_info.timeline.events {
                if let Err(err) =
                    handle_message(http_client, matrix_client, e, room_id.to_owned(), user_id).await
                {
                    eprintln!("failed to respond to message: {err}");
                }
            }
        });

        let invite_futures = response.rooms.invite.into_keys().map(|room_id| async move {
            if let Err(err) = handle_invitations(http_client, matrix_client, room_id.clone()).await
            {
                eprintln!("failed to accept invitation for room {room_id}: {err}");
            }
        });

        // Handle messages from different rooms as well as invites concurrently
        join(join_all(message_futures), join_all(invite_futures)).await;
    }

    Ok(())
}

async fn create_matrix_session(
    http_client: HttpClient,
    config: &Config,
) -> Result<MatrixClient, Box<dyn Error>> {
    if let Some(password) = &config.password {
        let client = ruma::Client::builder()
            .homeserver_url(config.homeserver.clone())
            .http_client(http_client)
            .await?;

        if let Err(e) = client.log_in(config.username.as_ref(), password, None, None).await {
            let reason = match e {
                client::Error::AuthenticationRequired => "invalid credentials specified".to_owned(),
                client::Error::Response(response_err) => {
                    format!("failed to get a response from the server: {response_err}")
                }
                client::Error::FromHttpResponse(parse_err) => {
                    format!("failed to parse log in response: {parse_err}")
                }
                _ => e.to_string(),
            };
            return Err(format!("Failed to log in: {reason}").into());
        }

        Ok(client)
    } else {
        Err("Failed to create session: no password stored in config".to_owned().into())
    }
}

async fn handle_message(
    http_client: &HttpClient,
    matrix_client: &MatrixClient,
    e: &Raw<AnySyncTimelineEvent>,
    room_id: OwnedRoomId,
    bot_user_id: &UserId,
) -> Result<(), Box<dyn Error>> {
    if let Ok(AnySyncTimelineEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(
        SyncMessageLikeEvent::Original(m),
    ))) = e.deserialize()
    {
        // workaround because Conduit does not implement filtering.
        if m.sender == bot_user_id {
            return Ok(());
        }

        if let MessageType::Text(t) = m.content.msgtype {
            println!("{}:\t{}", m.sender, t.body);
            if t.body.to_ascii_lowercase().contains("joke") {
                let joke = match get_joke(http_client).await {
                    Ok(joke) => joke,
                    Err(_) => "I thought of a joke... but I just forgot it.".to_owned(),
                };
                let joke_content = RoomMessageEventContent::text_plain(joke);

                let txn_id = TransactionId::new();
                let req = send_message_event::v3::Request::new(
                    room_id.to_owned(),
                    txn_id,
                    &joke_content,
                )?;
                // Do nothing if we can't send the message.
                let _ = matrix_client.send_request(req).await;
            }
        }
    }

    Ok(())
}

async fn handle_invitations(
    http_client: &HttpClient,
    matrix_client: &MatrixClient,
    room_id: OwnedRoomId,
) -> Result<(), Box<dyn Error>> {
    println!("invited to {room_id}");
    matrix_client.send_request(join_room_by_id::v3::Request::new(room_id.clone())).await?;

    let greeting = "Hello! My name is Mr. Bot! I like to tell jokes. Like this one: ";
    let joke = get_joke(http_client).await.unwrap_or_else(|_| "err... never mind.".to_owned());
    let content = RoomMessageEventContent::text_plain(format!("{greeting}\n{joke}"));
    let txn_id = TransactionId::new();
    let message = send_message_event::v3::Request::new(room_id, txn_id, &content)?;
    matrix_client.send_request(message).await?;
    Ok(())
}

async fn get_joke(client: &HttpClient) -> Result<String, Box<dyn Error>> {
    let uri = "https://v2.jokeapi.dev/joke/Programming,Pun,Misc?safe-mode&type=single"
        .parse::<hyper::Uri>()?;
    let rsp = client.get(uri).await?;
    let bytes = rsp.into_body().collect().await?.to_bytes();
    let joke_obj = serde_json::from_slice::<JsonValue>(&bytes)
        .map_err(|_| "invalid JSON returned from joke API")?;
    let joke = joke_obj["joke"].as_str().ok_or("joke field missing from joke API response")?;
    Ok(joke.to_owned())
}

struct State {
    access_token: String,
}

async fn write_state(state: &State) -> io::Result<()> {
    let content = &state.access_token;
    fs::write("./session", content).await?;
    Ok(())
}

async fn read_state() -> io::Result<Option<State>> {
    match fs::read_to_string("./session").await {
        Ok(access_token) => Ok(Some(State { access_token })),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

struct Config {
    homeserver: String,
    username: OwnedUserId,
    password: Option<String>,
}

async fn read_config() -> io::Result<Config> {
    let content = fs::read_to_string("./config").await?;
    let lines = content.split('\n');

    let mut homeserver = None;
    let mut username = Err("required field `username` is missing".to_owned());
    let mut password = None;
    for line in lines {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "homeserver" => homeserver = Some(value.trim().to_owned()),
                // TODO: infer domain from `homeserver`
                "username" => {
                    username =
                        value.trim().to_owned().try_into().map_err(|e| {
                            format!("invalid Matrix user ID format for `username`: {e}")
                        });
                }
                "password" => password = Some(value.trim().to_owned()),
                _ => {}
            }
        }
    }

    match (homeserver, username) {
        (Some(homeserver), Ok(username)) => Ok(Config { homeserver, username, password }),
        (homeserver, username) => {
            let mut error = String::from("Invalid config specified:");
            if homeserver.is_none() {
                error.push_str("\n  required field `homeserver` is missing");
            }
            if let Err(e) = username {
                error.push_str("\n  ");
                error.push_str(&e);
            }
            Err(io::Error::new(io::ErrorKind::InvalidData, error))
        }
    }
}
