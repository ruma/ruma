//! `POST /_matrix/client/*/safety/report/{transaction_id}`
//!
//! Report content.

pub mod unstable {
    //! `org.matrix.msc4457` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4457

    use ruma_common::{
        EventId, OwnedEventId, OwnedMxcUri, OwnedRoomOrAliasId, OwnedServerName,
        OwnedTransactionId, OwnedUserId, RoomAliasId, RoomId, ServerName, UserId,
        api::{auth_scheme::AccessToken, request, response},
        harm::Harm,
        metadata,
        serde::deserialize_cow_str,
    };
    use serde::{Deserialize, Serialize};

    use crate::PrivOwnedStr;

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4457/safety/report/{transaction_id}",
        }
    }

    /// Request type for the `report` endpoint.
    #[request]
    pub struct Request {
        /// The transaction ID supplied by the client.
        #[ruma_api(path)]
        pub transaction_id: OwnedTransactionId,

        /// The report data.
        #[ruma_api(body)]
        pub report: Report,
    }

    impl Request {
        /// Creates a new `Request` with the given transaction ID and report.
        pub fn new(transaction_id: OwnedTransactionId, report: Report) -> Self {
            Self { transaction_id, report }
        }
    }

    /// Response type for the `report` endpoint.
    #[response]
    pub struct Response {
        /// The opaque ID of the report in the homeserver's system.
        pub report_id: String,
    }

    impl Response {
        /// Creates a new `Response` with the given report ID.
        pub fn new(report_id: String) -> Self {
            Self { report_id }
        }
    }

    /// A report.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum Report {
        /// A complaint about inappropriate behavior.
        Complaint(ComplaintReport),
    }

    impl From<ComplaintReport> for Report {
        fn from(value: ComplaintReport) -> Self {
            Self::Complaint(value)
        }
    }

    /// A complaint about inappropriate behavior.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[non_exhaustive]
    pub struct ComplaintReport {
        /// The target of this report.
        pub regarding: ReportTarget,

        /// The harm which the reporter claims is being caused by the target.
        pub harm: Harm,

        /// The reporter's description of the report.
        ///
        /// Cannot exceed 1024 bytes.
        pub description: String,
    }

    /// The target of a report.
    #[derive(Clone, Debug, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum ReportTarget {
        /// A user.
        User(OwnedUserId),

        /// An event.
        Event(OwnedEventId),

        /// A room.
        Room(OwnedRoomOrAliasId),

        /// A homeserver.
        Server(OwnedServerName),

        /// A media file.
        Media(OwnedMxcUri),

        /// The reporting system itself.
        ReportSystem,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    const SERVER_MAGIC: &str = "m.system";

    impl<'de> Deserialize<'de> for ReportTarget {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::Error;

            let string = deserialize_cow_str(deserializer)?;

            if string.starts_with("@") {
                Ok(Self::User(
                    UserId::parse(string)
                        .map_err(|_| Error::custom("failed to deserialize user id"))?,
                ))
            } else if string.starts_with("$") {
                Ok(Self::Event(
                    EventId::parse(string)
                        .map_err(|_| Error::custom("failed to deserialize event id"))?,
                ))
            } else if string.starts_with("#") {
                Ok(Self::Room(
                    RoomAliasId::parse(string)
                        .map_err(|_| Error::custom("failed to parse room alias"))?
                        .into(),
                ))
            } else if string.starts_with("!") {
                Ok(Self::Room(
                    RoomId::parse(string)
                        .map_err(|_| Error::custom("failed to parse room id"))?
                        .into(),
                ))
            } else if let Some(server_name) = string.strip_prefix("server:") {
                Ok(Self::Server(
                    ServerName::parse(server_name)
                        .map_err(|_| Error::custom("failed to parse server name"))?,
                ))
            } else if string.starts_with("mxc://") {
                Ok(Self::Media(OwnedMxcUri::from(string)))
            } else if string == SERVER_MAGIC {
                Ok(Self::ReportSystem)
            } else {
                Ok(Self::_Custom(PrivOwnedStr(string.into_owned().into())))
            }
        }
    }

    impl Serialize for ReportTarget {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::User(user_id) => user_id.serialize(serializer),
                Self::Event(event_id) => event_id.serialize(serializer),
                Self::Room(room_id) => room_id.serialize(serializer),
                Self::Server(server_name) => {
                    serializer.collect_str(&format!("server:{server_name}"))
                }
                Self::Media(mxc) => mxc.serialize(serializer),
                Self::ReportSystem => serializer.collect_str(SERVER_MAGIC),
                Self::_Custom(custom) => serializer.collect_str(&custom.0),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use ruma_common::{
            owned_event_id, owned_mxc_uri, owned_room_alias_id, owned_room_id, owned_server_name,
            owned_user_id,
        };
        use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

        use super::ReportTarget;

        #[test]
        fn deserialize_report_target() {
            assert_eq!(
                from_json_value::<ReportTarget>(json!("@alice:example.org")).unwrap(),
                ReportTarget::User(owned_user_id!("@alice:example.org"))
            );
            assert_eq!(
                from_json_value::<ReportTarget>(json!("$foo")).unwrap(),
                ReportTarget::Event(owned_event_id!("$foo"))
            );
            assert_eq!(
                from_json_value::<ReportTarget>(json!("!foo")).unwrap(),
                ReportTarget::Room(owned_room_id!("!foo").into())
            );
            assert_eq!(
                from_json_value::<ReportTarget>(json!("#foo:example.org")).unwrap(),
                ReportTarget::Room(owned_room_alias_id!("#foo:example.org").into())
            );
            assert_eq!(
                from_json_value::<ReportTarget>(json!("mxc://example.org/foo")).unwrap(),
                ReportTarget::Media(owned_mxc_uri!("mxc://example.org/foo"))
            );
            assert_eq!(
                from_json_value::<ReportTarget>(json!("server:example.org")).unwrap(),
                ReportTarget::Server(owned_server_name!("example.org"))
            );
            assert_eq!(
                from_json_value::<ReportTarget>(json!("m.system")).unwrap(),
                ReportTarget::ReportSystem
            );
            assert_eq!(
                from_json_value::<ReportTarget>(json!("foo")).unwrap(),
                ReportTarget::_Custom(crate::PrivOwnedStr("foo".into()))
            );
        }

        #[test]
        fn serialize_report_target() {
            assert_eq!(
                to_json_value(ReportTarget::User(owned_user_id!("@alice:example.org"))).unwrap(),
                json!("@alice:example.org")
            );
            assert_eq!(
                to_json_value(ReportTarget::Event(owned_event_id!("$foo"))).unwrap(),
                json!("$foo")
            );
            assert_eq!(
                to_json_value(ReportTarget::Room(owned_room_id!("!foo").into())).unwrap(),
                json!("!foo")
            );
            assert_eq!(
                to_json_value(ReportTarget::Room(owned_room_alias_id!("#foo:example.org").into()))
                    .unwrap(),
                json!("#foo:example.org")
            );
            assert_eq!(
                to_json_value(ReportTarget::Media(owned_mxc_uri!("mxc://example.org/foo").into()))
                    .unwrap(),
                json!("mxc://example.org/foo")
            );
            assert_eq!(
                to_json_value(ReportTarget::Server(owned_server_name!("example.org").into()))
                    .unwrap(),
                json!("server:example.org")
            );
            assert_eq!(to_json_value(ReportTarget::ReportSystem).unwrap(), json!("m.system"));
            assert_eq!(
                to_json_value(ReportTarget::_Custom(crate::PrivOwnedStr("foo".into()))).unwrap(),
                json!("foo")
            );
        }

        #[test]
        fn fail_to_deserialize_user_id_report_target() {
            assert!(from_json_value::<ReportTarget>(json!("@aawagga")).is_err());
        }
    }
}
