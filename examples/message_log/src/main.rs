use std::{env, process::exit, time::Duration};

use assign::assign;
use ruma::{
    api::client::{filter::FilterDefinition, sync::sync_events},
    events::{
        room::message::{MessageType, RoomMessageEventContent, TextMessageEventContent},
        AnySyncMessageLikeEvent, AnySyncRoomEvent, OriginalSyncMessageLikeEvent,
        SyncMessageLikeEvent,
    },
    presence::PresenceState,
};
use tokio_stream::StreamExt as _;

type HttpClient = ruma::client::http_client::HyperNativeTls;

async fn log_messages(
    homeserver_url: String,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let client =
        ruma::Client::builder().homeserver_url(homeserver_url).build::<HttpClient>().await?;

    client.log_in(username, password, None, None).await?;

    let filter = FilterDefinition::ignore_all().into();
    let initial_sync_response = client
        .send_request(assign!(sync_events::v3::Request::new(), {
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
                if let AnySyncRoomEvent::MessageLike(AnySyncMessageLikeEvent::RoomMessage(
                    SyncMessageLikeEvent::Original(OriginalSyncMessageLikeEvent {
                        content:
                            RoomMessageEventContent {
                                msgtype:
                                    MessageType::Text(TextMessageEventContent {
                                        body: msg_body, ..
                                    }),
                                ..
                            },
                        sender,
                        ..
                    }),
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

    log_messages(homeserver_url, &username, &password).await
}
