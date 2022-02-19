//! Matrix URIs.

use std::{convert::TryFrom, fmt};

use percent_encoding::{percent_decode_str, percent_encode, AsciiSet, CONTROLS};
use ruma_identifiers_validation::{
    error::{MatrixIdError, MatrixToError},
    Error,
};
use url::Url;

use crate::{EventId, RoomAliasId, RoomId, RoomOrAliasId, ServerName, UserId};

const MATRIX_TO_BASE_URL: &str = "https://matrix.to/#/";
// Controls + Space + reserved characters from RFC 3986. In practice only the
// reserved characters will be encountered most likely, but better be safe.
// https://datatracker.ietf.org/doc/html/rfc3986/#page-13
const TO_ENCODE: &AsciiSet = &CONTROLS
    .add(b':')
    .add(b'/')
    .add(b'?')
    .add(b'#')
    .add(b'[')
    .add(b']')
    .add(b'@')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b';')
    .add(b'=');

/// All Matrix Identifiers that can be represented as a Matrix URI.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MatrixId {
    /// A room ID.
    Room(Box<RoomId>),

    /// A room alias.
    RoomAlias(Box<RoomAliasId>),

    /// A user ID.
    User(Box<UserId>),

    /// An event ID.
    Event(Box<RoomOrAliasId>, Box<EventId>),
}

impl MatrixId {
    /// Try parsing a `&str` with sigils into a `MatrixId`.
    ///
    /// The identifiers are expected to start with a sigil and to be percent
    /// encoded. Slashes at the beginning and the end are stripped.
    ///
    /// For events, the room ID or alias and the event ID should be separated by
    /// a slash and they can be in any order.
    pub(crate) fn parse_with_sigil(s: &str) -> Result<Self, Error> {
        let s = if let Some(stripped) = s.strip_prefix('/') { stripped } else { s };
        let s = if let Some(stripped) = s.strip_suffix('/') { stripped } else { s };
        if s.is_empty() {
            return Err(MatrixIdError::NoIdentifier.into());
        }

        if s.matches('/').count() > 1 {
            return Err(MatrixIdError::TooManyIdentifiers.into());
        }

        if let Some((first_raw, second_raw)) = s.split_once('/') {
            let first = percent_decode_str(first_raw).decode_utf8()?;
            let second = percent_decode_str(second_raw).decode_utf8()?;

            match first.as_bytes()[0] {
                b'!' | b'#' if second.as_bytes()[0] == b'$' => {
                    let room_id = <&RoomOrAliasId>::try_from(first.as_ref())?;
                    let event_id = <&EventId>::try_from(second.as_ref())?;
                    Ok((room_id, event_id).into())
                }
                b'$' if matches!(second.as_bytes()[0], b'!' | b'#') => {
                    let room_id = <&RoomOrAliasId>::try_from(second.as_ref())?;
                    let event_id = <&EventId>::try_from(first.as_ref())?;
                    Ok((room_id, event_id).into())
                }
                _ => Err(MatrixIdError::UnknownIdentifierPair.into()),
            }
        } else {
            let id = percent_decode_str(s).decode_utf8()?;

            match id.as_bytes()[0] {
                b'@' => Ok(<&UserId>::try_from(id.as_ref())?.into()),
                b'!' => Ok(<&RoomId>::try_from(id.as_ref())?.into()),
                b'#' => Ok(<&RoomAliasId>::try_from(id.as_ref())?.into()),
                b'$' => Err(MatrixIdError::MissingRoom.into()),
                _ => Err(MatrixIdError::UnknownIdentifier.into()),
            }
        }
    }

    /// Construct a string with sigils from `self`.
    ///
    /// The identifiers will start with a sigil and be percent encoded.
    ///
    /// For events, the room ID or alias and the event ID will be separated by
    /// a slash.
    pub(crate) fn to_string_with_sigil(&self) -> String {
        match self {
            Self::Room(room_id) => percent_encode(room_id.as_bytes(), TO_ENCODE).to_string(),
            Self::RoomAlias(room_alias) => {
                percent_encode(room_alias.as_bytes(), TO_ENCODE).to_string()
            }
            Self::User(user_id) => percent_encode(user_id.as_bytes(), TO_ENCODE).to_string(),
            Self::Event(room_id, event_id) => format!(
                "{}/{}",
                percent_encode(room_id.as_bytes(), TO_ENCODE),
                percent_encode(event_id.as_bytes(), TO_ENCODE),
            ),
        }
    }
}

