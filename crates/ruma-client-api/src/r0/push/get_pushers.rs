//! [GET /_matrix/client/r0/pushers](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-pushers)

use ruma_api::ruma_api;
use serde::{Deserialize, Serialize};

use super::{PusherData, PusherKind};

ruma_api! {
    metadata: {
        description: "Gets all currently active pushers for the authenticated user.",
        method: GET,
        name: "get_pushers",
        path: "/_matrix/client/r0/pushers",
        rate_limited: false,
        authentication: AccessToken,
    }

    #[derive(Default)]
    request: {}

    response: {
        /// An array containing the current pushers for the user.
        pub pushers: Vec<Pusher>
    }

    error: crate::Error
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self
    }
}

impl Response {
    /// Creates a new `Response` with the given pushers.
    pub fn new(pushers: Vec<Pusher>) -> Self {
        Self { pushers }
    }
}

/// Defines a pusher.
///
/// To create an instance of this type, first create a `PusherInit` and convert it via
/// `Pusher::from` / `.into()`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Pusher {
    /// This is a unique identifier for this pusher. Max length, 512 bytes.
    pub pushkey: String,

    /// The kind of the pusher.
    pub kind: PusherKind,

    /// This is a reverse-DNS style identifier for the application. Max length, 64 chars.
    pub app_id: String,

    /// A string that will allow the user to identify what application owns this pusher.
    pub app_display_name: String,

    /// A string that will allow the user to identify what device owns this pusher.
    pub device_display_name: String,

    /// This string determines which set of device specific rules this pusher executes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_tag: Option<String>,

    /// The preferred language for receiving notifications (e.g. 'en' or 'en-US')
    pub lang: String,

    /// Information for the pusher implementation itself.
    pub data: PusherData,
}

/// Initial set of fields of `Pusher`.
///
/// This struct will not be updated even if additional fields are added to `Pusher` in a new
/// (non-breaking) release of the Matrix specification.
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct PusherInit {
    /// This is a unique identifier for this pusher. Max length, 512 bytes.
    pub pushkey: String,

    /// The kind of the pusher. `None` deletes the pusher.
    pub kind: PusherKind,

    /// This is a reverse-DNS style identifier for the application. Max length, 64 chars.
    pub app_id: String,

    /// A string that will allow the user to identify what application owns this pusher.
    pub app_display_name: String,

    /// A string that will allow the user to identify what device owns this pusher.
    pub device_display_name: String,

    /// This string determines which set of device specific rules this pusher executes.
    pub profile_tag: Option<String>,

    /// The preferred language for receiving notifications (e.g. 'en' or 'en-US')
    pub lang: String,

    /// Information for the pusher implementation itself.
    pub data: PusherData,
}

impl From<PusherInit> for Pusher {
    fn from(init: PusherInit) -> Self {
        let PusherInit {
            pushkey,
            kind,
            app_id,
            app_display_name,
            device_display_name,
            profile_tag,
            lang,
            data,
        } = init;
        Self {
            pushkey,
            kind,
            app_id,
            app_display_name,
            device_display_name,
            profile_tag,
            lang,
            data,
        }
    }
}
