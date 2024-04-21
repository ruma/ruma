//! Matrix URIs.

use std::{fmt, str::FromStr};

use percent_encoding::{percent_decode_str, percent_encode};
use ruma_identifiers_validation::{
    error::{MatrixIdError, MatrixToError, MatrixUriError},
    Error,
};
use url::Url;

use super::{
    EventId, OwnedEventId, OwnedRoomAliasId, OwnedRoomId, OwnedRoomOrAliasId, OwnedServerName,
    OwnedUserId, RoomAliasId, RoomId, RoomOrAliasId, UserId,
};
use crate::{percent_encode::PATH_PERCENT_ENCODE_SET, PrivOwnedStr, ServerName};

const MATRIX_TO_BASE_URL: &str = "https://matrix.to/#/";
const MATRIX_SCHEME: &str = "matrix";

/// All Matrix Identifiers that can be represented as a Matrix URI.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MatrixId {
    /// A room ID.
    Room(OwnedRoomId),

    /// A room alias.
    RoomAlias(OwnedRoomAliasId),

    /// A user ID.
    User(OwnedUserId),

    /// An event ID.
    Event(OwnedRoomOrAliasId, OwnedEventId),
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

    /// Try parsing a `&str` with types into a `MatrixId`.
    ///
    /// The identifiers are expected to be in the format
    /// `type/identifier_without_sigil` and the identifier part is expected to
    /// be percent encoded. Slashes at the beginning and the end are stripped.
    ///
    /// For events, the room ID or alias and the event ID should be separated by
    /// a slash and they can be in any order.
    pub(crate) fn parse_with_type(s: &str) -> Result<Self, Error> {
        let s = if let Some(stripped) = s.strip_prefix('/') { stripped } else { s };
        let s = if let Some(stripped) = s.strip_suffix('/') { stripped } else { s };
        if s.is_empty() {
            return Err(MatrixIdError::NoIdentifier.into());
        }

        if ![1, 3].contains(&s.matches('/').count()) {
            return Err(MatrixIdError::InvalidPartsNumber.into());
        }

        let mut id = String::new();
        let mut split = s.split('/');
        while let (Some(type_), Some(id_without_sigil)) = (split.next(), split.next()) {
            let sigil = match type_ {
                "u" | "user" => '@',
                "r" | "room" => '#',
                "e" | "event" => '$',
                "roomid" => '!',
                _ => return Err(MatrixIdError::UnknownType.into()),
            };
            id = format!("{id}/{sigil}{id_without_sigil}");
        }

        Self::parse_with_sigil(&id)
    }

    /// Construct a string with sigils from `self`.
    ///
    /// The identifiers will start with a sigil and be percent encoded.
    ///
    /// For events, the room ID or alias and the event ID will be separated by
    /// a slash.
    pub(crate) fn to_string_with_sigil(&self) -> String {
        match self {
            Self::Room(room_id) => {
                percent_encode(room_id.as_bytes(), PATH_PERCENT_ENCODE_SET).to_string()
            }
            Self::RoomAlias(room_alias) => {
                percent_encode(room_alias.as_bytes(), PATH_PERCENT_ENCODE_SET).to_string()
            }
            Self::User(user_id) => {
                percent_encode(user_id.as_bytes(), PATH_PERCENT_ENCODE_SET).to_string()
            }
            Self::Event(room_id, event_id) => format!(
                "{}/{}",
                percent_encode(room_id.as_bytes(), PATH_PERCENT_ENCODE_SET),
                percent_encode(event_id.as_bytes(), PATH_PERCENT_ENCODE_SET),
            ),
        }
    }

    /// Construct a string with types from `self`.
    ///
    /// The identifiers will be in the format `type/identifier_without_sigil`
    /// and the identifier part will be percent encoded.
    ///
    /// For events, the room ID or alias and the event ID will be separated by
    /// a slash.
    pub(crate) fn to_string_with_type(&self) -> String {
        match self {
            Self::Room(room_id) => {
                format!(
                    "roomid/{}",
                    percent_encode(&room_id.as_bytes()[1..], PATH_PERCENT_ENCODE_SET)
                )
            }
            Self::RoomAlias(room_alias) => {
                format!(
                    "r/{}",
                    percent_encode(&room_alias.as_bytes()[1..], PATH_PERCENT_ENCODE_SET)
                )
            }
            Self::User(user_id) => {
                format!("u/{}", percent_encode(&user_id.as_bytes()[1..], PATH_PERCENT_ENCODE_SET))
            }
            Self::Event(room_id, event_id) => {
                let room_type = if room_id.is_room_id() { "roomid" } else { "r" };
                format!(
                    "{}/{}/e/{}",
                    room_type,
                    percent_encode(&room_id.as_bytes()[1..], PATH_PERCENT_ENCODE_SET),
                    percent_encode(&event_id.as_bytes()[1..], PATH_PERCENT_ENCODE_SET),
                )
            }
        }
    }
}

