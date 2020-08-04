//! User sessions.

use ruma_identifiers::{DeviceId, UserId};

/// A user session, containing an access token and information about the associated user account.
#[derive(Clone, Debug, serde::Deserialize, Eq, Hash, PartialEq, serde::Serialize)]
pub struct Session {
    /// The access token used for this session.
    pub access_token: String,
    /// Identification information for a user
    pub identification: Option<Identification>,
}

/// The identification information about the associated user account if the session is associated with
/// a single user account.
#[derive(Clone, Debug, serde::Deserialize, Eq, Hash, PartialEq, serde::Serialize)]
pub struct Identification {
    /// The user the access token was issued for.
    pub user_id: UserId,
    /// The ID of the client device
    pub device_id: Box<DeviceId>,
}

impl Session {
    /// Create a new user session from an access token and a user ID.
    #[deprecated]
    pub fn new(access_token: String, user_id: UserId, device_id: Box<DeviceId>) -> Self {
        Self {
            access_token,
            identification: Some(Identification { user_id, device_id }),
        }
    }

    /// Get the access token associated with this session.
    #[deprecated]
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Get the ID of the user the session belongs to.
    #[deprecated]
    pub fn user_id(&self) -> Option<&UserId> {
        if let Some(identification) = &self.identification {
            return Some(&identification.user_id);
        }
        None
    }

    /// Get ID of the device the session belongs to.
    #[deprecated]
    pub fn device_id(&self) -> Option<&DeviceId> {
        if let Some(identification) = &self.identification {
            return Some(&identification.device_id);
        }
        None
    }
}
