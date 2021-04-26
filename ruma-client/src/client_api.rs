use std::time::Duration;

use assign::assign;
use async_stream::try_stream;
use futures_core::stream::Stream;
use ruma_client_api::r0::sync::sync_events::{
    Filter as SyncFilter, Request as SyncRequest, Response as SyncResponse,
};
use ruma_common::presence::PresenceState;
use ruma_identifiers::DeviceId;

use super::{Client, Error, Identification, Session};

impl Client {
    /// Log in with a username and password.
    ///
    /// In contrast to `api::r0::session::login::call()`, this method stores the
    /// session data returned by the endpoint in this client, instead of
    /// returning it.
    pub async fn log_in(
        &self,
        user: &str,
        password: &str,
        device_id: Option<&DeviceId>,
        initial_device_display_name: Option<&str>,
    ) -> Result<Session, Error<ruma_client_api::Error>> {
        use ruma_client_api::r0::session::login::{
            LoginInfo, Request as LoginRequest, UserIdentifier,
        };

        let response = self
            .request(assign!(
                LoginRequest::new(
                    LoginInfo::Password { identifier: UserIdentifier::MatrixId(user), password }
                ), {
                    device_id,
                    initial_device_display_name,
                }
            ))
            .await?;

        let session = Session {
            access_token: response.access_token,
            identification: Some(Identification {
                device_id: response.device_id,
                user_id: response.user_id,
            }),
        };
        *self.0.session.lock().unwrap() = Some(session.clone());

        Ok(session)
    }

    /// Register as a guest. In contrast to `api::r0::account::register::call()`,
    /// this method stores the session data returned by the endpoint in this
    /// client, instead of returning it.
    pub async fn register_guest(
        &self,
    ) -> Result<Session, Error<ruma_client_api::r0::uiaa::UiaaResponse>> {
        use ruma_client_api::r0::account::register::{self, RegistrationKind};

        let response = self
            .request(assign!(register::Request::new(), { kind: RegistrationKind::Guest }))
            .await?;

        let session = Session {
            // since we supply inhibit_login: false above, the access token needs to be there
            // TODO: maybe unwrap is not the best solution though
            access_token: response.access_token.unwrap(),
            identification: Some(Identification {
                // same as access_token
                device_id: response.device_id.unwrap(),
                user_id: response.user_id,
            }),
        };
        *self.0.session.lock().unwrap() = Some(session.clone());

        Ok(session)
    }

    /// Register as a new user on this server.
    ///
    /// In contrast to `api::r0::account::register::call()`, this method stores
    /// the session data returned by the endpoint in this client, instead of
    /// returning it.
    ///
    /// The username is the local part of the returned user_id. If it is
    /// omitted from this request, the server will generate one.
    pub async fn register_user(
        &self,
        username: Option<&str>,
        password: &str,
    ) -> Result<Session, Error<ruma_client_api::r0::uiaa::UiaaResponse>> {
        use ruma_client_api::r0::account::register;

        let response = self
            .request(assign!(register::Request::new(), { username, password: Some(password) }))
            .await?;

        let session = Session {
            // since we supply inhibit_login: false above, the access token needs to be there
            // TODO: maybe unwrap is not the best solution though
            access_token: response.access_token.unwrap(),
            identification: Some(Identification {
                // same as access_token
                device_id: response.device_id.unwrap(),
                user_id: response.user_id,
            }),
        };
        *self.0.session.lock().unwrap() = Some(session.clone());

        Ok(session)
    }

    /// Convenience method that represents repeated calls to the sync_events endpoint as a stream.
    pub fn sync<'a>(
        &self,
        filter: Option<&'a SyncFilter<'a>>,
        mut since: String,
        set_presence: &'a PresenceState,
        timeout: Option<Duration>,
    ) -> impl Stream<Item = Result<SyncResponse, Error<ruma_client_api::Error>>> + 'a {
        let client = self.clone();
        try_stream! {
            loop {
                let response = client
                    .request(assign!(SyncRequest::new(), {
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
