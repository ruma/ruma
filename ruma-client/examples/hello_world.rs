use std::{convert::TryFrom, env, process::exit};

use http::Uri;
use ruma::{
    api::client::r0::{alias::get_alias, membership::join_room_by_id, message::send_message_event},
    events::{room::message::MessageEventContent, AnyMessageEventContent},
    RoomAliasId,
};
use ruma_client::{self, Client};

async fn hello_world(
    homeserver_url: Uri,
    username: &str,
    password: &str,
    room_alias: &RoomAliasId,
) -> anyhow::Result<()> {
    let client = Client::new(homeserver_url, None);
    client.log_in(username, password, None, Some("ruma-example-client")).await?;

    let room_id = client.request(get_alias::Request::new(room_alias)).await?.room_id;
    client.request(join_room_by_id::Request::new(&room_id)).await?;
    client
        .request(send_message_event::Request::new(
            &room_id,
            "1",
            &AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain("Hello World!")),
        ))
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

    hello_world(
        homeserver_url.parse()?,
        &username,
        &password,
        &RoomAliasId::try_from(room.as_str())?,
    )
    .await
}
