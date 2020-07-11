//! [GET /_matrix/client/r0/room_keys/keys](https://matrix.org/docs/spec/client_server/unstable#get-matrix-client-r0-room-keys-keys)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Retrieve all keys from a backup.",
        method: GET,
        name: "get_backup_keys",
        path: "/_matrix/client/r0/room_keys/keys",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The backup version. Must be the current backup.
        #[ruma_api(query)]
        pub version: String,
    }

    response: {
        /// A map from room IDs to session IDs to key data.
        pub rooms: BTreeMap<RoomId, Sessions>, // TODO: synapse has the sessions:{} wrapper, the spec has not
    }

    error: crate::Error
}

/// TODO: remove
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sessions {
    /// TODO: remove
    pub sessions: BTreeMap<String, super::KeyData>,
}
