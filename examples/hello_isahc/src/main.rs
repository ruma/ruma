use std::{convert::TryFrom, env, process::exit};

use ruma::{
    api::client::{alias::get_alias, membership::join_room_by_id, message::send_message_event},
    events::room::message::RoomMessageEventContent,
    RoomAliasId, TransactionId,
};

async fn hello_world(
    homeserver_url: String,
    username: &str,
    password: &str,
    room_alias: &RoomAliasId,
) -> anyhow::Result<()> {
    let http_client = isahc::HttpClient::new()?;
    let client =
        ruma::Client::builder().homeserver_url(homeserver_url).http_client(http_client).await?;
    client.log_in(username, password, None, Some("ruma-example-client")).await?;

    let room_id = client.send_request(get_alias::v3::Request::new(room_alias)).await?.room_id;
    client.send_request(join_room_by_id::v3::Request::new(&room_id)).await?;
    client
        .send_request(send_message_event::v3::Request::new(
            &room_id,
            &TransactionId::new(),
            &RoomMessageEventContent::text_plain("Hello World!"),
        )?)
        .await?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let (homeserver_url, username, password, room) =
        match (env::args().nth(1), env::args().nth(2), env::args().nth(3), env::args().nth(4)) {
            (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
            _ => {
                eprintln!(
                    "Usage: {} <homeserver_url> <username> <password> <room>",
                    env::args().next().unwrap()
                );
                exit(1)
            }
        };

    hello_world(homeserver_url, &username, &password, <&RoomAliasId>::try_from(room.as_str())?)
        .await
}
