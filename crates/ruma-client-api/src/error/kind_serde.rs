use std::{
    borrow::Cow,
    collections::btree_map::{BTreeMap, Entry},
    fmt,
    time::Duration,
};

use js_int::UInt;
use ruma_common::serde::{DeserializeFromCowStr, FromString};
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, Visitor},
    ser::{self, Serialize, SerializeMap, Serializer},
};
use serde_json::from_value as from_json_value;

use super::{ErrorKind, Extra, RetryAfter};
use crate::PrivOwnedStr;

enum Field<'de> {
    ErrCode,
    SoftLogout,
    RetryAfterMs,
    RoomVersion,
    AdminContact,
    Status,
    Body,
    CurrentVersion,
    Other(Cow<'de, str>),
}

impl<'de> Field<'de> {
    fn new(s: Cow<'de, str>) -> Field<'de> {
        match s.as_ref() {
            "errcode" => Self::ErrCode,
            "soft_logout" => Self::SoftLogout,
            "retry_after_ms" => Self::RetryAfterMs,
            "room_version" => Self::RoomVersion,
            "admin_contact" => Self::AdminContact,
            "status" => Self::Status,
            "body" => Self::Body,
            "current_version" => Self::CurrentVersion,
            _ => Self::Other(s),
        }
    }
}

impl<'de> Deserialize<'de> for Field<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Field<'de>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("any struct field")
            }

            fn visit_str<E>(self, value: &str) -> Result<Field<'de>, E>
            where
                E: de::Error,
            {
                Ok(Field::new(Cow::Owned(value.to_owned())))
            }

            fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Field<'de>, E>
            where
                E: de::Error,
            {
                Ok(Field::new(Cow::Borrowed(value)))
            }

            fn visit_string<E>(self, value: String) -> Result<Field<'de>, E>
            where
                E: de::Error,
            {
                Ok(Field::new(Cow::Owned(value)))
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)
    }
}

struct ErrorKindVisitor;

