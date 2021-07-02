//! User sessions.

use ruma_identifiers::{DeviceId, DeviceIdBox, UserId};

/// A user session, containing an access token and information about the associated user account.
#[derive(Clone, Debug, serde::Deserialize, Eq, Hash, PartialEq, serde::Serialize)]
pub struct Session {
    /// The access token used for this session.
    pub access_token: String,

    /// Identification information for a user
    pub identification: Option<Identification>,
}

/// The identification information about the associated user account if the session is associated
/// with a single user account.
#[derive(Clone, Debug, serde::Deserialize, Eq, Hash, PartialEq, serde::Serialize)]
pub struct Identification {
    /// The user the access token was issued for.
    pub user_id: UserId,

    /// The ID of the client device
    pub device_id: DeviceIdBox,
}

impl Session {
    /// Create a new user session from an access token and a user ID.
    #[deprecated]
    pub fn new(access_token: String, user_id: UserId, device_id: DeviceIdBox) -> Self {
        Self { access_token, identification: Some(Identification { user_id, device_id }) }
    }
}
