//! `GET /_matrix/client/*/rooms/{roomId}/state/{eventType}/{stateKey}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3roomsroomidstateeventtypestatekey

    use ruma_common::{
        api::ruma_api,
        events::{AnyStateEventContent, StateEventType},
        serde::{Incoming, Raw},
        RoomId,
    };

    ruma_api! {
        metadata: {
            description: "Get state events associated with a given key.",
            method: GET,
            name: "get_state_events_for_key",
            r0_path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
            stable_path: "/_matrix/client/v3/rooms/:room_id/state/:event_type/:state_key",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        response: {
            /// The content of the state event.
            ///
            /// Since the inner type of the `Raw` does not implement `Deserialize`, you need to use
            /// [`Raw::deserialize_content`] to deserialize it.
            #[ruma_api(body)]
            pub content: Raw<AnyStateEventContent>,
        }

        error: crate::Error
    }

    /// Data for a request to the `get_state_events_for_key` API endpoint.
    ///
    /// Get state events associated with a given key.
    #[derive(Clone, Debug, Incoming)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[incoming_derive(!Deserialize)]
    pub struct Request<'a> {
        /// The room to look up the state for.
        pub room_id: &'a RoomId,

        /// The type of state to look up.
        pub event_type: StateEventType,

        /// The key of the state to look up.
        pub state_key: &'a str,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID, event type and state key.
        pub fn new(room_id: &'a RoomId, event_type: StateEventType, state_key: &'a str) -> Self {
            Self { room_id, event_type, state_key }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given content.
        pub fn new(content: Raw<AnyStateEventContent>) -> Self {
            Self { content }
        }
    }

    #[cfg(feature = "client")]
    impl<'a> ruma_common::api::OutgoingRequest for Request<'a> {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        const METADATA: ruma_common::api::Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            access_token: ruma_common::api::SendAccessToken<'_>,
            considering_versions: &'_ [ruma_common::api::MatrixVersion],
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use std::borrow::Cow;

            use http::header;
            use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

            let room_id_percent = utf8_percent_encode(self.room_id.as_str(), NON_ALPHANUMERIC);
            let event_type = self.event_type.to_string();
            let event_type_percent = utf8_percent_encode(&event_type, NON_ALPHANUMERIC);

            let mut url = format!(
                "{}{}",
                base_url.strip_suffix('/').unwrap_or(base_url),
                ruma_common::api::select_path(
                    considering_versions,
                    &METADATA,
                    None,
                    Some(format_args!(
                        "/_matrix/client/r0/rooms/{}/state/{}",
                        room_id_percent, event_type_percent
                    )),
                    Some(format_args!(
                        "/_matrix/client/v3/rooms/{}/state/{}",
                        room_id_percent, event_type_percent
                    )),
                )?
            );

            if !self.state_key.is_empty() {
                url.push('/');
                url.push_str(&Cow::from(utf8_percent_encode(self.state_key, NON_ALPHANUMERIC)));
            }

            http::Request::builder()
                .method(http::Method::GET)
                .uri(url)
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    format!(
                        "Bearer {}",
                        access_token
                            .get_required_for_endpoint()
                            .ok_or(ruma_common::api::error::IntoHttpError::NeedsAuthentication)?,
                    ),
                )
                .body(T::default())
                .map_err(Into::into)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for IncomingRequest {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        const METADATA: ruma_common::api::Metadata = METADATA;

        fn try_from_http_request<B, S>(
            _request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            use ruma_common::OwnedRoomId;

            // FIXME: find a way to make this if-else collapse with serde recognizing trailing
            // Option
            let (room_id, event_type, state_key): (OwnedRoomId, StateEventType, String) =
                if path_args.len() == 3 {
                    serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                        _,
                        serde::de::value::Error,
                    >::new(
                        path_args.iter().map(::std::convert::AsRef::as_ref),
                    ))?
                } else {
                    let (a, b) =
                        serde::Deserialize::deserialize(serde::de::value::SeqDeserializer::<
                            _,
                            serde::de::value::Error,
                        >::new(
                            path_args.iter().map(::std::convert::AsRef::as_ref),
                        ))?;

                    (a, b, "".into())
                };

            Ok(Self { room_id, event_type, state_key })
        }
    }
}
