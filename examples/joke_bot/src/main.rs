use std::{convert::TryInto, error::Error, io, process::exit, time::Duration};

use ruma::{
    api::client::r0::{
        filter::FilterDefinition, membership::join_room_by_id, message::send_message_event,
        sync::sync_events,
    },
    assign, client,
    events::{
        room::message::{MessageType, RoomMessageEventContent},
        AnySyncMessageEvent, AnySyncRoomEvent,
    },
    identifiers::TransactionId,
    presence::PresenceState,
    serde::Raw,
    RoomId, UserId,
};
use serde_json::Value as JsonValue;
use tokio::fs;
use tokio_stream::StreamExt as _;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        exit(1)
    }
    Ok(())
}

type HttpClient = client::http_client::HyperNativeTls;
type MatrixClient = client::Client<client::http_client::HyperNativeTls>;

async fn run() -> Result<(), Box<dyn Error>> {
    let config =
        read_config().await.map_err(|e| format!("configuration in ./config is invalid: {}", e))?;
    let http_client =
        hyper::Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());
    let matrix_client = if let Some(state) = read_state().await.ok().flatten() {
        MatrixClient::with_http_client(
            http_client.clone(),
            config.homeserver.to_owned(),
            Some(state.access_token),
        )
    } else if config.password.is_some() {
        let client = create_matrix_session(http_client.clone(), &config).await?;

        if let Err(err) = write_state(&State {
            access_token: client.access_token().expect("Matrix access token is missing"),
        })
        .await
        {
            eprintln!("Failed to persist access token to disk. Re-authentication will be required on the next startup: {}", err)
        }
        client
    } else {
        return Err("No previous session found and no credentials stored in config".into());
    };

    let filter = FilterDefinition::ignore_all().into();
    let initial_sync_response = matrix_client
        .send_request(assign!(sync_events::Request::new(), {
            filter: Some(&filter),
        }))
        .await?;
    let user_id = &config.username;
    let not_senders = &[user_id.clone()];
    let filter = {
        let mut filter = FilterDefinition::empty();
        filter.room.timeline.not_senders = not_senders;
        filter
    }
    .into();

    let mut sync_stream = Box::pin(matrix_client.sync(
        Some(&filter),
        initial_sync_response.next_batch,
        &PresenceState::Online,
        Some(Duration::from_secs(30)),
    ));
    println!("Listening...");
    while let Some(response) = sync_stream.try_next().await? {
        for (room_id, room_info) in response.rooms.join {
            for e in &room_info.timeline.events {
                match handle_messages(&http_client, &matrix_client, e, &room_id, user_id).await {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("failed to respond to message: {}", err)
                    }
                }
            }
        }

        for (room_id, _) in response.rooms.invite {
            match handle_invitations(&http_client, &matrix_client, &room_id).await {
                Ok(_) => {}
                Err(err) => eprintln!("failed to accept invitation for room {}: {}", &room_id, err),
            }
        }
    }
    Ok(())
}

async fn create_matrix_session(
    http_client: HttpClient,
    config: &Config,
) -> Result<MatrixClient, Box<dyn Error>> {
    if let Some(password) = &config.password {
        let client =
            MatrixClient::with_http_client(http_client, config.homeserver.to_owned(), None);
        if let Err(e) = client.log_in(config.username.as_ref(), password, None, None).await {
            let reason = match e {
                client::Error::AuthenticationRequired => "invalid credentials specified".to_owned(),
                client::Error::Response(response_err) => {
                    format!("failed to get a response from the server: {}", response_err)
                }
                client::Error::FromHttpResponse(parse_err) => {
                    format!("failed to parse log in response: {}", parse_err)
                }
                _ => e.to_string(),
            };
            return Err(format!("Failed to log in: {}", reason).into());
        }
        Ok(client)
    } else {
        Err("Failed to create session: no password stored in config".to_owned().into())
    }
}

async fn handle_messages(
    http_client: &HttpClient,
    matrix_client: &MatrixClient,
    e: &Raw<AnySyncRoomEvent>,
    room_id: &RoomId,
    bot_user_id: &UserId,
) -> Result<(), Box<dyn Error>> {
    if let Ok(AnySyncRoomEvent::Message(AnySyncMessageEvent::RoomMessage(m))) = e.deserialize() {
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
                let req = send_message_event::Request::new(room_id, &txn_id, &joke_content)?;
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
    room_id: &RoomId,
) -> Result<(), Box<dyn Error>> {
    println!("invited to {}", &room_id);
    matrix_client.send_request(join_room_by_id::Request::new(room_id)).await?;

    let greeting = "Hello! My name is Mr. Bot! I like to tell jokes. Like this one: ";
    let joke = get_joke(http_client).await.unwrap_or_else(|_| "err... never mind.".to_owned());
    let content = RoomMessageEventContent::text_plain(format!("{}\n{}", greeting, joke));
    let txn_id = TransactionId::new();
    let message = send_message_event::Request::new(room_id, &txn_id, &content)?;
    matrix_client.send_request(message).await?;
    Ok(())
}

async fn get_joke(client: &HttpClient) -> Result<String, Box<dyn Error>> {
    let uri = "https://v2.jokeapi.dev/joke/Programming,Pun,Misc?safe-mode&type=single"
        .parse::<hyper::Uri>()?;
    let rsp = client.get(uri).await?;
    let bytes = hyper::body::to_bytes(rsp).await?;
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
    username: Box<UserId>,
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
                            format!("invalid Matrix user ID format for `username`: {}", e)
                        })
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
                error.push_str("\n  required field `homeserver` is missing")
            }
            if let Err(e) = username {
                error.push_str(&format!("\n  {}", e))
            }
            Err(io::Error::new(io::ErrorKind::InvalidData, error))
        }
    }
}
