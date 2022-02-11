use std::time::Duration;

use assign::assign;
use async_stream::try_stream;
use futures_core::stream::Stream;
use ruma_api::MatrixVersion;
use ruma_client_api::r0::{
    account::register::{self, RegistrationKind},
    session::login::{self, LoginInfo},
    sync::sync_events,
    uiaa::UserIdentifier,
};
use ruma_common::presence::PresenceState;
use ruma_identifiers::DeviceId;

use super::{Client, Error, HttpClient};

/// Client-API specific functionality of `Client`.
impl<C: HttpClient> Client<C> {
    /// Log in with a username and password.
    ///
    /// In contrast to [`send_request`][Self::send_request], this method stores the access token
    /// returned by the endpoint in this client, in addition to returning it.
    pub async fn log_in(
        &self,
        user: &str,
        password: &str,
        device_id: Option<&DeviceId>,
        initial_device_display_name: Option<&str>,
    ) -> Result<login::Response, Error<C::Error, ruma_client_api::Error>> {
        let response = self
            .send_request(assign!(login::Request::new(
                LoginInfo::Password(login::Password::new(UserIdentifier::MatrixId(user), password))), {
                device_id,
                initial_device_display_name,
                }
            ), &[MatrixVersion::V1_0])
            .await?;

        *self.0.access_token.lock().unwrap() = Some(response.access_token.clone());

        Ok(response)
    }

    /// Register as a guest.
    ///
    /// In contrast to [`send_request`][Self::send_request], this method stores the access token
    /// returned by the endpoint in this client, in addition to returning it.
    pub async fn register_guest(
        &self,
    ) -> Result<register::Response, Error<C::Error, ruma_client_api::r0::uiaa::UiaaResponse>> {
        let response = self
            .send_request(
                assign!(register::Request::new(), { kind: RegistrationKind::Guest }),
                &[MatrixVersion::V1_0],
            )
            .await?;

        *self.0.access_token.lock().unwrap() = response.access_token.clone();

        Ok(response)
    }

    /// Register as a new user on this server.
    ///
    /// In contrast to [`send_request`][Self::send_request], this method stores the access token
    /// returned by the endpoint in this client, in addition to returning it.
    ///
    /// The username is the local part of the returned user_id. If it is omitted from this request,
    /// the server will generate one.
    pub async fn register_user(
        &self,
        username: Option<&str>,
        password: &str,
    ) -> Result<register::Response, Error<C::Error, ruma_client_api::r0::uiaa::UiaaResponse>> {
        let response = self
            .send_request(
                assign!(register::Request::new(), { username, password: Some(password)}),
                &[MatrixVersion::V1_0],
            )
            .await?;

        *self.0.access_token.lock().unwrap() = response.access_token.clone();

        Ok(response)
    }

    /// Convenience method that represents repeated calls to the sync_events endpoint as a stream.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// # type MatrixClient = ruma_client::Client<ruma_client::http_client::Dummy>;
    /// # use ruma_common::presence::PresenceState;
    /// # use tokio_stream::{StreamExt as _};
    /// # let homeserver_url = "https://example.com".parse().unwrap();
    /// # let client = MatrixClient::new(homeserver_url, None);
    /// # let next_batch_token = String::new();
    /// # async {
    /// let mut sync_stream = Box::pin(client.sync(
    ///     None,
    ///     next_batch_token,
    ///     &PresenceState::Online,
    ///     Some(Duration::from_secs(30)),
    /// ));
    /// while let Some(response) = sync_stream.try_next().await? {
    ///     // Do something with the data in the response...
    /// }
    /// # Result::<(), ruma_client::Error<_, _>>::Ok(())
    /// # };
    /// ```
    pub fn sync<'a>(
        &'a self,
        filter: Option<&'a sync_events::Filter<'a>>,
        mut since: String,
        set_presence: &'a PresenceState,
        timeout: Option<Duration>,
    ) -> impl Stream<Item = Result<sync_events::Response, Error<C::Error, ruma_client_api::Error>>> + 'a
    {
        try_stream! {
            loop {
                let response = self
                    .send_request(assign!(sync_events::Request::new(), {
                        filter,
                        since: Some(&since),
                        set_presence,
                        timeout,
                    }), &[MatrixVersion::V1_0])
                    .await?;

                since = response.next_batch.clone();
                yield response;
            }
        }
    }
}
