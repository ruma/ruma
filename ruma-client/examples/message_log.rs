use std::{env, process::exit, time::Duration};

use futures_util::stream::{StreamExt as _, TryStreamExt as _};
use ruma_client::{
    self,
    events::room::message::{MessageEventContent, TextMessageEventContent},
    HttpClient,
};
use ruma_common::presence::PresenceState;
use ruma_events::{AnySyncMessageEvent, AnySyncRoomEvent, SyncMessageEvent};
use url::Url;

async fn log_messages(
    homeserver_url: Url,
    username: String,
    password: String,
) -> anyhow::Result<()> {
    let client = HttpClient::new(homeserver_url, None);

    client.log_in(username, password, None, None).await?;

    let mut sync_stream = Box::pin(
        client
            .sync(
                None,
                None,
                PresenceState::Online,
                Some(Duration::from_secs(30)),
            )
            // TODO: This is a horrible way to obtain an initial next_batch token that generates way
            //       too much server load and network traffic. Fix this!
            .skip(1),
    );

    while let Some(res) = sync_stream.try_next().await? {
        // Only look at rooms the user hasn't left yet
        for (room_id, room) in res.rooms.join {
            for event in room
                .timeline
                .events
                .into_iter()
                .flat_map(|r| r.deserialize())
            {
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

#[tokio::main]
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

    let server = Url::parse(&homeserver_url).unwrap();
    log_messages(server, username, password).await
}