impl From<&RoomId> for MatrixId {
    fn from(room_id: &RoomId) -> Self {
        Self::Room(room_id.into())
    }
}

impl From<&RoomAliasId> for MatrixId {
    fn from(room_alias: &RoomAliasId) -> Self {
        Self::RoomAlias(room_alias.into())
    }
}

impl From<&UserId> for MatrixId {
    fn from(user_id: &UserId) -> Self {
        Self::User(user_id.into())
    }
}

impl From<(&RoomOrAliasId, &EventId)> for MatrixId {
    fn from(ids: (&RoomOrAliasId, &EventId)) -> Self {
        Self::Event(ids.0.into(), ids.1.into())
    }
}

impl From<(&RoomId, &EventId)> for MatrixId {
    fn from(ids: (&RoomId, &EventId)) -> Self {
        Self::Event(<&RoomOrAliasId>::from(ids.0).into(), ids.1.into())
    }
}

impl From<(&RoomAliasId, &EventId)> for MatrixId {
    fn from(ids: (&RoomAliasId, &EventId)) -> Self {
        Self::Event(<&RoomOrAliasId>::from(ids.0).into(), ids.1.into())
    }
}

/// The [`matrix.to` URI] representation of a user, room or event.
///
/// Get the URI through its `Display` implementation (i.e. by interpolating it
/// in a formatting macro or via `.to_string()`).
///
/// [`matrix.to` URI]: https://spec.matrix.org/v1.2/appendices/#matrixto-navigation
#[derive(Debug, PartialEq, Eq)]
pub struct MatrixToUri {
    id: MatrixId,
    via: Vec<Box<ServerName>>,
}

impl MatrixToUri {
    pub(crate) fn new(id: MatrixId, via: Vec<&ServerName>) -> Self {
        Self { id, via: via.into_iter().map(ToOwned::to_owned).collect() }
    }

    /// The identifier represented by this `matrix.to` URI.
    pub fn id(&self) -> &MatrixId {
        &self.id
    }

    /// Matrix servers usable to route a `RoomId`.
    pub fn via(&self) -> &[Box<ServerName>] {
        &self.via
    }

    /// Try parsing a `&str` into a `MatrixToUri`.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let without_base = if let Some(stripped) = s.strip_prefix(MATRIX_TO_BASE_URL) {
            stripped
        } else {
            return Err(MatrixToError::WrongBaseUrl.into());
        };

        let url = Url::parse(MATRIX_TO_BASE_URL.trim_end_matches("#/"))?.join(without_base)?;

        let id = MatrixId::parse_with_sigil(url.path())?;
        let mut via = vec![];

        for (key, value) in url.query_pairs() {
            if key.as_ref() == "via" {
                via.push(ServerName::parse(value)?);
            } else {
                return Err(MatrixToError::UnknownArgument.into());
            }
        }

        Ok(Self { id, via })
    }
}

impl fmt::Display for MatrixToUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(MATRIX_TO_BASE_URL)?;
        write!(f, "{}", self.id().to_string_with_sigil())?;

        let mut first = true;
        for server_name in &self.via {
            f.write_str(if first { "?via=" } else { "&via=" })?;
            f.write_str(server_name.as_str())?;

            first = false;
        }

        Ok(())
    }
}

impl TryFrom<&str> for MatrixToUri {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use ruma_identifiers_validation::{
        error::{MatrixIdError, MatrixToError},
        Error,
    };

    use super::{MatrixId, MatrixToUri};
    use crate::{event_id, room_alias_id, room_id, server_name, user_id, RoomOrAliasId};

