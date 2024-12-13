use std::{
    borrow::Cow,
    collections::btree_map::{BTreeMap, Entry},
    fmt,
    time::Duration,
};

use js_int::UInt;
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, Visitor},
    ser::{self, Serialize, SerializeMap, Serializer},
};
use serde_json::from_value as from_json_value;

use super::{ErrorCode, ErrorKind, Extra, RetryAfter};

enum Field<'de> {
    ErrorCode,
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
            "errcode" => Self::ErrorCode,
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
            (@variant_containing soft_logout) => { ErrorCode::UnknownToken };
            (@variant_containing retry_after_ms) => { ErrorCode::LimitExceeded };
            (@variant_containing room_version) => { ErrorCode::IncompatibleRoomVersion };
            (@variant_containing admin_contact) => { ErrorCode::ResourceLimitExceeded };
            (@variant_containing status) => { ErrorCode::BadStatus };
            (@variant_containing body) => { ErrorCode::BadStatus };
            (@variant_containing current_version) => { ErrorCode::WrongRoomKeysVersion };
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
                Field::ErrorCode => set_field!(errcode),
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
            ErrorCode::BadAlias => ErrorKind::BadAlias,
            ErrorCode::BadJson => ErrorKind::BadJson,
            ErrorCode::BadState => ErrorKind::BadState,
            ErrorCode::BadStatus => ErrorKind::BadStatus {
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
            ErrorCode::CannotLeaveServerNoticeRoom => ErrorKind::CannotLeaveServerNoticeRoom,
            ErrorCode::CannotOverwriteMedia => ErrorKind::CannotOverwriteMedia,
            ErrorCode::CaptchaInvalid => ErrorKind::CaptchaInvalid,
            ErrorCode::CaptchaNeeded => ErrorKind::CaptchaNeeded,
            ErrorCode::ConnectionFailed => ErrorKind::ConnectionFailed,
            ErrorCode::ConnectionTimeout => ErrorKind::ConnectionTimeout,
            ErrorCode::DuplicateAnnotation => ErrorKind::DuplicateAnnotation,
            ErrorCode::Exclusive => ErrorKind::Exclusive,
            ErrorCode::Forbidden => ErrorKind::forbidden(),
            ErrorCode::GuestAccessForbidden => ErrorKind::GuestAccessForbidden,
            ErrorCode::IncompatibleRoomVersion => ErrorKind::IncompatibleRoomVersion {
                room_version: from_json_value(
                    room_version.ok_or_else(|| de::Error::missing_field("room_version"))?,
                )
                .map_err(de::Error::custom)?,
            },
            ErrorCode::InvalidParam => ErrorKind::InvalidParam,
            ErrorCode::InvalidRoomState => ErrorKind::InvalidRoomState,
            ErrorCode::InvalidUsername => ErrorKind::InvalidUsername,
            ErrorCode::LimitExceeded => ErrorKind::LimitExceeded {
                retry_after: retry_after_ms
                    .map(from_json_value::<UInt>)
                    .transpose()
                    .map_err(de::Error::custom)?
                    .map(Into::into)
                    .map(Duration::from_millis)
                    .map(RetryAfter::Delay),
            },
            ErrorCode::MissingParam => ErrorKind::MissingParam,
            ErrorCode::MissingToken => ErrorKind::MissingToken,
            ErrorCode::NotFound => ErrorKind::NotFound,
            ErrorCode::NotJson => ErrorKind::NotJson,
            ErrorCode::NotYetUploaded => ErrorKind::NotYetUploaded,
            ErrorCode::ResourceLimitExceeded => ErrorKind::ResourceLimitExceeded {
                admin_contact: from_json_value(
                    admin_contact.ok_or_else(|| de::Error::missing_field("admin_contact"))?,
                )
                .map_err(de::Error::custom)?,
            },
            ErrorCode::RoomInUse => ErrorKind::RoomInUse,
            ErrorCode::ServerNotTrusted => ErrorKind::ServerNotTrusted,
            ErrorCode::ThreepidAuthFailed => ErrorKind::ThreepidAuthFailed,
            ErrorCode::ThreepidDenied => ErrorKind::ThreepidDenied,
            ErrorCode::ThreepidInUse => ErrorKind::ThreepidInUse,
            ErrorCode::ThreepidMediumNotSupported => ErrorKind::ThreepidMediumNotSupported,
            ErrorCode::ThreepidNotFound => ErrorKind::ThreepidNotFound,
            ErrorCode::TooLarge => ErrorKind::TooLarge,
            ErrorCode::UnableToAuthorizeJoin => ErrorKind::UnableToAuthorizeJoin,
            ErrorCode::UnableToGrantJoin => ErrorKind::UnableToGrantJoin,
            #[cfg(feature = "unstable-msc3843")]
            ErrorCode::Unactionable => ErrorKind::Unactionable,
            ErrorCode::Unauthorized => ErrorKind::Unauthorized,
            ErrorCode::Unknown => ErrorKind::Unknown,
            #[cfg(any(feature = "unstable-msc3575", feature = "unstable-msc4186"))]
            ErrorCode::UnknownPos => ErrorKind::UnknownPos,
            ErrorCode::UnknownToken => ErrorKind::UnknownToken {
                soft_logout: soft_logout
                    .map(from_json_value)
                    .transpose()
                    .map_err(de::Error::custom)?
                    .unwrap_or_default(),
            },
            ErrorCode::Unrecognized => ErrorKind::Unrecognized,
            ErrorCode::UnsupportedRoomVersion => ErrorKind::UnsupportedRoomVersion,
            ErrorCode::UrlNotSet => ErrorKind::UrlNotSet,
            ErrorCode::UserDeactivated => ErrorKind::UserDeactivated,
            ErrorCode::UserInUse => ErrorKind::UserInUse,
            ErrorCode::UserLocked => ErrorKind::UserLocked,
            ErrorCode::UserSuspended => ErrorKind::UserSuspended,
            ErrorCode::WeakPassword => ErrorKind::WeakPassword,
            ErrorCode::WrongRoomKeysVersion => ErrorKind::WrongRoomKeysVersion {
                current_version: from_json_value(
                    current_version.ok_or_else(|| de::Error::missing_field("current_version"))?,
                )
                .map_err(de::Error::custom)?,
            },
            ErrorCode::_Custom(errcode) => ErrorKind::_Custom { errcode, extra },
        })
    }
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
        st.serialize_entry("errcode", &self.errcode())?;
        match self {
            Self::UnknownToken { soft_logout: true } | Self::UserLocked => {
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
