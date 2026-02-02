//! `GET /_matrix/client/*/capabilities`
//!
//! Get information about the server's supported feature set and other relevant capabilities
//! ([spec]).
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#capabilities-negotiation

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3capabilities

    use std::{borrow::Cow, collections::BTreeMap};

    use maplit::btreemap;
    use ruma_common::{
        RoomVersionId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::StringEnum,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::{
        Value as JsonValue, from_value as from_json_value, to_value as to_json_value,
    };

    use crate::{PrivOwnedStr, profile::ProfileFieldName};

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/capabilities",
            1.1 => "/_matrix/client/v3/capabilities",
        }
    }

    /// Request type for the `get_capabilities` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `get_capabilities` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The capabilities the server supports
        pub capabilities: Capabilities,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given capabilities.
        pub fn new(capabilities: Capabilities) -> Self {
            Self { capabilities }
        }
    }

    impl From<Capabilities> for Response {
        fn from(capabilities: Capabilities) -> Self {
            Self::new(capabilities)
        }
    }

    /// Contains information about all the capabilities that the server supports.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[allow(deprecated)]
    pub struct Capabilities {
        /// Capability to indicate if the user can change their password.
        #[serde(
            rename = "m.change_password",
            default,
            skip_serializing_if = "ChangePasswordCapability::is_default"
        )]
        pub change_password: ChangePasswordCapability,

        /// The room versions the server supports.
        #[serde(
            rename = "m.room_versions",
            default,
            skip_serializing_if = "RoomVersionsCapability::is_default"
        )]
        pub room_versions: RoomVersionsCapability,

        /// Capability to indicate if the user can change their display name.
        #[serde(
            rename = "m.set_displayname",
            default,
            skip_serializing_if = "SetDisplayNameCapability::is_default"
        )]
        #[deprecated = "Since Matrix 1.16, prefer profile_fields if it is set."]
        pub set_displayname: SetDisplayNameCapability,

        /// Capability to indicate if the user can change their avatar.
        #[serde(
            rename = "m.set_avatar_url",
            default,
            skip_serializing_if = "SetAvatarUrlCapability::is_default"
        )]
        #[deprecated = "Since Matrix 1.16, prefer profile_fields if it is set."]
        pub set_avatar_url: SetAvatarUrlCapability,

        /// Capability to indicate if the user can change the third-party identifiers associated
        /// with their account.
        #[serde(
            rename = "m.3pid_changes",
            default,
            skip_serializing_if = "ThirdPartyIdChangesCapability::is_default"
        )]
        pub thirdparty_id_changes: ThirdPartyIdChangesCapability,

        /// Capability to indicate if the user can generate tokens to log further clients into
        /// their account.
        #[serde(
            rename = "m.get_login_token",
            default,
            skip_serializing_if = "GetLoginTokenCapability::is_default"
        )]
        pub get_login_token: GetLoginTokenCapability,

        /// Capability to indicate if the user can set extended profile fields.
        #[serde(
            rename = "m.profile_fields",
            alias = "uk.tcpip.msc4133.profile_fields",
            skip_serializing_if = "Option::is_none"
        )]
        pub profile_fields: Option<ProfileFieldsCapability>,

        /// Capability to indicate if the server automatically forgets rooms that the user leaves.
        #[serde(
            rename = "m.forget_forced_upon_leave",
            default,
            skip_serializing_if = "ForgetForcedUponLeaveCapability::is_default"
        )]
        pub forget_forced_upon_leave: ForgetForcedUponLeaveCapability,

        /// Any other custom capabilities that the server supports outside of the specification,
        /// labeled using the Java package naming convention and stored as arbitrary JSON values.
        #[serde(flatten)]
        custom_capabilities: BTreeMap<String, JsonValue>,
    }

    impl Capabilities {
        /// Creates empty `Capabilities`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns the value of the given capability.
        ///
        /// Prefer to use the public fields of `Capabilities` where possible; this method is meant
        /// to be used for unsupported capabilities only.
        pub fn get(&self, capability: &str) -> Option<Cow<'_, JsonValue>> {
            fn serialize<T: Serialize>(cap: &T) -> JsonValue {
                to_json_value(cap).expect("capability serialization to succeed")
            }

            match capability {
                "m.change_password" => Some(Cow::Owned(serialize(&self.change_password))),
                "m.room_versions" => Some(Cow::Owned(serialize(&self.room_versions))),
                #[allow(deprecated)]
                "m.set_displayname" => Some(Cow::Owned(serialize(&self.set_displayname))),
                #[allow(deprecated)]
                "m.set_avatar_url" => Some(Cow::Owned(serialize(&self.set_avatar_url))),
                "m.3pid_changes" => Some(Cow::Owned(serialize(&self.thirdparty_id_changes))),
                "m.get_login_token" => Some(Cow::Owned(serialize(&self.get_login_token))),
                "m.forget_forced_upon_leave" => {
                    Some(Cow::Owned(serialize(&self.forget_forced_upon_leave)))
                }
                _ => self.custom_capabilities.get(capability).map(Cow::Borrowed),
            }
        }

        /// Sets a capability to the given value.
        ///
        /// Prefer to use the public fields of `Capabilities` where possible; this method is meant
        /// to be used for unsupported capabilities only and does not allow setting
        /// arbitrary data for supported ones.
        pub fn set(&mut self, capability: &str, value: JsonValue) -> serde_json::Result<()> {
            match capability {
                "m.change_password" => self.change_password = from_json_value(value)?,
                "m.room_versions" => self.room_versions = from_json_value(value)?,
                #[allow(deprecated)]
                "m.set_displayname" => self.set_displayname = from_json_value(value)?,
                #[allow(deprecated)]
                "m.set_avatar_url" => self.set_avatar_url = from_json_value(value)?,
                "m.3pid_changes" => self.thirdparty_id_changes = from_json_value(value)?,
                "m.get_login_token" => self.get_login_token = from_json_value(value)?,
                "m.forget_forced_upon_leave" => {
                    self.forget_forced_upon_leave = from_json_value(value)?;
                }
                _ => {
                    self.custom_capabilities.insert(capability.to_owned(), value);
                }
            }

            Ok(())
        }
    }

    /// Information about the m.change_password capability
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ChangePasswordCapability {
        /// `true` if the user can change their password, `false` otherwise.
        pub enabled: bool,
    }

    impl ChangePasswordCapability {
        /// Creates a new `ChangePasswordCapability` with the given enabled flag.
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }

        /// Returns whether all fields have their default value.
        pub fn is_default(&self) -> bool {
            self.enabled
        }
    }

    impl Default for ChangePasswordCapability {
        fn default() -> Self {
            Self { enabled: true }
        }
    }

    /// Information about the m.room_versions capability
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RoomVersionsCapability {
        /// The default room version the server is using for new rooms.
        pub default: RoomVersionId,

        /// A detailed description of the room versions the server supports.
        pub available: BTreeMap<RoomVersionId, RoomVersionStability>,
    }

    impl RoomVersionsCapability {
        /// Creates a new `RoomVersionsCapability` with the given default room version ID and room
        /// version descriptions.
        pub fn new(
            default: RoomVersionId,
            available: BTreeMap<RoomVersionId, RoomVersionStability>,
        ) -> Self {
            Self { default, available }
        }

        /// Returns whether all fields have their default value.
        pub fn is_default(&self) -> bool {
            self.default == RoomVersionId::V1
                && self.available.len() == 1
                && self
                    .available
                    .get(&RoomVersionId::V1)
                    .map(|stability| *stability == RoomVersionStability::Stable)
                    .unwrap_or(false)
        }
    }

    impl Default for RoomVersionsCapability {
        fn default() -> Self {
            Self {
                default: RoomVersionId::V1,
                available: btreemap! { RoomVersionId::V1 => RoomVersionStability::Stable },
            }
        }
    }

    /// The stability of a room version.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum)]
    #[ruma_enum(rename_all = "lowercase")]
    #[non_exhaustive]
    pub enum RoomVersionStability {
        /// Support for the given version is stable.
        Stable,

        /// Support for the given version is unstable.
        Unstable,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// Information about the `m.set_displayname` capability
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[deprecated = "Since Matrix 1.16, prefer ProfileFieldsCapability instead."]
    pub struct SetDisplayNameCapability {
        /// `true` if the user can change their display name, `false` otherwise.
        pub enabled: bool,
    }

    #[allow(deprecated)]
    impl SetDisplayNameCapability {
        /// Creates a new `SetDisplayNameCapability` with the given enabled flag.
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }

        /// Returns whether all fields have their default value.
        pub fn is_default(&self) -> bool {
            self.enabled
        }
    }

    #[allow(deprecated)]
    impl Default for SetDisplayNameCapability {
        fn default() -> Self {
            Self { enabled: true }
        }
    }

    /// Information about the `m.set_avatar_url` capability
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[deprecated = "Since Matrix 1.16, prefer ProfileFieldsCapability instead."]
    pub struct SetAvatarUrlCapability {
        /// `true` if the user can change their avatar, `false` otherwise.
        pub enabled: bool,
    }

    #[allow(deprecated)]
    impl SetAvatarUrlCapability {
        /// Creates a new `SetAvatarUrlCapability` with the given enabled flag.
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }

        /// Returns whether all fields have their default value.
        pub fn is_default(&self) -> bool {
            self.enabled
        }
    }

    #[allow(deprecated)]
    impl Default for SetAvatarUrlCapability {
        fn default() -> Self {
            Self { enabled: true }
        }
    }

    /// Information about the `m.3pid_changes` capability
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThirdPartyIdChangesCapability {
        /// `true` if the user can change the third-party identifiers associated with their
        /// account, `false` otherwise.
        pub enabled: bool,
    }

    impl ThirdPartyIdChangesCapability {
        /// Creates a new `ThirdPartyIdChangesCapability` with the given enabled flag.
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }

        /// Returns whether all fields have their default value.
        pub fn is_default(&self) -> bool {
            self.enabled
        }
    }

    impl Default for ThirdPartyIdChangesCapability {
        fn default() -> Self {
            Self { enabled: true }
        }
    }

    /// Information about the `m.get_login_token` capability.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct GetLoginTokenCapability {
        /// Whether the user can request a login token.
        pub enabled: bool,
    }

    impl GetLoginTokenCapability {
        /// Creates a new `GetLoginTokenCapability` with the given enabled flag.
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }

        /// Returns whether all fields have their default value.
        pub fn is_default(&self) -> bool {
            !self.enabled
        }
    }

    /// Information about the `m.profile_fields` capability.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ProfileFieldsCapability {
        /// Whether the user can set extended profile fields.
        pub enabled: bool,

        /// The fields that can be set by the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub allowed: Option<Vec<ProfileFieldName>>,

        /// The fields that cannot be set by the user.
        ///
        /// This list is ignored if `allowed` is provided.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub disallowed: Option<Vec<ProfileFieldName>>,
    }

    impl ProfileFieldsCapability {
        /// Creates a new `ProfileFieldsCapability` with the given enabled flag.
        pub fn new(enabled: bool) -> Self {
            Self { enabled, allowed: None, disallowed: None }
        }

        /// Whether the server advertises that the field with the given name can be set.
        pub fn can_set_field(&self, field: &ProfileFieldName) -> bool {
            if !self.enabled {
                return false;
            }

            if let Some(allowed) = &self.allowed {
                allowed.contains(field)
            } else if let Some(disallowed) = &self.disallowed {
                !disallowed.contains(field)
            } else {
                // The default is that any field is allowed.
                true
            }
        }
    }

    /// Information about the `m.forget_forced_upon_leave` capability.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ForgetForcedUponLeaveCapability {
        /// Whether the server will automatically forget any room that the user leaves.
        ///
        /// This behavior applies irrespective of whether the user has left the room on their own
        /// or has been kicked or banned from the room by another user.
        pub enabled: bool,
    }

    impl ForgetForcedUponLeaveCapability {
        /// Creates a new `ForgetForcedUponLeaveCapability` with the given enabled flag.
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }

        /// Returns whether all fields have their default value.
        pub fn is_default(&self) -> bool {
            !self.enabled
        }
    }
}
