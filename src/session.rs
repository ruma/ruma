use ruma_identifiers::UserId;

/// A user session, containing an access token and information about the associated user account.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Session {
    /// The access token used for this session.
    access_token: String,
    /// The user the access token was issued for.
    user_id: UserId,
}

impl Session {
    /// Create a new user session from an access token and a user ID.
    pub fn new(access_token: String, user_id: UserId) -> Self {
        Session {
            access_token,
            user_id,
        }
    }

    /// Get the access token associated with this session.
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Get the ID of the user the session belongs to.
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }
}
