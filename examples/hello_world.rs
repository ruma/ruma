#![feature(async_await)]

use std::{convert::TryFrom, env, process::exit};

use ruma_client::{
    self,
    events::{
        room::message::{MessageEventContent, MessageType, TextMessageEventContent},
        EventType,
    },
    identifiers::RoomAliasId,
    Client,
};
use ruma_client_api::r0;
use url::Url;

async fn hello_world(homeserver_url: Url, room: String) -> Result<(), ruma_client::Error> {
    let client = Client::new(homeserver_url, None);

    client.register_guest().await?;
    let response = client
        .request::<r0::alias::get_alias::Endpoint>(r0::alias::get_alias::Request {
            room_alias: RoomAliasId::try_from(&room[..]).unwrap(),
        })
        .await?;

    let room_id = response.room_id;

    client
        .request::<r0::membership::join_room_by_id::Endpoint>(
            r0::membership::join_room_by_id::Request {
                room_id: room_id.clone(),
                third_party_signed: None,
            },
        )
        .await?;

    client
        .request::<r0::send::send_message_event::Endpoint>(r0::send::send_message_event::Request {
            room_id: room_id,
            event_type: EventType::RoomMessage,
            txn_id: "1".to_owned(),
            data: MessageEventContent::Text(TextMessageEventContent {
                msgtype: MessageType::Text,
                body: "Hello World!".to_owned(),
                format: None,
                formatted_body: None,
                relates_to: None,
            }),
        })
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ruma_client::Error> {
    let (homeserver_url, room) = match (env::args().nth(1), env::args().nth(2)) {
        (Some(a), Some(b)) => (a, b),
        _ => {
            eprintln!(
                "Usage: {} <homeserver_url> <room>",
                env::args().next().unwrap()
            );
            exit(1)
        }
    };

    hello_world(homeserver_url.parse().unwrap(), room).await
}
