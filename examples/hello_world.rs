#![feature(conservative_impl_trait)]
#![feature(try_from)]

extern crate futures;
extern crate hyper;
extern crate ruma_client;
extern crate ruma_events;
extern crate ruma_identifiers;
extern crate tokio_core;
extern crate url;

use std::convert::TryFrom;

use futures::Future;
use hyper::client::Connect;
use ruma_client::Client;
use ruma_client::api::r0;
use ruma_events::EventType;
use ruma_events::room::message::{MessageEventContent, MessageType, TextMessageEventContent};
use ruma_identifiers::RoomAliasId;
use tokio_core::reactor::Core;
use url::Url;

fn hello_world<'a, C: Connect>(client: &'a Client<C>)
-> impl Future<Item = (), Error = ruma_client::Error> + 'a
{
    client.register_guest().and_then(move |_| {
        r0::alias::get_alias::call(client, r0::alias::get_alias::Request {
            room_alias: RoomAliasId::try_from("#ruma-client-test:matrix.org").unwrap(),
        })
    }).and_then(move |response| {
        let room_id = response.room_id;

        r0::membership::join_room_by_id::call(client, r0::membership::join_room_by_id::Request {
            room_id: room_id.clone(),
            third_party_signed: None,
        }).and_then(move |_| {
            r0::send::send_message_event::call(client, r0::send::send_message_event::Request {
                room_id: room_id,
                event_type: EventType::RoomMessage,
                txn_id: "1".to_owned(),
                data: MessageEventContent::Text(TextMessageEventContent {
                    body: "Hello World!".to_owned(),
                    msgtype: MessageType::Text,
                }),
            })
        })
    }).map(|_| ())
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let server = Url::parse("https://matrix.org/").unwrap();

    let client = Client::https(&handle, server, None).unwrap();
    core.run(hello_world(&client)).unwrap();
}
