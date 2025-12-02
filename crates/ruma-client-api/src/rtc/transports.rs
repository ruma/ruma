//! `GET /_matrix/client/*/rtc/transports`
//!
//! Discover the RTC transports advertised by the homeserver.

pub mod v1 {
    //! `/v1/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143

    use std::borrow::Cow;

    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::JsonObject,
    };
    use serde::{Deserialize, Serialize, de::DeserializeOwned};
    use serde_json::Value as JsonValue;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4143/rtc/transports",
        }
    }

    /// Request type for the `transports` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    impl Request {
        /// Creates a new empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    /// Response type for the `transports` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// The RTC transports advertised by the homeserver.
        pub rtc_transports: Vec<RtcTransport>,
    }

    impl Response {
        /// Creates a `Response` with the given RTC transports.
        pub fn new(rtc_transports: Vec<RtcTransport>) -> Self {
            Self { rtc_transports }
        }
    }

    /// A MatrixRTC transport.
    ///
    /// This type can hold arbitrary RTC transports. Their data can be accessed with
    /// [`transport_type()`](Self::transport_type) and [`data()`](Self::data()).
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[serde(tag = "type")]
    pub enum RtcTransport {
        /// A LiveKit multi-SFU transport.
        #[cfg(feature = "unstable-msc4195")]
        #[serde(rename = "livekit_multi_sfu")]
        LivekitMultiSfu(LivekitMultiSfuTransport),

        /// An unsupported transport.
        #[doc(hidden)]
        #[serde(untagged)]
        _Custom(CustomRtcTransport),
    }

    impl RtcTransport {
        /// A constructor to create a custom RTC transport.
        ///
        /// Prefer to use the public variants of `RtcTransport` where possible; this constructor is
        /// meant to be used for unsupported focus types only and does not allow setting arbitrary
        /// data for supported ones.
        ///
        /// # Errors
        ///
        /// Returns an error if the `transport_type` is known and deserialization of `data` to the
        /// corresponding `RtcTransport` variant fails.
        pub fn new(transport_type: String, data: JsonObject) -> serde_json::Result<Self> {
            fn deserialize_variant<T: DeserializeOwned>(obj: JsonObject) -> serde_json::Result<T> {
                serde_json::from_value(obj.into())
            }

            Ok(match transport_type.as_str() {
                #[cfg(feature = "unstable-msc4195")]
                "livekit_multi_sfu" => Self::LivekitMultiSfu(deserialize_variant(data)?),
                _ => Self::_Custom(CustomRtcTransport { transport_type, data }),
            })
        }

        /// Returns a reference to the type of this RTC transport.
        pub fn transport_type(&self) -> &str {
            match self {
                #[cfg(feature = "unstable-msc4195")]
                Self::LivekitMultiSfu(_) => "livekit_multi_sfu",
                Self::_Custom(custom) => &custom.transport_type,
            }
        }

        /// Returns the associated data.
        ///
        /// The returned JSON object won't contain the `type` field, please use
        /// [`transport_type()`][Self::transport_type] to access that.
        ///
        /// Prefer to use the public variants of `RtcTransport` where possible; this method is meant
        /// to be used for custom focus types only.
        pub fn data(&self) -> Cow<'_, JsonObject> {
            fn serialize<T: Serialize>(object: &T) -> JsonObject {
                match serde_json::to_value(object).expect("rtc focus type serialization to succeed")
                {
                    JsonValue::Object(object) => object,
                    _ => panic!("rtc transports must serialize to JSON objects"),
                }
            }

            match self {
                #[cfg(feature = "unstable-msc4195")]
                Self::LivekitMultiSfu(info) => Cow::Owned(serialize(info)),
                Self::_Custom(info) => Cow::Borrowed(&info.data),
            }
        }
    }

    /// A LiveKit multi-SFU transport.
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[cfg(feature = "unstable-msc4195")]
    pub struct LivekitMultiSfuTransport {
        /// The URL for the LiveKit service.
        pub livekit_service_url: String,
    }

    #[cfg(feature = "unstable-msc4195")]
    impl LivekitMultiSfuTransport {
        /// Construct a new `LivekitMultiSfuTransport` with the given LiveKit service URL.
        pub fn new(livekit_service_url: String) -> Self {
            Self { livekit_service_url }
        }
    }

    #[cfg(feature = "unstable-msc4195")]
    impl From<LivekitMultiSfuTransport> for RtcTransport {
        fn from(value: LivekitMultiSfuTransport) -> Self {
            Self::LivekitMultiSfu(value)
        }
    }

    #[cfg(feature = "unstable-msc4195")]
    impl From<LivekitMultiSfuTransport> for crate::discovery::discover_homeserver::LiveKitRtcFocusInfo {
        fn from(value: LivekitMultiSfuTransport) -> Self {
            let LivekitMultiSfuTransport { livekit_service_url } = value;
            Self { service_url: livekit_service_url }
        }
    }

    #[cfg(feature = "unstable-msc4195")]
    impl From<crate::discovery::discover_homeserver::LiveKitRtcFocusInfo> for LivekitMultiSfuTransport {
        fn from(value: crate::discovery::discover_homeserver::LiveKitRtcFocusInfo) -> Self {
            let crate::discovery::discover_homeserver::LiveKitRtcFocusInfo { service_url } = value;
            Self { livekit_service_url: service_url }
        }
    }

    /// Information about an unsupported RTC transport.
    #[doc(hidden)]
    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
    pub struct CustomRtcTransport {
        /// The type of RTC transport.
        #[serde(rename = "type")]
        transport_type: String,

        /// Remaining data.
        #[serde(flatten)]
        data: JsonObject,
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{
        Value as JsonValue, from_value as from_json_value, json, to_value as to_json_value,
    };

    use super::v1::{LivekitMultiSfuTransport, RtcTransport};

    #[test]
    fn serialize_roundtrip_custom_rtc_transport() {
        let transport_type = "local.custom.transport";
        assert_matches!(
            json!({
                "foo": "bar",
                "baz": true,
            }),
            JsonValue::Object(transport_data)
        );
        let transport =
            RtcTransport::new(transport_type.to_owned(), transport_data.clone()).unwrap();
        let json = json!({
            "type": transport_type,
            "foo": "bar",
            "baz": true,
        });

        assert_eq!(transport.transport_type(), transport_type);
        assert_eq!(*transport.data().as_ref(), transport_data);
        assert_eq!(to_json_value(&transport).unwrap(), json);
        assert_eq!(from_json_value::<RtcTransport>(json).unwrap(), transport);
    }

    #[cfg(feature = "unstable-msc4195")]
    #[test]
    fn serialize_roundtrip_livekit_multi_sfu_transport() {
        let transport_type = "livekit_multi_sfu";
        let livekit_service_url = "http://livekit.local/";
        let transport =
            RtcTransport::from(LivekitMultiSfuTransport::new(livekit_service_url.to_owned()));
        let json = json!({
            "type": transport_type,
            "livekit_service_url": livekit_service_url,
        });

        assert_eq!(transport.transport_type(), transport_type);
        assert_eq!(to_json_value(&transport).unwrap(), json);
        assert_eq!(from_json_value::<RtcTransport>(json).unwrap(), transport);
    }
}
