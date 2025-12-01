//! `GET /_matrix/client/*/rooms/{roomId}/state/{eventType}/{stateKey}`
//!
//! Get state events associated with a given key.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3roomsroomidstateeventtypestatekey

    use ruma_common::{
        OwnedRoomId,
        api::{auth_scheme::AccessToken, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::{AnyStateEvent, AnyStateEventContent, StateEventType};
    use serde_json::value::RawValue as RawJsonValue;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/state/{event_type}/{state_key}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/state/{event_type}/{state_key}",
        }
    }

    /// Request type for the `get_state_events_for_key` endpoint.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Request {
        /// The room to look up the state for.
        pub room_id: OwnedRoomId,

        /// The type of state to look up.
        pub event_type: StateEventType,

        /// The key of the state to look up.
        pub state_key: String,

        /// The format to use for the returned data.
        pub format: StateEventFormat,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID, event type and state key.
        pub fn new(room_id: OwnedRoomId, event_type: StateEventType, state_key: String) -> Self {
            Self { room_id, event_type, state_key, format: StateEventFormat::default() }
        }
    }

    /// The format to use for the returned data.
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[derive(Default, Debug, PartialEq, Clone, Copy)]
    #[serde(rename_all = "lowercase")]
    pub enum StateEventFormat {
        /// Will return only the content of the state event.
        ///
        /// This is the default value if the format is unspecified in the request.
        #[default]
        Content,

        /// Will return the entire event in the usual format suitable for clients, including fields
        /// like event ID, sender and timestamp.
        Event,
    }

    /// Response type for the `get_state_events_for_key` endpoint, either the `Raw` `AnyStateEvent`
    /// or `AnyStateEventContent`.
    ///
    /// While it's possible to access the raw value directly, it's recommended you use the
    /// provided helper methods to access it, and `From` to create it.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The full event (content) of the state event.
        #[ruma_api(body)]
        pub event_or_content: Box<RawJsonValue>,
    }

    impl From<Raw<AnyStateEvent>> for Response {
        fn from(value: Raw<AnyStateEvent>) -> Self {
            Self { event_or_content: value.into_json() }
        }
    }

    impl From<Raw<AnyStateEventContent>> for Response {
        fn from(value: Raw<AnyStateEventContent>) -> Self {
            Self { event_or_content: value.into_json() }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given event (content).
        pub fn new(event_or_content: Box<RawJsonValue>) -> Self {
            Self { event_or_content }
        }

        /// Returns an unchecked `Raw<AnyStateEvent>`.
        ///
        /// This method should only be used if you specified the `format` in the request to be
        /// `StateEventFormat::Event`
        pub fn into_event(self) -> Raw<AnyStateEvent> {
            Raw::from_json(self.event_or_content)
        }

        /// Returns an unchecked `Raw<AnyStateEventContent>`.
        ///
        /// This method should only be used if you did not specify the `format` in the request, or
        /// set it to be `StateEventFormat::Content`
        ///
        /// Since the inner type of the `Raw` does not implement `Deserialize`, you need to use
        /// `.deserialize_as_unchecked::<T>()` or
        /// `.cast_ref_unchecked::<T>().deserialize_with_type()` to deserialize it.
        pub fn into_content(self) -> Raw<AnyStateEventContent> {
            Raw::from_json(self.event_or_content)
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        fn try_into_http_request<T: Default + bytes::BufMut + AsRef<[u8]>>(
            self,
            base_url: &str,
            access_token: ruma_common::api::auth_scheme::SendAccessToken<'_>,
            considering: std::borrow::Cow<'_, ruma_common::api::SupportedVersions>,
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            use ruma_common::api::{Metadata, auth_scheme::AuthScheme};

            let query_string = serde_html_form::to_string(RequestQuery { format: self.format })?;

            let mut http_request = http::Request::builder()
                .method(Self::METHOD)
                .uri(Self::make_endpoint_url(
                    considering,
                    base_url,
                    &[&self.room_id, &self.event_type, &self.state_key],
                    &query_string,
                )?)
                .body(T::default())?;

            Self::Authentication::add_authentication(&mut http_request, access_token).map_err(
                |error| ruma_common::api::error::IntoHttpError::Authentication(error.into()),
            )?;

            Ok(http_request)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            Self::check_request_method(request.method())?;

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

            let RequestQuery { format } =
                serde_html_form::from_str(request.uri().query().unwrap_or(""))?;

            Ok(Self { room_id, event_type, state_key, format })
        }
    }

    /// Data in the request's query string.
    #[derive(Debug)]
    #[cfg_attr(feature = "client", derive(serde::Serialize))]
    #[cfg_attr(feature = "server", derive(serde::Deserialize))]
    struct RequestQuery {
        /// Timestamp to use for the `origin_server_ts` of the event.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        format: StateEventFormat,
    }
}

#[cfg(all(test, feature = "client"))]
mod tests {
    use ruma_common::api::IncomingResponse;
    use ruma_events::room::name::RoomNameEventContent;
    use serde_json::{json, to_vec as to_json_vec};

    use super::v3::Response;

    #[test]
    fn deserialize_response() {
        let body = json!({
            "name": "Nice room 🙂"
        });
        let response = http::Response::new(to_json_vec(&body).unwrap());

        let response = Response::try_from_http_response(response).unwrap();
        let content =
            response.into_content().deserialize_as_unchecked::<RoomNameEventContent>().unwrap();

        assert_eq!(&content.name, "Nice room 🙂");
    }
}
