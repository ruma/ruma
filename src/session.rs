use ruma_identifiers::UserId;
use url::Host;

/// An active user session with a Matrix homeserver, allowing authenticated requests.
#[derive(Clone, Debug)]
pub struct Session {
    /// The access token of this session
    pub access_token: String,
    /// The homeserver this session is associated with
    pub homeserver: Host,
    /// the ID of the user owning this session
    pub user_id: UserId,
}
