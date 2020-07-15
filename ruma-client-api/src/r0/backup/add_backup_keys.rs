//! [PUT /_matrix/client/r0/room_keys/keys](https://matrix.org/docs/spec/client_server/unstable#put-matrix-client-r0-room-keys-keys)

use std::collections::BTreeMap;

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

use super::KeyData;

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
        ///
        /// Note: synapse has the `sessions: {}` wrapper, the Matrix spec does not.
        pub rooms: BTreeMap<RoomId, Sessions>,
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

// TODO: remove
/// A wrapper around a mapping of session IDs to key data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sessions {
    // TODO: remove
    ///  A map of session IDs to key data.
    pub sessions: BTreeMap<String, KeyData>,
}