impl From<OwnedRoomId> for MatrixId {
    fn from(room_id: OwnedRoomId) -> Self {
        Self::Room(room_id)
    }
}

impl From<&RoomId> for MatrixId {
    fn from(room_id: &RoomId) -> Self {
        room_id.to_owned().into()
    }
}

impl From<OwnedRoomAliasId> for MatrixId {
    fn from(room_alias: OwnedRoomAliasId) -> Self {
        Self::RoomAlias(room_alias)
    }
}

impl From<&RoomAliasId> for MatrixId {
    fn from(room_alias: &RoomAliasId) -> Self {
        room_alias.to_owned().into()
    }
}

impl From<OwnedUserId> for MatrixId {
    fn from(user_id: OwnedUserId) -> Self {
        Self::User(user_id)
    }
}

impl From<&UserId> for MatrixId {
    fn from(user_id: &UserId) -> Self {
        user_id.to_owned().into()
    }
}

impl From<(OwnedRoomOrAliasId, OwnedEventId)> for MatrixId {
    fn from(ids: (OwnedRoomOrAliasId, OwnedEventId)) -> Self {
        Self::Event(ids.0, ids.1)
    }
}

impl From<(&RoomOrAliasId, &EventId)> for MatrixId {
    fn from(ids: (&RoomOrAliasId, &EventId)) -> Self {
        (ids.0.to_owned(), ids.1.to_owned()).into()
    }
}

impl From<(OwnedRoomId, OwnedEventId)> for MatrixId {
    fn from(ids: (OwnedRoomId, OwnedEventId)) -> Self {
        Self::Event(ids.0.into(), ids.1)
    }
}

impl From<(&RoomId, &EventId)> for MatrixId {
    fn from(ids: (&RoomId, &EventId)) -> Self {
        (ids.0.to_owned(), ids.1.to_owned()).into()
    }
}

impl From<(OwnedRoomAliasId, OwnedEventId)> for MatrixId {
    fn from(ids: (OwnedRoomAliasId, OwnedEventId)) -> Self {
        Self::Event(ids.0.into(), ids.1)
    }
}

impl From<(&RoomAliasId, &EventId)> for MatrixId {
    fn from(ids: (&RoomAliasId, &EventId)) -> Self {
        (ids.0.to_owned(), ids.1.to_owned()).into()
    }
}

/// The [`matrix.to` URI] representation of a user, room or event.
///
/// Get the URI through its `Display` implementation (i.e. by interpolating it
/// in a formatting macro or via `.to_string()`).
///
/// [`matrix.to` URI]: https://spec.matrix.org/latest/appendices/#matrixto-navigation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixToUri {
    id: MatrixId,
    via: Vec<OwnedServerName>,
}

impl MatrixToUri {
    pub(crate) fn new(id: MatrixId, via: Vec<OwnedServerName>) -> Self {
        Self { id, via }
    }

    /// The identifier represented by this `matrix.to` URI.
    pub fn id(&self) -> &MatrixId {
        &self.id
    }

    /// Matrix servers usable to route a `RoomId`.
    pub fn via(&self) -> &[OwnedServerName] {
        &self.via
    }

