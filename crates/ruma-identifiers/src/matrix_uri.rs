//! Matrix URIs.

use std::fmt;

use percent_encoding::{percent_encode, AsciiSet, CONTROLS};

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

/// The `matrix.to` URI representation of a user, room or event.
///
/// Get the URI through its `Display` implementation (i.e. by interpolating it
/// in a formatting macro or via `.to_string()`).
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

#[cfg(test)]
mod tests {
    use crate::{event_id, room_alias_id, room_id, server_name, user_id};

    #[test]
    fn display_matrixtouri() {
        assert_eq!(
            user_id!("@jplatte:notareal.hs").matrix_to_url().to_string(),
            "https://matrix.to/#/%40jplatte%3Anotareal.hs"
        );
        assert_eq!(
            room_alias_id!("#ruma:notareal.hs").matrix_to_url().to_string(),
            "https://matrix.to/#/%23ruma%3Anotareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_to_url(vec![server_name!("notareal.hs")])
                .to_string(),
            "https://matrix.to/#/%21ruma%3Anotareal.hs?via=notareal.hs"
        );
        assert_eq!(
            room_alias_id!("#ruma:notareal.hs")
                .matrix_to_event_url(event_id!("$event:notareal.hs"))
                .to_string(),
            "https://matrix.to/#/%23ruma%3Anotareal.hs/%24event%3Anotareal.hs"
        );
        assert_eq!(
            room_id!("!ruma:notareal.hs")
                .matrix_to_event_url(event_id!("$event:notareal.hs"))
                .to_string(),
            "https://matrix.to/#/%21ruma%3Anotareal.hs/%24event%3Anotareal.hs"
        );
    }
}
