use std::sync::{Arc, Mutex};

use ruma_client_api::discovery::get_supported_versions;
use ruma_common::api::{MatrixVersion, SendAccessToken};

use super::{Client, ClientData};
use crate::{DefaultConstructibleHttpClient, Error, HttpClient, HttpClientExt};

/// A [`Client`] builder.
///
/// This type can be used to construct a `Client` through a few method calls.
pub struct ClientBuilder {
    homeserver_url: Option<String>,
    access_token: Option<String>,
    supported_matrix_versions: Option<Vec<MatrixVersion>>,
}

impl ClientBuilder {
    pub(super) fn new() -> Self {
        Self { homeserver_url: None, access_token: None, supported_matrix_versions: None }
    }

    /// Set the homeserver URL.
    ///
    /// The homeserver URL must be set before calling [`build()`][Self::build] or
    /// [`http_client()`][Self::http_client].
    pub fn homeserver_url(self, url: String) -> Self {
        Self { homeserver_url: Some(url), ..self }
    }

    /// Set the access token.
    pub fn access_token(self, access_token: Option<String>) -> Self {
        Self { access_token, ..self }
    }

    /// Set the supported Matrix versions.
    ///
    /// This method generally *shouldn't* be called. The [`build()`][Self::build] or
    /// [`http_client()`][Self::http_client] method will take care of doing a
    /// [`get_supported_versions`] request to find out about the supported versions.
    pub fn supported_matrix_versions(self, versions: Vec<MatrixVersion>) -> Self {
        Self { supported_matrix_versions: Some(versions), ..self }
    }

    /// Finish building the [`Client`].
    ///
    /// Uses [`DefaultConstructibleHttpClient::default()`] to create an HTTP client instance.
    /// Unless the supported Matrix versions were manually set via
    /// [`supported_matrix_versions`][Self::supported_matrix_versions], this will do a
    /// [`get_supported_versions`] request to find out about the supported versions.
    pub async fn build<C>(self) -> Result<Client<C>, Error<C::Error, ruma_client_api::Error>>
    where
        C: DefaultConstructibleHttpClient,
    {
        self.http_client(C::default()).await
    }

    /// Set the HTTP client to finish building the [`Client`].
    ///
    /// Unless the supported Matrix versions were manually set via
    /// [`supported_matrix_versions`][Self::supported_matrix_versions], this will do a
    /// [`get_supported_versions`] request to find out about the supported versions.
    pub async fn http_client<C>(
        self,
        http_client: C,
    ) -> Result<Client<C>, Error<C::Error, ruma_client_api::Error>>
    where
        C: HttpClient,
    {
        let homeserver_url = self
            .homeserver_url
            .expect("homeserver URL has to be set prior to calling .build() or .http_client()");

        let supported_matrix_versions = match self.supported_matrix_versions {
            Some(versions) => versions,
            None => http_client
                .send_matrix_request(
                    &homeserver_url,
                    SendAccessToken::None,
                    &[MatrixVersion::V1_0],
                    get_supported_versions::Request::new(),
                )
                .await?
                .known_versions()
                .into_iter()
                .collect(),
        };

        Ok(Client(Arc::new(ClientData {
            homeserver_url,
            http_client,
            access_token: Mutex::new(self.access_token),
            supported_matrix_versions,
        })))
    }
}
