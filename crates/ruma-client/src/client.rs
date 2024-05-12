use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use assign::assign;
use async_stream::try_stream;
use futures_core::stream::Stream;
use ruma_client_api::{
    account::register::{self, RegistrationKind},
    session::login::{self, v3::LoginInfo},
    sync::sync_events,
    uiaa::UserIdentifier,
};
use ruma_common::{
    api::{MatrixVersion, OutgoingRequest, SendAccessToken},
    presence::PresenceState,
    DeviceId, UserId,
};

use crate::{
    add_user_id_to_query, send_customized_request, Error, HttpClient, ResponseError, ResponseResult,
};

mod builder;

pub use self::builder::ClientBuilder;

/// A client for the Matrix client-server API.
#[derive(Clone, Debug)]
pub struct Client<C>(Arc<ClientData<C>>);

/// Data contained in Client's Rc
#[derive(Debug)]
struct ClientData<C> {
    /// The URL of the homeserver to connect to.
    homeserver_url: String,

    /// The underlying HTTP client.
    http_client: C,

    /// The access token, if logged in.
    access_token: Mutex<Option<String>>,

    /// The (known) Matrix versions the homeserver supports.
    supported_matrix_versions: Vec<MatrixVersion>,
}

impl Client<()> {
    /// Creates a new client builder.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

impl<C> Client<C> {
    /// Get a copy of the current `access_token`, if any.
    ///
    /// Useful for serializing and persisting the session to be restored later.
    pub fn access_token(&self) -> Option<String> {
        self.0.access_token.lock().expect("session mutex was poisoned").clone()
    }
}

impl<C: HttpClient> Client<C> {
    /// Makes a request to a Matrix API endpoint.
    pub async fn send_request<R: OutgoingRequest>(&self, request: R) -> ResponseResult<C, R> {
        self.send_customized_request(request, |_| Ok(())).await
    }

    /// Makes a request to a Matrix API endpoint including additional URL parameters.
    pub async fn send_customized_request<R, F>(
        &self,
        request: R,
        customize: F,
    ) -> ResponseResult<C, R>
    where
        R: OutgoingRequest,
        F: FnOnce(&mut http::Request<C::RequestBody>) -> Result<(), ResponseError<C, R>>,
    {
        let access_token = self.access_token();
        let send_access_token = match access_token.as_deref() {
            Some(at) => SendAccessToken::IfRequired(at),
            None => SendAccessToken::None,
        };

        send_customized_request(
            &self.0.http_client,
            &self.0.homeserver_url,
            send_access_token,
            &self.0.supported_matrix_versions,
            request,
            customize,
        )
        .await
    }

    /// Makes a request to a Matrix API endpoint as a virtual user.
    ///
    /// This method is meant to be used by application services when interacting with the
    /// client-server API.
    pub async fn send_request_as<R: OutgoingRequest>(
        &self,
        user_id: &UserId,
        request: R,
    ) -> ResponseResult<C, R> {
        self.send_customized_request(request, add_user_id_to_query::<C, R>(user_id)).await
    }

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
    ) -> Result<login::v3::Response, Error<C::Error, ruma_client_api::Error>> {
        let login_info = LoginInfo::Password(login::v3::Password::new(
            UserIdentifier::UserIdOrLocalpart(user.to_owned()),
            password.to_owned(),
        ));
        let response = self
            .send_request(assign!(login::v3::Request::new(login_info), {
                device_id: device_id.map(ToOwned::to_owned),
                initial_device_display_name: initial_device_display_name.map(ToOwned::to_owned),
            }))
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
    ) -> Result<register::v3::Response, Error<C::Error, ruma_client_api::uiaa::UiaaResponse>> {
        let response = self
            .send_request(assign!(register::v3::Request::new(), { kind: RegistrationKind::Guest }))
            .await?;

        self.0.access_token.lock().unwrap().clone_from(&response.access_token);

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
    ) -> Result<register::v3::Response, Error<C::Error, ruma_client_api::uiaa::UiaaResponse>> {
        let response = self
            .send_request(assign!(register::v3::Request::new(), {
                username: username.map(ToOwned::to_owned),
                password: Some(password.to_owned())
            }))
            .await?;

        self.0.access_token.lock().unwrap().clone_from(&response.access_token);

        Ok(response)
    }

    /// Convenience method that represents repeated calls to the sync_events endpoint as a stream.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// # use ruma_common::presence::PresenceState;
    /// # use tokio_stream::{StreamExt as _};
    /// # let homeserver_url = "https://example.com".to_owned();
    /// # async {
    /// # let client = ruma_client::Client::builder()
    /// #     .homeserver_url(homeserver_url)
    /// #     .build::<ruma_client::http_client::Dummy>()
    /// #     .await?;
    /// # let next_batch_token = String::new();
    /// let mut sync_stream = Box::pin(client.sync(
    ///     None,
    ///     next_batch_token,
    ///     PresenceState::Online,
    ///     Some(Duration::from_secs(30)),
    /// ));
    /// while let Some(response) = sync_stream.try_next().await? {
    ///     // Do something with the data in the response...
    /// }
    /// # Result::<(), ruma_client::Error<_, _>>::Ok(())
    /// # };
    /// ```
    pub fn sync(
        &self,
        filter: Option<sync_events::v3::Filter>,
        mut since: String,
        set_presence: PresenceState,
        timeout: Option<Duration>,
    ) -> impl Stream<Item = Result<sync_events::v3::Response, Error<C::Error, ruma_client_api::Error>>>
           + '_ {
        try_stream! {
            loop {
                let response = self
                    .send_request(assign!(sync_events::v3::Request::new(), {
                        filter: filter.clone(),
                        since: Some(since.clone()),
                        set_presence: set_presence.clone(),
                        timeout,
                    }))
                    .await?;

                since.clone_from(&response.next_batch);
                yield response;
            }
        }
    }
}
