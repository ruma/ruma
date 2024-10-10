//! `/v4/` ([MSC])
//!
//! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3983

use std::{collections::BTreeMap, time::Duration};

use ruma_common::{
    api::{request, response, Metadata},
    encryption::OneTimeKey,
    metadata,
    serde::Raw,
    OneTimeKeyAlgorithm, OwnedDeviceId, OwnedOneTimeKeyId, OwnedUserId,
};
use serde_json::Value as JsonValue;

const METADATA: Metadata = metadata! {
    method: POST,
    rate_limited: false,
    authentication: AccessToken,
    history: {
        unstable => "/_matrix/client/unstable/org.matrix.msc3983/keys/claim",
    }
};

/// Request type for the `claim_keys` endpoint.
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
    pub one_time_keys: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, Vec<OneTimeKeyAlgorithm>>>,
}

/// Response type for the `claim_keys` endpoint.
#[response(error = crate::Error)]
pub struct Response {
    /// If any remote homeservers could not be reached, they are recorded here.
    ///
    /// The names of the properties are the names of the unreachable servers.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub failures: BTreeMap<String, JsonValue>,

    /// One-time keys for the queried devices.
    pub one_time_keys: BTreeMap<OwnedUserId, OneTimeKeys>,
}

impl Request {
    /// Creates a new `Request` with the given key claims and the recommended 10 second timeout.
    pub fn new(
        one_time_keys: BTreeMap<OwnedUserId, BTreeMap<OwnedDeviceId, Vec<OneTimeKeyAlgorithm>>>,
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
pub type OneTimeKeys = BTreeMap<OwnedDeviceId, BTreeMap<OwnedOneTimeKeyId, Raw<OneTimeKey>>>;
