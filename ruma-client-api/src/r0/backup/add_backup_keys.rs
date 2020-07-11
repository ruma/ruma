//! [PUT /_matrix/client/r0/room_keys/keys](https://matrix.org/docs/spec/client_server/unstable#put-matrix-client-r0-room-keys-keys)

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Store several keys in the backup.",
        method: PUT,
        name: "add_backup_keys",
        path: "/_matrix/client/r0/room_keys/keys",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The backup version. Must be the current backup.
        #[ruma_api(query)]
        pub version: String,
        /// A map from room IDs to session IDs to key data.
        pub rooms: BTreeMap<RoomId, Sessions>, // TODO: synapse has the sessions:{} wrapper, the spec has not
    }

    response: {
        /// An opaque string representing stored keys in the backup. Clients can compare it with
        /// the etag value they received in the request of their last key storage request.
        pub etag: String,
        /// The number of keys stored in the backup.
        pub count: UInt,
    }

    error: crate::Error
}

/// TODO: remove
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sessions {
    /// TODO: remove
    pub sessions: BTreeMap<String, super::KeyData>,
}
