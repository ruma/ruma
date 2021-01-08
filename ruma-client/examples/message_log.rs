use std::{env, process::exit, time::Duration};

use assign::assign;
use http::Uri;
use ruma::{
    api::client::r0::{filter::FilterDefinition, sync::sync_events},
    events::{
        room::message::{MessageEventContent, TextMessageEventContent},
        AnySyncMessageEvent, AnySyncRoomEvent, SyncMessageEvent,
    },
    presence::PresenceState,
};
use ruma_client::{self, Client};
use tokio_stream::StreamExt as _;

async fn log_messages(homeserver_url: Uri, username: &str, password: &str) -> anyhow::Result<()> {
    let client = Client::new(homeserver_url, None);

    client.log_in(username, password, None, None).await?;

    let filter = FilterDefinition::ignore_all().into();
    let initial_sync_response = client
        .request(assign!(sync_events::Request::new(), {
            filter: Some(&filter),
        }))
        .await?;

    let mut sync_stream = Box::pin(client.sync(
        None,
        initial_sync_response.next_batch,
        &PresenceState::Online,
        Some(Duration::from_secs(30)),
    ));

    while let Some(res) = sync_stream.try_next().await? {
        // Only look at rooms the user hasn't left yet
        for (room_id, room) in res.rooms.join {
            for event in room.timeline.events.into_iter().flat_map(|r| r.deserialize()) {
                // Filter out the text messages
                if let AnySyncRoomEvent::Message(AnySyncMessageEvent::RoomMessage(
                    SyncMessageEvent {
                        content:
                            MessageEventContent::Text(TextMessageEventContent {
                                body: msg_body, ..
                            }),
                        sender,
                        ..
                    },
                )) = event
                {
                    println!("{:?} in {:?}: {}", sender, room_id, msg_body);
                }
            }
        }
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let (homeserver_url, username, password) =
        match (env::args().nth(1), env::args().nth(2), env::args().nth(3)) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => {
                eprintln!(
                    "Usage: {} <homeserver_url> <username> <password>",
                    env::args().next().unwrap()
                );
                exit(1)
            }
        };

    let server = homeserver_url.parse()?;
    log_messages(server, &username, &password).await
}