    /// Try parsing a `&str` into a `MatrixToUri`.
    pub fn parse(s: &str) -> Result<Self, Error> {
        // We do not rely on parsing with `url::Url` because the meaningful part
        // of the URI is in its fragment part.
        //
        // Even if the fragment part looks like parts of a URI, non-url-encoded
        // room aliases (starting with `#`) could be detected as fragments,
        // messing up the URI parsing.
        //
        // A matrix.to URI looks like this: https://matrix.to/#/{MatrixId}?{query};
        // where the MatrixId should be percent-encoded, but might not, and the query
        // should also be percent-encoded.

        let s = s.strip_prefix(MATRIX_TO_BASE_URL).ok_or(MatrixToError::WrongBaseUrl)?;
        let s = s.strip_suffix('/').unwrap_or(s);

        // Separate the identifiers and the query.
        let mut parts = s.split('?');

        let ids_part = parts.next().expect("a split iterator yields at least one value");
        let id = MatrixId::parse_with_sigil(ids_part)?;

        // Parse the query for routing arguments.
        let via = parts
            .next()
            .map(|query| {
                // `form_urlencoded` takes care of percent-decoding the query.
                let query_parts = form_urlencoded::parse(query.as_bytes());

                query_parts
                    .map(|(key, value)| {
                        (key == "via")
                            .then(|| ServerName::parse(&value))
                            .unwrap_or_else(|| Err(MatrixToError::UnknownArgument.into()))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        // That would mean there are two `?` in the URL which is not valid.
        if parts.next().is_some() {
            return Err(MatrixToError::InvalidUrl.into());
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

impl FromStr for MatrixToUri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// The intent of a Matrix URI.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum UriAction {
    /// Join the room referenced by the URI.
    ///
    /// The client should prompt for confirmation prior to joining the room, if
    /// the user isn’t already part of the room.
    Join,

    /// Start a direct chat with the user referenced by the URI.
    ///
    /// Clients supporting a form of Canonical DMs should reuse existing DMs
    /// instead of creating new ones if available. The client should prompt for
    /// confirmation prior to creating the DM, if the user isn’t being
    /// redirected to an existing canonical DM.
    Chat,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl UriAction {
    /// Creates a string slice from this `UriAction`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    fn from<T>(s: T) -> Self
    where
        T: AsRef<str> + Into<Box<str>>,
    {
        match s.as_ref() {
            "join" => UriAction::Join,
            "chat" => UriAction::Chat,
            _ => UriAction::_Custom(PrivOwnedStr(s.into())),
        }
    }
}

impl AsRef<str> for UriAction {
    fn as_ref(&self) -> &str {
        match self {
            UriAction::Join => "join",
            UriAction::Chat => "chat",
            UriAction::_Custom(s) => s.0.as_ref(),
        }
    }
}

impl fmt::Display for UriAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())?;
        Ok(())
    }
}

impl From<&str> for UriAction {
    fn from(s: &str) -> Self {
        Self::from(s)
    }
}

impl From<String> for UriAction {
    fn from(s: String) -> Self {
        Self::from(s)
    }
}

impl From<Box<str>> for UriAction {
    fn from(s: Box<str>) -> Self {
        Self::from(s)
    }
}

/// The [`matrix:` URI] representation of a user, room or event.
///
/// Get the URI through its `Display` implementation (i.e. by interpolating it
/// in a formatting macro or via `.to_string()`).
///
/// [`matrix:` URI]: https://spec.matrix.org/latest/appendices/#matrix-uri-scheme
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixUri {
    id: MatrixId,
    via: Vec<OwnedServerName>,
    action: Option<UriAction>,
}

impl MatrixUri {
    pub(crate) fn new(id: MatrixId, via: Vec<OwnedServerName>, action: Option<UriAction>) -> Self {
        Self { id, via, action }
    }

    /// The identifier represented by this `matrix:` URI.
    pub fn id(&self) -> &MatrixId {
        &self.id
    }

    /// Matrix servers usable to route a `RoomId`.
    pub fn via(&self) -> &[OwnedServerName] {
        &self.via
    }

    /// The intent of this URI.
    pub fn action(&self) -> Option<&UriAction> {
        self.action.as_ref()
    }

    /// Try parsing a `&str` into a `MatrixUri`.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let url = Url::parse(s).map_err(|_| MatrixToError::InvalidUrl)?;

        if url.scheme() != MATRIX_SCHEME {
            return Err(MatrixUriError::WrongScheme.into());
        }

        let id = MatrixId::parse_with_type(url.path())?;

        let mut via = vec![];
        let mut action = None;

        for (key, value) in url.query_pairs() {
            if key.as_ref() == "via" {
                via.push(value.parse()?);
            } else if key.as_ref() == "action" {
                if action.is_some() {
                    return Err(MatrixUriError::TooManyActions.into());
                };

                action = Some(value.as_ref().into());
            } else {
                return Err(MatrixUriError::UnknownQueryItem.into());
            }
        }

        Ok(Self { id, via, action })
    }
}

impl fmt::Display for MatrixUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{MATRIX_SCHEME}:{}", self.id().to_string_with_type())?;

        let mut first = true;
        for server_name in &self.via {
            f.write_str(if first { "?via=" } else { "&via=" })?;
            f.write_str(server_name.as_str())?;

            first = false;
        }

        if let Some(action) = self.action() {
            f.write_str(if first { "?action=" } else { "&action=" })?;
            f.write_str(action.as_str())?;
        }

        Ok(())
    }
}

impl TryFrom<&str> for MatrixUri {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl FromStr for MatrixUri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_identifiers_validation::{
        error::{MatrixIdError, MatrixToError, MatrixUriError},
        Error,
    };

