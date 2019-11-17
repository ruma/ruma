//! [GET /_matrix/client/r0/presence/list/{userId}](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-presence-list-userid)

use ruma_api::ruma_api;
use ruma_events::{presence::PresenceEvent, EventResult};
use ruma_identifiers::UserId;

ruma_api! {
    metadata {
        description: "Get the precence status from the user's subscriptions.",
        method: GET,
        name: "get_subscribed_presences",
        path: "/_matrix/client/r0/presence/list/:user_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The user whose presence state will be retrieved.
        #[ruma_api(path)]
        pub user_id: UserId,
    }

    response {
        /// A list of presence events for every user on this list.
        #[ruma_api(body)]
        #[wrap_incoming(PresenceEvent with EventResult)]
        pub presence_events: Vec<PresenceEvent>,
    }
}