impl<'de> Visitor<'de> for ErrorKindVisitor {
    type Value = ErrorKind;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("enum ErrorKind")
    }

    fn visit_map<V>(self, mut map: V) -> Result<ErrorKind, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut errcode = None;
        let mut soft_logout = None;
        let mut retry_after_ms = None;
        let mut room_version = None;
        let mut admin_contact = None;
        let mut status = None;
        let mut body = None;
        let mut current_version = None;
        let mut extra = BTreeMap::new();

        macro_rules! set_field {
            (errcode) => {
                set_field!(@inner errcode)
            };
            ($field:ident) => {
                match errcode {
                    Some(set_field!(@variant_containing $field)) | None => {
                        set_field!(@inner $field)
                    }
                    // if we already know we're deserializing a different variant to the one
                    // containing this field, ignore its value.
                    Some(_) => {
                        let _ = map.next_value::<de::IgnoredAny>()?;
                    },
                }
            };
            (@variant_containing soft_logout) => { ErrCode::UnknownToken };
            (@variant_containing retry_after_ms) => { ErrCode::LimitExceeded };
            (@variant_containing room_version) => { ErrCode::IncompatibleRoomVersion };
            (@variant_containing admin_contact) => { ErrCode::ResourceLimitExceeded };
            (@variant_containing status) => { ErrCode::BadStatus };
            (@variant_containing body) => { ErrCode::BadStatus };
            (@variant_containing current_version) => { ErrCode::WrongRoomKeysVersion };
            (@inner $field:ident) => {
                {
                    if $field.is_some() {
                        return Err(de::Error::duplicate_field(stringify!($field)));
                    }
                    $field = Some(map.next_value()?);
                }
            };
        }

        while let Some(key) = map.next_key()? {
            match key {
                Field::ErrCode => set_field!(errcode),
                Field::SoftLogout => set_field!(soft_logout),
                Field::RetryAfterMs => set_field!(retry_after_ms),
                Field::RoomVersion => set_field!(room_version),
                Field::AdminContact => set_field!(admin_contact),
                Field::Status => set_field!(status),
                Field::Body => set_field!(body),
                Field::CurrentVersion => set_field!(current_version),
                Field::Other(other) => match extra.entry(other.into_owned()) {
                    Entry::Vacant(v) => {
                        v.insert(map.next_value()?);
                    }
                    Entry::Occupied(o) => {
                        return Err(de::Error::custom(format!("duplicate field `{}`", o.key())));
                    }
                },
            }
        }

        let errcode = errcode.ok_or_else(|| de::Error::missing_field("errcode"))?;
        let extra = Extra(extra);

        Ok(match errcode {
            ErrCode::Forbidden => ErrorKind::forbidden(),
            ErrCode::UnknownToken => ErrorKind::UnknownToken {
                soft_logout: soft_logout
                    .map(from_json_value)
                    .transpose()
                    .map_err(de::Error::custom)?
                    .unwrap_or_default(),
            },
            ErrCode::MissingToken => ErrorKind::MissingToken,
            ErrCode::BadJson => ErrorKind::BadJson,
            ErrCode::NotJson => ErrorKind::NotJson,
            ErrCode::NotFound => ErrorKind::NotFound,
            ErrCode::LimitExceeded => ErrorKind::LimitExceeded {
                retry_after: retry_after_ms
                    .map(from_json_value::<UInt>)
                    .transpose()
                    .map_err(de::Error::custom)?
                    .map(Into::into)
                    .map(Duration::from_millis)
                    .map(RetryAfter::Delay),
            },
            ErrCode::Unknown => ErrorKind::Unknown,
            ErrCode::Unrecognized => ErrorKind::Unrecognized,
            ErrCode::Unauthorized => ErrorKind::Unauthorized,
            ErrCode::UserDeactivated => ErrorKind::UserDeactivated,
            ErrCode::UserInUse => ErrorKind::UserInUse,
            ErrCode::InvalidUsername => ErrorKind::InvalidUsername,
            ErrCode::RoomInUse => ErrorKind::RoomInUse,
            ErrCode::InvalidRoomState => ErrorKind::InvalidRoomState,
            ErrCode::ThreepidInUse => ErrorKind::ThreepidInUse,
            ErrCode::ThreepidNotFound => ErrorKind::ThreepidNotFound,
            ErrCode::ThreepidAuthFailed => ErrorKind::ThreepidAuthFailed,
            ErrCode::ThreepidDenied => ErrorKind::ThreepidDenied,
            ErrCode::ServerNotTrusted => ErrorKind::ServerNotTrusted,
            ErrCode::UnsupportedRoomVersion => ErrorKind::UnsupportedRoomVersion,
            ErrCode::IncompatibleRoomVersion => ErrorKind::IncompatibleRoomVersion {
                room_version: from_json_value(
                    room_version.ok_or_else(|| de::Error::missing_field("room_version"))?,
                )
                .map_err(de::Error::custom)?,
            },
            ErrCode::BadState => ErrorKind::BadState,
            ErrCode::GuestAccessForbidden => ErrorKind::GuestAccessForbidden,
            ErrCode::CaptchaNeeded => ErrorKind::CaptchaNeeded,
            ErrCode::CaptchaInvalid => ErrorKind::CaptchaInvalid,
            ErrCode::MissingParam => ErrorKind::MissingParam,
            ErrCode::InvalidParam => ErrorKind::InvalidParam,
            ErrCode::TooLarge => ErrorKind::TooLarge,
            ErrCode::Exclusive => ErrorKind::Exclusive,
            ErrCode::ResourceLimitExceeded => ErrorKind::ResourceLimitExceeded {
                admin_contact: from_json_value(
                    admin_contact.ok_or_else(|| de::Error::missing_field("admin_contact"))?,
                )
                .map_err(de::Error::custom)?,
            },
            ErrCode::CannotLeaveServerNoticeRoom => ErrorKind::CannotLeaveServerNoticeRoom,
            ErrCode::WeakPassword => ErrorKind::WeakPassword,
            ErrCode::UnableToAuthorizeJoin => ErrorKind::UnableToAuthorizeJoin,
            ErrCode::UnableToGrantJoin => ErrorKind::UnableToGrantJoin,
            ErrCode::BadAlias => ErrorKind::BadAlias,
            ErrCode::DuplicateAnnotation => ErrorKind::DuplicateAnnotation,
            ErrCode::NotYetUploaded => ErrorKind::NotYetUploaded,
            ErrCode::CannotOverwriteMedia => ErrorKind::CannotOverwriteMedia,
            #[cfg(feature = "unstable-msc3575")]
            ErrCode::UnknownPos => ErrorKind::UnknownPos,
            ErrCode::UrlNotSet => ErrorKind::UrlNotSet,
            ErrCode::BadStatus => ErrorKind::BadStatus {
                status: status
                    .map(|s| {
                        from_json_value::<u16>(s)
                            .map_err(de::Error::custom)?
                            .try_into()
                            .map_err(de::Error::custom)
                    })
                    .transpose()?,
                body: body.map(from_json_value).transpose().map_err(de::Error::custom)?,
            },
            ErrCode::ConnectionFailed => ErrorKind::ConnectionFailed,
            ErrCode::ConnectionTimeout => ErrorKind::ConnectionTimeout,
            ErrCode::WrongRoomKeysVersion => ErrorKind::WrongRoomKeysVersion {
                current_version: from_json_value(
                    current_version.ok_or_else(|| de::Error::missing_field("current_version"))?,
                )
                .map_err(de::Error::custom)?,
            },
            #[cfg(feature = "unstable-msc3843")]
            ErrCode::Unactionable => ErrorKind::Unactionable,
            ErrCode::_Custom(errcode) => ErrorKind::_Custom { errcode, extra },
        })
    }
}