    use super::{MatrixId, MatrixToUri, MatrixUri};
    use crate::{
        event_id, matrix_uri::UriAction, room_alias_id, room_id, server_name, user_id,
        RoomOrAliasId,
    };

    #[test]
    fn display_matrixtouri() {
        assert_eq!(
            user_id!("@jplatte:notareal.hs").matrix_to_uri().to_string(),
            "https://matrix.to/#/@jplatte:notareal.hs"
        );
        assert_eq!(
            room_alias_id!("#ruma:notareal.hs").matrix_to_uri().to_string(),
            "https://matrix.to/#/%23ruma:notareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs").matrix_to_uri().to_string(),
            "https://matrix.to/#/!ruma:notareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_to_uri_via(vec![server_name!("notareal.hs")])
                .to_string(),
            "https://matrix.to/#/!ruma:notareal.hs?via=notareal.hs"
        );
        assert_eq!(
            room_alias_id!("#ruma:notareal.hs")
                .matrix_to_event_uri(event_id!("$event:notareal.hs"))
                .to_string(),
            "https://matrix.to/#/%23ruma:notareal.hs/$event:notareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_to_event_uri(event_id!("$event:notareal.hs"))
                .to_string(),
            "https://matrix.to/#/!ruma:notareal.hs/$event:notareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_to_event_uri_via(
                    event_id!("$event:notareal.hs"),
                    vec![server_name!("notareal.hs")]
                )
                .to_string(),
            "https://matrix.to/#/!ruma:notareal.hs/$event:notareal.hs?via=notareal.hs"
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

        let matrix_to = MatrixToUri::parse(
            "https://matrix.to/#/%21ruma%3Anotareal.hs?via=notareal.hs&via=anotherunreal.hs",
        )
        .expect("Failed to create MatrixToUri.");
        assert_eq!(matrix_to.id(), &room_id!("!ruma:notareal.hs").into());
        assert_eq!(
            matrix_to.via(),
            &[server_name!("notareal.hs").to_owned(), server_name!("anotherunreal.hs").to_owned(),]
        );

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
        assert_eq!(matrix_to.via().len(), 0);
    }