    #[test]
    fn display_matrixtouri() {
        assert_eq!(
            user_id!("@jplatte:notareal.hs").matrix_to_uri().to_string(),
            "https://matrix.to/#/%40jplatte%3Anotareal.hs"
        );
        assert_eq!(
            room_alias_id!("#ruma:notareal.hs").matrix_to_uri().to_string(),
            "https://matrix.to/#/%23ruma%3Anotareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_to_uri(vec![server_name!("notareal.hs")])
                .to_string(),
            "https://matrix.to/#/%21ruma%3Anotareal.hs?via=notareal.hs"
        );
        assert_eq!(
            room_alias_id!("#ruma:notareal.hs")
                .matrix_to_event_uri(event_id!("$event:notareal.hs"))
                .to_string(),
            "https://matrix.to/#/%23ruma%3Anotareal.hs/%24event%3Anotareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_to_event_uri(event_id!("$event:notareal.hs"))
                .to_string(),
            "https://matrix.to/#/%21ruma%3Anotareal.hs/%24event%3Anotareal.hs"
        );
    }

    #[test]
    fn parse_valid_matrixid_with_sigil() {
        assert_eq!(
            MatrixId::parse_with_sigil("@user:imaginary.hs").expect("Failed to create MatrixId."),
            MatrixId::User(user_id!("@user:imaginary.hs").into())
        );
        assert_eq!(
            MatrixId::parse_with_sigil("!roomid:imaginary.hs").expect("Failed to create MatrixId."),
            MatrixId::Room(room_id!("!roomid:imaginary.hs").into())
        );
        assert_eq!(
            MatrixId::parse_with_sigil("#roomalias:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::RoomAlias(room_alias_id!("#roomalias:imaginary.hs").into())
        );
        assert_eq!(
            MatrixId::parse_with_sigil("!roomid:imaginary.hs/$event:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Event(
                <&RoomOrAliasId>::from(room_id!("!roomid:imaginary.hs")).into(),
                event_id!("$event:imaginary.hs").into()
            )
        );
        assert_eq!(
            MatrixId::parse_with_sigil("#roomalias:imaginary.hs/$event:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Event(
                <&RoomOrAliasId>::from(room_alias_id!("#roomalias:imaginary.hs")).into(),
                event_id!("$event:imaginary.hs").into()
            )
        );
        // Invert the order of the event and the room.
        assert_eq!(
            MatrixId::parse_with_sigil("$event:imaginary.hs/!roomid:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Event(
                <&RoomOrAliasId>::from(room_id!("!roomid:imaginary.hs")).into(),
                event_id!("$event:imaginary.hs").into()
            )
        );
        assert_eq!(
            MatrixId::parse_with_sigil("$event:imaginary.hs/#roomalias:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Event(
                <&RoomOrAliasId>::from(room_alias_id!("#roomalias:imaginary.hs")).into(),
                event_id!("$event:imaginary.hs").into()
            )
        );
        // Starting with a slash
        assert_eq!(
            MatrixId::parse_with_sigil("/@user:imaginary.hs").expect("Failed to create MatrixId."),
            MatrixId::User(user_id!("@user:imaginary.hs").into())
        );
        // Ending with a slash
        assert_eq!(
            MatrixId::parse_with_sigil("!roomid:imaginary.hs/")
                .expect("Failed to create MatrixId."),
            MatrixId::Room(room_id!("!roomid:imaginary.hs").into())
        );
        // Starting and ending with a slash
        assert_eq!(
            MatrixId::parse_with_sigil("/#roomalias:imaginary.hs/")
                .expect("Failed to create MatrixId."),
            MatrixId::RoomAlias(room_alias_id!("#roomalias:imaginary.hs").into())
        );
    }

    #[test]
    fn parse_matrixid_no_identifier() {
        assert_eq!(MatrixId::parse_with_sigil("").unwrap_err(), MatrixIdError::NoIdentifier.into());
        assert_eq!(
            MatrixId::parse_with_sigil("/").unwrap_err(),
            MatrixIdError::NoIdentifier.into()
        );
    }

    #[test]
    fn parse_matrixid_too_many_identifiers() {
        assert_eq!(
            MatrixId::parse_with_sigil(
                "@user:imaginary.hs/#room:imaginary.hs/$event1:imaginary.hs"
            )
            .unwrap_err(),
            MatrixIdError::TooManyIdentifiers.into()
        );
    }

    #[test]
    fn parse_matrixid_unknown_identifier_pair() {
        assert_eq!(
            MatrixId::parse_with_sigil("!roomid:imaginary.hs/@user:imaginary.hs").unwrap_err(),
            MatrixIdError::UnknownIdentifierPair.into()
        );
        assert_eq!(
            MatrixId::parse_with_sigil("#roomalias:imaginary.hs/notanidentifier").unwrap_err(),
            MatrixIdError::UnknownIdentifierPair.into()
        );
        assert_eq!(
            MatrixId::parse_with_sigil("$event:imaginary.hs/$otherevent:imaginary.hs").unwrap_err(),
            MatrixIdError::UnknownIdentifierPair.into()
        );
        assert_eq!(
            MatrixId::parse_with_sigil("notanidentifier/neitheristhis").unwrap_err(),
            MatrixIdError::UnknownIdentifierPair.into()
        );
    }

    #[test]
    fn parse_matrixid_missing_room() {
        assert_eq!(
            MatrixId::parse_with_sigil("$event:imaginary.hs").unwrap_err(),
            MatrixIdError::MissingRoom.into()
        );
    }

    #[test]
    fn parse_matrixid_unknown_identifier() {
        assert_eq!(
            MatrixId::parse_with_sigil("event:imaginary.hs").unwrap_err(),
            MatrixIdError::UnknownIdentifier.into()
        );
        assert_eq!(
            MatrixId::parse_with_sigil("notanidentifier").unwrap_err(),
            MatrixIdError::UnknownIdentifier.into()
        );
    }

    #[test]
    fn parse_matrixtouri_valid_uris() {
        let matrix_to = MatrixToUri::parse("https://matrix.to/#/%40jplatte%3Anotareal.hs")
            .expect("Failed to create MatrixToUri.");
        assert_eq!(matrix_to.id(), &user_id!("@jplatte:notareal.hs").into());

        let matrix_to = MatrixToUri::parse("https://matrix.to/#/%23ruma%3Anotareal.hs")
            .expect("Failed to create MatrixToUri.");
        assert_eq!(matrix_to.id(), &room_alias_id!("#ruma:notareal.hs").into());

        let matrix_to =
            MatrixToUri::parse("https://matrix.to/#/%21ruma%3Anotareal.hs?via=notareal.hs")
                .expect("Failed to create MatrixToUri.");
        assert_eq!(matrix_to.id(), &room_id!("!ruma:notareal.hs").into());
        assert_eq!(matrix_to.via(), &vec![server_name!("notareal.hs").to_owned()]);

        let matrix_to =
            MatrixToUri::parse("https://matrix.to/#/%23ruma%3Anotareal.hs/%24event%3Anotareal.hs")
                .expect("Failed to create MatrixToUri.");
        assert_eq!(
            matrix_to.id(),
            &(room_alias_id!("#ruma:notareal.hs"), event_id!("$event:notareal.hs")).into()
        );

        let matrix_to =
            MatrixToUri::parse("https://matrix.to/#/%21ruma%3Anotareal.hs/%24event%3Anotareal.hs")
                .expect("Failed to create MatrixToUri.");
        assert_eq!(
            matrix_to.id(),
            &(room_id!("!ruma:notareal.hs"), event_id!("$event:notareal.hs")).into()
        );
        assert!(matrix_to.via().is_empty());
    }

    #[test]
    fn parse_matrixtouri_wrong_base_url() {
        assert_eq!(MatrixToUri::parse("").unwrap_err(), MatrixToError::WrongBaseUrl.into());
        assert_eq!(
            MatrixToUri::parse("https://notreal.to/#/").unwrap_err(),
            MatrixToError::WrongBaseUrl.into()
        );
    }

    #[test]
    fn parse_matrixtouri_wrong_identifier() {
        assert_matches!(
            MatrixToUri::parse("https://matrix.to/#/notanidentifier").unwrap_err(),
            Error::InvalidMatrixId(_)
        );
        assert_matches!(
            MatrixToUri::parse("https://matrix.to/#/").unwrap_err(),
            Error::InvalidMatrixId(_)
        );
        assert_matches!(
            MatrixToUri::parse(
                "https://matrix.to/#/%40jplatte%3Anotareal.hs/%24event%3Anotareal.hs"
            )
            .unwrap_err(),
            Error::InvalidMatrixId(_)
        );
    }

    #[test]
    fn parse_matrixtouri_unknown_arguments() {
        assert_eq!(
            MatrixToUri::parse(
                "https://matrix.to/#/%21ruma%3Anotareal.hs?via=notareal.hs&custom=data"
            )
            .unwrap_err(),
            MatrixToError::UnknownArgument.into()
        )
    }
}
