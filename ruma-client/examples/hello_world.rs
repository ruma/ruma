use std::{convert::TryFrom, env, process::exit};

use http::Uri;
use ruma::{
    api::client::r0::{alias::get_alias, membership::join_room_by_id, message::send_message_event},
    events::{room::message::MessageEventContent, AnyMessageEventContent},
    RoomAliasId,
};
use ruma_client::{self, Client};

async fn hello_world(homeserver_url: Uri, room_alias: &RoomAliasId) -> anyhow::Result<()> {
    let client = Client::new(homeserver_url, None);
    client.register_guest().await?;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (homeserver_url, room) = match (env::args().nth(1), env::args().nth(2)) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            eprintln!("Usage: {} <homeserver_url> <room>", env::args().next().unwrap());
            exit(1)
        }
    };

    hello_world(homeserver_url.parse()?, &RoomAliasId::try_from(room.as_str())?).await
}