    #[test]
    fn parse_matrixtouri_valid_uris_not_urlencoded() {
        let matrix_to = MatrixToUri::parse("https://matrix.to/#/@jplatte:notareal.hs")
            .expect("Failed to create MatrixToUri.");
        assert_eq!(matrix_to.id(), &user_id!("@jplatte:notareal.hs").into());

        let matrix_to = MatrixToUri::parse("https://matrix.to/#/#ruma:notareal.hs")
            .expect("Failed to create MatrixToUri.");
        assert_eq!(matrix_to.id(), &room_alias_id!("#ruma:notareal.hs").into());

        let matrix_to = MatrixToUri::parse("https://matrix.to/#/!ruma:notareal.hs?via=notareal.hs")
            .expect("Failed to create MatrixToUri.");
        assert_eq!(matrix_to.id(), &room_id!("!ruma:notareal.hs").into());
        assert_eq!(matrix_to.via(), &[server_name!("notareal.hs").to_owned()]);

        let matrix_to =
            MatrixToUri::parse("https://matrix.to/#/#ruma:notareal.hs/$event:notareal.hs")
                .expect("Failed to create MatrixToUri.");
        assert_eq!(
            matrix_to.id(),
            &(room_alias_id!("#ruma:notareal.hs"), event_id!("$event:notareal.hs")).into()
        );

        let matrix_to =
            MatrixToUri::parse("https://matrix.to/#/!ruma:notareal.hs/$event:notareal.hs")
                .expect("Failed to create MatrixToUri.");
        assert_eq!(
            matrix_to.id(),
            &(room_id!("!ruma:notareal.hs"), event_id!("$event:notareal.hs")).into()
        );
        assert_eq!(matrix_to.via().len(), 0);
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
        );
    }

    #[test]
    fn display_matrixuri() {
        assert_eq!(
            user_id!("@jplatte:notareal.hs").matrix_uri(false).to_string(),
            "matrix:u/jplatte:notareal.hs"
        );
        assert_eq!(
            user_id!("@jplatte:notareal.hs").matrix_uri(true).to_string(),
            "matrix:u/jplatte:notareal.hs?action=chat"
        );
        assert_eq!(
            room_alias_id!("#ruma:notareal.hs").matrix_uri(false).to_string(),
            "matrix:r/ruma:notareal.hs"
        );
        assert_eq!(
            room_alias_id!("#ruma:notareal.hs").matrix_uri(true).to_string(),
            "matrix:r/ruma:notareal.hs?action=join"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs").matrix_uri(false).to_string(),
            "matrix:roomid/ruma:notareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_uri_via(vec![server_name!("notareal.hs")], false)
                .to_string(),
            "matrix:roomid/ruma:notareal.hs?via=notareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_uri_via(
                    vec![server_name!("notareal.hs"), server_name!("anotherunreal.hs")],
                    true
                )
                .to_string(),
            "matrix:roomid/ruma:notareal.hs?via=notareal.hs&via=anotherunreal.hs&action=join"
        );
        assert_eq!(
            room_alias_id!("#ruma:notareal.hs")
                .matrix_event_uri(event_id!("$event:notareal.hs"))
                .to_string(),
            "matrix:r/ruma:notareal.hs/e/event:notareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_event_uri(event_id!("$event:notareal.hs"))
                .to_string(),
            "matrix:roomid/ruma:notareal.hs/e/event:notareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_event_uri_via(
                    event_id!("$event:notareal.hs"),
                    vec![server_name!("notareal.hs")]
                )
                .to_string(),
            "matrix:roomid/ruma:notareal.hs/e/event:notareal.hs?via=notareal.hs"
        );
    }

    #[test]
    fn parse_valid_matrixid_with_type() {
        assert_eq!(
            MatrixId::parse_with_type("u/user:imaginary.hs").expect("Failed to create MatrixId."),
            MatrixId::User(user_id!("@user:imaginary.hs").into())
        );
        assert_eq!(
            MatrixId::parse_with_type("user/user:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::User(user_id!("@user:imaginary.hs").into())
        );
        assert_eq!(
            MatrixId::parse_with_type("roomid/roomid:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Room(room_id!("!roomid:imaginary.hs").into())
        );
        assert_eq!(
            MatrixId::parse_with_type("r/roomalias:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::RoomAlias(room_alias_id!("#roomalias:imaginary.hs").into())
        );
        assert_eq!(
            MatrixId::parse_with_type("room/roomalias:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::RoomAlias(room_alias_id!("#roomalias:imaginary.hs").into())
        );
        assert_eq!(
            MatrixId::parse_with_type("roomid/roomid:imaginary.hs/e/event:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Event(
                <&RoomOrAliasId>::from(room_id!("!roomid:imaginary.hs")).into(),
                event_id!("$event:imaginary.hs").into()
            )
        );
        assert_eq!(
            MatrixId::parse_with_type("r/roomalias:imaginary.hs/e/event:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Event(
                <&RoomOrAliasId>::from(room_alias_id!("#roomalias:imaginary.hs")).into(),
                event_id!("$event:imaginary.hs").into()
            )
        );
        assert_eq!(
            MatrixId::parse_with_type("room/roomalias:imaginary.hs/event/event:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Event(
                <&RoomOrAliasId>::from(room_alias_id!("#roomalias:imaginary.hs")).into(),
                event_id!("$event:imaginary.hs").into()
            )
        );
        // Invert the order of the event and the room.
        assert_eq!(
            MatrixId::parse_with_type("e/event:imaginary.hs/roomid/roomid:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Event(
                <&RoomOrAliasId>::from(room_id!("!roomid:imaginary.hs")).into(),
                event_id!("$event:imaginary.hs").into()
            )
        );
        assert_eq!(
            MatrixId::parse_with_type("e/event:imaginary.hs/r/roomalias:imaginary.hs")
                .expect("Failed to create MatrixId."),
            MatrixId::Event(
                <&RoomOrAliasId>::from(room_alias_id!("#roomalias:imaginary.hs")).into(),
                event_id!("$event:imaginary.hs").into()
            )
        );
        // Starting with a slash
        assert_eq!(
            MatrixId::parse_with_type("/u/user:imaginary.hs").expect("Failed to create MatrixId."),
            MatrixId::User(user_id!("@user:imaginary.hs").into())
        );
        // Ending with a slash
        assert_eq!(
            MatrixId::parse_with_type("roomid/roomid:imaginary.hs/")
                .expect("Failed to create MatrixId."),
            MatrixId::Room(room_id!("!roomid:imaginary.hs").into())
        );
        // Starting and ending with a slash
        assert_eq!(
            MatrixId::parse_with_type("/r/roomalias:imaginary.hs/")
                .expect("Failed to create MatrixId."),
            MatrixId::RoomAlias(room_alias_id!("#roomalias:imaginary.hs").into())
        );
    }

    #[test]
    fn parse_matrixid_type_no_identifier() {
        assert_eq!(MatrixId::parse_with_type("").unwrap_err(), MatrixIdError::NoIdentifier.into());
        assert_eq!(MatrixId::parse_with_type("/").unwrap_err(), MatrixIdError::NoIdentifier.into());
    }

    #[test]
    fn parse_matrixid_invalid_parts_number() {
        assert_eq!(
            MatrixId::parse_with_type("u/user:imaginary.hs/r/room:imaginary.hs/e").unwrap_err(),
            MatrixIdError::InvalidPartsNumber.into()
        );
    }

    #[test]
    fn parse_matrixid_unknown_type() {
        assert_eq!(
            MatrixId::parse_with_type("notatype/fake:notareal.hs").unwrap_err(),
            MatrixIdError::UnknownType.into()
        );
    }

    #[test]
    fn parse_matrixuri_valid_uris() {
        let matrix_uri =
            MatrixUri::parse("matrix:u/jplatte:notareal.hs").expect("Failed to create MatrixUri.");
        assert_eq!(matrix_uri.id(), &user_id!("@jplatte:notareal.hs").into());
        assert_eq!(matrix_uri.action(), None);

        let matrix_uri = MatrixUri::parse("matrix:u/jplatte:notareal.hs?action=chat")
            .expect("Failed to create MatrixUri.");
        assert_eq!(matrix_uri.id(), &user_id!("@jplatte:notareal.hs").into());
        assert_eq!(matrix_uri.action(), Some(&UriAction::Chat));

        let matrix_uri =
            MatrixUri::parse("matrix:r/ruma:notareal.hs").expect("Failed to create MatrixToUri.");
        assert_eq!(matrix_uri.id(), &room_alias_id!("#ruma:notareal.hs").into());

        let matrix_uri = MatrixUri::parse("matrix:roomid/ruma:notareal.hs?via=notareal.hs")
            .expect("Failed to create MatrixToUri.");
        assert_eq!(matrix_uri.id(), &room_id!("!ruma:notareal.hs").into());
        assert_eq!(matrix_uri.via(), &[server_name!("notareal.hs").to_owned()]);
        assert_eq!(matrix_uri.action(), None);

        let matrix_uri = MatrixUri::parse("matrix:r/ruma:notareal.hs/e/event:notareal.hs")
            .expect("Failed to create MatrixToUri.");
        assert_eq!(
            matrix_uri.id(),
            &(room_alias_id!("#ruma:notareal.hs"), event_id!("$event:notareal.hs")).into()
        );

        let matrix_uri = MatrixUri::parse("matrix:roomid/ruma:notareal.hs/e/event:notareal.hs")
            .expect("Failed to create MatrixToUri.");
        assert_eq!(
            matrix_uri.id(),
            &(room_id!("!ruma:notareal.hs"), event_id!("$event:notareal.hs")).into()
        );
        assert_eq!(matrix_uri.via().len(), 0);
        assert_eq!(matrix_uri.action(), None);

        let matrix_uri =
            MatrixUri::parse("matrix:roomid/ruma:notareal.hs/e/event:notareal.hs?via=notareal.hs&action=join&via=anotherinexistant.hs")
                .expect("Failed to create MatrixToUri.");
        assert_eq!(
            matrix_uri.id(),
            &(room_id!("!ruma:notareal.hs"), event_id!("$event:notareal.hs")).into()
        );
        assert_eq!(
            matrix_uri.via(),
            &vec![
                server_name!("notareal.hs").to_owned(),
                server_name!("anotherinexistant.hs").to_owned()
            ]
        );
        assert_eq!(matrix_uri.action(), Some(&UriAction::Join));
    }

    #[test]
    fn parse_matrixuri_invalid_uri() {
        assert_eq!(
            MatrixUri::parse("").unwrap_err(),
            Error::InvalidMatrixToUri(MatrixToError::InvalidUrl)
        );
    }

    #[test]
    fn parse_matrixuri_wrong_scheme() {
        assert_eq!(
            MatrixUri::parse("unknown:u/user:notareal.hs").unwrap_err(),
            MatrixUriError::WrongScheme.into()
        );
    }

    #[test]
    fn parse_matrixuri_too_many_actions() {
        assert_eq!(
            MatrixUri::parse("matrix:u/user:notareal.hs?action=chat&action=join").unwrap_err(),
            MatrixUriError::TooManyActions.into()
        );
    }

    #[test]
    fn parse_matrixuri_unknown_query_item() {
        assert_eq!(
            MatrixUri::parse("matrix:roomid/roomid:notareal.hs?via=notareal.hs&fake=data")
                .unwrap_err(),
            MatrixUriError::UnknownQueryItem.into()
        );
    }

    #[test]
    fn parse_matrixuri_wrong_identifier() {
        assert_matches!(
            MatrixUri::parse("matrix:notanidentifier").unwrap_err(),
            Error::InvalidMatrixId(_)
        );
        assert_matches!(MatrixUri::parse("matrix:").unwrap_err(), Error::InvalidMatrixId(_));
        assert_matches!(
            MatrixUri::parse("matrix:u/jplatte:notareal.hs/e/event:notareal.hs").unwrap_err(),
            Error::InvalidMatrixId(_)
        );
    }
}
