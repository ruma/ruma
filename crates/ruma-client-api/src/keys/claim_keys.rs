//! `POST /_matrix/client/*/keys/claim`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3keysclaim

    use std::{collections::BTreeMap, time::Duration};

    use ruma_common::{
        api::{request, response, Metadata},
        encryption::OneTimeKey,
        metadata,
        serde::Raw,
        DeviceKeyAlgorithm, OwnedDeviceId, OwnedDeviceKeyId, OwnedUserId,
    };
    use serde_json::Value as JsonValue;

    const METADATA: Metadata = metadata! {
        description: "Claims one-time keys for use in pre-key messages.",
        method: POST,
        name: "claim_keys",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/keys/claim",
            1.1 => "/_matrix/client/v3/keys/claim",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request {
        /// The time (in milliseconds) to wait when downloading keys from remote servers.
        /// 10 seconds is the recommended default.
        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub timeout: Option<Duration>,

        /// The keys to be claimed.
        pub one_time_keys: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, DeviceKeyAlgorithm>>,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// If any remote homeservers could not be reached, they are recorded here.
        /// The names of the properties are the names of the unreachable servers.
        pub failures: BTreeMap<String, JsonValue>,

        /// One-time keys for the queried devices.
        pub one_time_keys: BTreeMap<OwnedUserId, OneTimeKeys>,
    }

    impl Request {
        /// Creates a new `Request` with the given key claims and the recommended 10 second timeout.
        pub fn new(
            one_time_keys: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, DeviceKeyAlgorithm>>,
        ) -> Self {
            Self { timeout: Some(Duration::from_secs(10)), one_time_keys }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given keys and no failures.
        pub fn new(one_time_keys: BTreeMap<OwnedUserId, OneTimeKeys>) -> Self {
            Self { failures: BTreeMap::new(), one_time_keys }
        }
    }

    /// The one-time keys for a given device.
    pub type OneTimeKeys = BTreeMap<OwnedDeviceId, BTreeMap<OwnedDeviceKeyId, Raw<OneTimeKey>>>;
}
