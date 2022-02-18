//! Matrix URIs.

use std::fmt;

use percent_encoding::{percent_encode, AsciiSet, CONTROLS};

use crate::{EventId, ServerName};

const BASE_URL: &str = "https://matrix.to/#/";
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

/// A reference to a user, room or event.
///
/// Turn it into a `matrix.to` URL through its `Display` implementation (i.e. by
/// interpolating it in a formatting macro or via `.to_string()`).
#[derive(Debug, PartialEq, Eq)]
pub struct MatrixToUri<'a> {
    id: &'a str,
    event_id: Option<&'a EventId>,
    via: Vec<&'a ServerName>,
}

impl<'a> MatrixToUri<'a> {
    pub(crate) fn new(id: &'a str, via: Vec<&'a ServerName>) -> Self {
        Self { id, event_id: None, via }
    }

    pub(crate) fn event(id: &'a str, event_id: &'a EventId, via: Vec<&'a ServerName>) -> Self {
        Self { id, event_id: Some(event_id), via }
    }
}

impl<'a> fmt::Display for MatrixToUri<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(BASE_URL)?;
        write!(f, "{}", percent_encode(self.id.as_bytes(), TO_ENCODE))?;

        if let Some(ev_id) = self.event_id {
            write!(f, "/{}", percent_encode(ev_id.as_bytes(), TO_ENCODE))?;
        }

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
    use crate::user_id;

    #[test]
    fn matrix_to_uri() {
        assert_eq!(
            user_id!("@jplatte:notareal.hs").matrix_to_url().to_string(),
            "https://matrix.to/#/%40jplatte%3Anotareal.hs"
        );
    }
}
