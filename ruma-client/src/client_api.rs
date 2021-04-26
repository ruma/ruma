use std::time::Duration;

use assign::assign;
use async_stream::try_stream;
use futures_core::stream::Stream;
use ruma_client_api::r0::{
    account::register::{self, RegistrationKind},
    session::login::{self, LoginInfo, UserIdentifier},
    sync::sync_events,
};
use ruma_common::presence::PresenceState;
use ruma_identifiers::DeviceId;

use super::{Client, Error};

/// Client-API specific functionality of `Client`.
impl Client {
    /// Log in with a username and password.
    ///
    /// In contrast to [`request`], this method stores the access token returned by the endpoint in
    /// this client, in addition to returning it.
    pub async fn log_in(
        &self,
        user: &str,
        password: &str,
        device_id: Option<&DeviceId>,
        initial_device_display_name: Option<&str>,
    ) -> Result<login::Response, Error<ruma_client_api::Error>> {
        let response = self
            .request(assign!(
                login::Request::new(
                    LoginInfo::Password { identifier: UserIdentifier::MatrixId(user), password }
                ), {
                    device_id,
                    initial_device_display_name,
                }
            ))
            .await?;

        *self.0.access_token.lock().unwrap() = Some(response.access_token.clone());

        Ok(response)
    }

    /// Register as a guest.
    ///
    /// In contrast to [`request`], this method stores the access token returned by the endpoint in
    /// this client, in addition to returning it.
    pub async fn register_guest(
        &self,
    ) -> Result<register::Response, Error<ruma_client_api::r0::uiaa::UiaaResponse>> {
        let response = self
            .request(assign!(register::Request::new(), { kind: RegistrationKind::Guest }))
            .await?;

        *self.0.access_token.lock().unwrap() = response.access_token.clone();

        Ok(response)
    }

    /// Register as a new user on this server.
    ///
    /// In contrast to [`request`], this method stores the access token returned by the endpoint in
    /// this client, in addition to returning it.
    ///
    /// The username is the local part of the returned user_id. If it is omitted from this request,
    /// the server will generate one.
    pub async fn register_user(
        &self,
        username: Option<&str>,
        password: &str,
    ) -> Result<register::Response, Error<ruma_client_api::r0::uiaa::UiaaResponse>> {
        let response = self
            .request(assign!(register::Request::new(), { username, password: Some(password) }))
            .await?;

        *self.0.access_token.lock().unwrap() = response.access_token.clone();

        Ok(response)
    }

    /// Convenience method that represents repeated calls to the sync_events endpoint as a stream.
    pub fn sync<'a>(
        &self,
        filter: Option<&'a sync_events::Filter<'a>>,
        mut since: String,
        set_presence: &'a PresenceState,
        timeout: Option<Duration>,
    ) -> impl Stream<Item = Result<sync_events::Response, Error<ruma_client_api::Error>>> + 'a {
        let client = self.clone();
        try_stream! {
            loop {
                let response = client
                    .request(assign!(sync_events::Request::new(), {
                        filter,
                        since: Some(&since),
                        set_presence,
                        timeout,
                    }))
                    .await?;

                since = response.next_batch.clone();
                yield response;
            }
        }
    }
}
