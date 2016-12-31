use ruma_identifiers::UserId;
use url::Host;

/// An active user session with a Matrix homeserver, allowing authenticated requests.
#[derive(Clone, Debug)]
pub struct Session {
    access_token: String,
    homeserver: Host,
    user_id: UserId,
}
