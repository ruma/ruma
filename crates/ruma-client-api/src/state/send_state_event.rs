//! `PUT /_matrix/client/*/rooms/{roomId}/state/{eventType}/{stateKey}`
//!
//! Send a state event to a room associated with a given state key.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3roomsroomidstateeventtypestatekey

    use std::borrow::Borrow;

    use ruma_common::{
        api::{response, Metadata},
        metadata,
        serde::Raw,
        MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId,
    };
    use ruma_events::{AnyStateEventContent, StateEventContent, StateEventType};
    use serde_json::value::to_raw_value as to_raw_json_value;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/state/{event_type}/{state_key}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/state/{event_type}/{state_key}",
        }
    };

    /// Request type for the `send_state_event` endpoint.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Request {
        /// The room to set the state in.
        pub room_id: OwnedRoomId,

        /// The type of event to send.
        pub event_type: StateEventType,

        /// The state_key for the state to send.
        pub state_key: String,

        /// The event content to send.
        pub body: Raw<AnyStateEventContent>,

        /// Timestamp to use for the `origin_server_ts` of the event.
        ///
        /// This is called [timestamp massaging] and can only be used by Appservices.
        ///
        /// Note that this does not change the position of the event in the timeline.
        ///
        /// [timestamp massaging]: https://spec.matrix.org/latest/application-service-api/#timestamp-massaging
        pub timestamp: Option<MilliSecondsSinceUnixEpoch>,
    }

    impl Request {
        /// Creates a new `Request` with the given room id, state key and event content.
        ///
        /// # Errors
        ///
        /// Since `Request` stores the request body in serialized form, this function can fail if
        /// `T`s [`Serialize`][serde::Serialize] implementation can fail.
        pub fn new<T, K>(
            room_id: OwnedRoomId,
            state_key: &K,
            content: &T,
        ) -> serde_json::Result<Self>
        where
            T: StateEventContent,
            T::StateKey: Borrow<K>,
            K: AsRef<str> + ?Sized,
        {
            Ok(Self {
                room_id,
                state_key: state_key.as_ref().to_owned(),
                event_type: content.event_type(),
                body: Raw::from_json(to_raw_json_value(content)?),
                timestamp: None,
            })
        }

        /// Creates a new `Request` with the given room id, event type, state key and raw event
        /// content.
        pub fn new_raw(
            room_id: OwnedRoomId,
            event_type: StateEventType,
            state_key: String,
            body: Raw<AnyStateEventContent>,
        ) -> Self {
            Self { room_id, event_type, state_key, body, timestamp: None }
        }
    }

    /// Response type for the `send_state_event` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A unique identifier for the event.
        pub event_id: OwnedEventId,
    }

    impl Response {
        /// Creates a new `Response` with the given event id.
        pub fn new(event_id: OwnedEventId) -> Self {
            Self { event_id }
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        const METADATA: Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            access_token: ruma_common::api::SendAccessToken<'_>,
            considering_versions: &'_ [ruma_common::api::MatrixVersion],
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use http::header::{self, HeaderValue};

            let query_string =
                serde_html_form::to_string(RequestQuery { timestamp: self.timestamp })?;

            let http_request = http::Request::builder()
                .method(http::Method::PUT)
                .uri(METADATA.make_endpoint_url(
                    considering_versions,
                    base_url,
                    &[&self.room_id, &self.event_type, &self.state_key],
                    &query_string,
                )?)
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
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        const METADATA: Metadata = METADATA;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
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

            let request_query: RequestQuery =
                serde_html_form::from_str(request.uri().query().unwrap_or(""))?;

            let body = serde_json::from_slice(request.body().as_ref())?;

            Ok(Self { room_id, event_type, state_key, body, timestamp: request_query.timestamp })
        }
    }

    /// Data in the request's query string.
    #[derive(Debug)]
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct RequestQuery {
        /// Timestamp to use for the `origin_server_ts` of the event.
        #[serde(rename = "ts", skip_serializing_if = "Option::is_none")]
        timestamp: Option<MilliSecondsSinceUnixEpoch>,
    }

    #[cfg(feature = "client")]
    #[test]
    fn serialize() {
        use ruma_common::{
            api::{MatrixVersion, OutgoingRequest as _, SendAccessToken},
            owned_room_id,
        };
        use ruma_events::{room::name::RoomNameEventContent, EmptyStateKey};

        // This used to panic in make_endpoint_url because of a mismatch in the path parameter count
        let req = Request::new(
            owned_room_id!("!room:server.tld"),
            &EmptyStateKey,
            &RoomNameEventContent::new("Test room".to_owned()),
        )
        .unwrap()
        .try_into_http_request::<Vec<u8>>(
            "https://server.tld",
            SendAccessToken::IfRequired("access_token"),
            &[MatrixVersion::V1_1],
        )
        .unwrap();

        assert_eq!(
            req.uri(),
            "https://server.tld/_matrix/client/v3/rooms/!room:server.tld/state/m.room.name/"
        );
    }
}