#[derive(FromString, DeserializeFromCowStr)]
#[ruma_enum(rename_all = "M_MATRIX_ERROR_CASE")]
enum ErrCode {
    Forbidden,
    UnknownToken,
    MissingToken,
    BadJson,
    NotJson,
    NotFound,
    LimitExceeded,
    Unknown,
    Unrecognized,
    Unauthorized,
    UserDeactivated,
    UserInUse,
    InvalidUsername,
    RoomInUse,
    InvalidRoomState,
    ThreepidInUse,
    ThreepidNotFound,
    ThreepidAuthFailed,
    ThreepidDenied,
    ServerNotTrusted,
    UnsupportedRoomVersion,
    IncompatibleRoomVersion,
    BadState,
    GuestAccessForbidden,
    CaptchaNeeded,
    CaptchaInvalid,
    MissingParam,
    InvalidParam,
    TooLarge,
    Exclusive,
    ResourceLimitExceeded,
    CannotLeaveServerNoticeRoom,
    WeakPassword,
    UnableToAuthorizeJoin,
    UnableToGrantJoin,
    BadAlias,
    DuplicateAnnotation,
    #[ruma_enum(alias = "FI.MAU.MSC2246_NOT_YET_UPLOADED")]
    NotYetUploaded,
    #[ruma_enum(alias = "FI.MAU.MSC2246_CANNOT_OVERWRITE_MEDIA")]
    CannotOverwriteMedia,
    #[cfg(feature = "unstable-msc3575")]
    UnknownPos,
    UrlNotSet,
    BadStatus,
    ConnectionFailed,
    ConnectionTimeout,
    WrongRoomKeysVersion,
    #[cfg(feature = "unstable-msc3843")]
    Unactionable,
    _Custom(PrivOwnedStr),
}

impl<'de> Deserialize<'de> for ErrorKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ErrorKindVisitor)
    }
}

impl Serialize for ErrorKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_map(None)?;
        st.serialize_entry("errcode", self.as_ref())?;
        match self {
            Self::UnknownToken { soft_logout: true } => {
                st.serialize_entry("soft_logout", &true)?;
            }
            Self::LimitExceeded { retry_after: Some(RetryAfter::Delay(duration)) } => {
                st.serialize_entry(
                    "retry_after_ms",
                    &UInt::try_from(duration.as_millis()).map_err(ser::Error::custom)?,
                )?;
            }
            Self::IncompatibleRoomVersion { room_version } => {
                st.serialize_entry("room_version", room_version)?;
            }
            Self::ResourceLimitExceeded { admin_contact } => {
                st.serialize_entry("admin_contact", admin_contact)?;
            }
            Self::_Custom { extra, .. } => {
                for (k, v) in &extra.0 {
                    st.serialize_entry(k, v)?;
                }
            }
            _ => {}
        }
        st.end()
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::room_version_id;
    use serde_json::{from_value as from_json_value, json};

    use super::ErrorKind;

    #[test]
    fn deserialize_forbidden() {
        let deserialized: ErrorKind = from_json_value(json!({ "errcode": "M_FORBIDDEN" })).unwrap();
        assert_eq!(
            deserialized,
            ErrorKind::Forbidden {
                #[cfg(feature = "unstable-msc2967")]
                authenticate: None
            }
        );
    }

    #[test]
    fn deserialize_forbidden_with_extra_fields() {
        let deserialized: ErrorKind = from_json_value(json!({
            "errcode": "M_FORBIDDEN",
            "error": "…",
        }))
        .unwrap();

        assert_eq!(
            deserialized,
            ErrorKind::Forbidden {
                #[cfg(feature = "unstable-msc2967")]
                authenticate: None
            }
        );
    }

    #[test]
    fn deserialize_incompatible_room_version() {
        let deserialized: ErrorKind = from_json_value(json!({
            "errcode": "M_INCOMPATIBLE_ROOM_VERSION",
            "room_version": "7",
        }))
        .unwrap();

        assert_eq!(
            deserialized,
            ErrorKind::IncompatibleRoomVersion { room_version: room_version_id!("7") }
        );
    }
}
