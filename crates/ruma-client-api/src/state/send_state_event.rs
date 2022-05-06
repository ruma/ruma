//! `PUT /_matrix/client/*/rooms/{roomId}/state/{eventType}/{stateKey}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3roomsroomidstateeventtypestatekey

    use ruma_common::{
        api::ruma_api,
        events::{AnyStateEventContent, StateEventContent, StateEventType},
        serde::{Incoming, Raw},
        OwnedEventId, RoomId,
    };
    use serde_json::value::to_raw_value as to_raw_json_value;

    ruma_api! {
        metadata: {
            description: "Send a state event to a room associated with a given state key.",
            method: PUT,
            name: "send_state_event",
            r0_path: "/_matrix/client/r0/rooms/:room_id/state/:event_type/:state_key",
            stable_path: "/_matrix/client/v3/rooms/:room_id/state/:event_type/:state_key",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        response: {
            /// A unique identifier for the event.
            pub event_id: OwnedEventId,
        }

        error: crate::Error
    }

    /// Data for a request to the `send_state_event` API endpoint.
    ///
    /// Send a state event to a room associated with a given state key.
    #[derive(Clone, Debug, Incoming)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[incoming_derive(!Deserialize)]
    pub struct Request<'a> {
        /// The room to set the state in.
        pub room_id: &'a RoomId,

        /// The type of event to send.
        pub event_type: StateEventType,

        /// The state_key for the state to send.
        pub state_key: &'a str,

        /// The event content to send.
        pub body: Raw<AnyStateEventContent>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room id, state key and event content.
        ///
        /// # Errors
        ///
        /// Since `Request` stores the request body in serialized form, this function can fail if
        /// `T`s [`Serialize`][serde::Serialize] implementation can fail.
        pub fn new<T>(
            room_id: &'a RoomId,
            state_key: &'a str,
            content: &'a T,
        ) -> serde_json::Result<Self>
        where
            T: StateEventContent,
        {
            Ok(Self {
                room_id,
                state_key,
                event_type: content.event_type(),
                body: Raw::from_json(to_raw_json_value(content)?),
            })
        }

        /// Creates a new `Request` with the given room id, event type, state key and raw event
        /// content.
        pub fn new_raw(
            room_id: &'a RoomId,
            event_type: StateEventType,
            state_key: &'a str,
            body: Raw<AnyStateEventContent>,
        ) -> Self {
            Self { room_id, event_type, state_key, body }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given event id.
        pub fn new(event_id: OwnedEventId) -> Self {
            Self { event_id }
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

            use http::header::{self, HeaderValue};
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

            // Last URL segment is optional, that is why this trait impl is not generated.
            if !self.state_key.is_empty() {
                url.push('/');
                url.push_str(&Cow::from(utf8_percent_encode(self.state_key, NON_ALPHANUMERIC)));
            }

            let http_request = http::Request::builder()
                .method(http::Method::PUT)
                .uri(url)
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::AUTHORIZATION,
                    HeaderValue::from_str(&format!(
                        "Bearer {}",
                        access_token
                            .get_required_for_endpoint()
                            .ok_or(ruma_common::api::error::IntoHttpError::NeedsAuthentication)?
                    ))?,
                )
                .body(ruma_common::serde::json_to_buf(&self.body)?)?;

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for IncomingRequest {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        const METADATA: ruma_common::api::Metadata = METADATA;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
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

            let body = serde_json::from_slice(request.body().as_ref())?;

            Ok(Self { room_id, event_type, state_key, body })
        }
    }
}
