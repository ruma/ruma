//! [GET /_matrix/client/r0/room_keys/keys](https://matrix.org/docs/spec/client_server/unstable#get-matrix-client-r0-room-keys-keys)

use std::collections::BTreeMap;

use ruma_api::ruma_api;
use ruma_identifiers::RoomId;

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
        ///
        /// Note: synapse has the `sessions: {}` wrapper, the Matrix spec does not.
        #[cfg(feature = "unstable-synapse-quirks")]
        pub rooms: BTreeMap<RoomId, super::Sessions>,

        /// A map from room IDs to session IDs to key data.
        #[cfg(not(feature = "unstable-synapse-quirks"))]
        pub rooms: BTreeMap<RoomId, BTreeMap<String, super::KeyData>>,
    }

    error: crate::Error
}
