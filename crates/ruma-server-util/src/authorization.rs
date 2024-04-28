//! Common types for implementing federation authorization.

use headers::{authorization::Credentials, HeaderValue};
use ruma_common::{OwnedServerName, OwnedServerSigningKeyId};
use tracing::debug;
use yap::{IntoTokens, TokenLocation, Tokens};

/// Typed representation of an `Authorization` header of scheme `X-Matrix`, as defined in the
/// [Matrix Server-Server API][spec]. Includes an implementation of
/// [`headers::authorization::Credentials`] for automatically handling the encoding and decoding
/// when using a web framework that supports typed headers.
///
/// [spec]: https://spec.matrix.org/latest/server-server-api/#request-authentication
#[non_exhaustive]
pub struct XMatrix {
    /// The server name of the sending server.
    pub origin: OwnedServerName,
    /// The server name of the receiving sender. For compatibility with older servers, recipients
    /// should accept requests without this parameter, but MUST always send it. If this property is
    /// included, but the value does not match the receiving server's name, the receiving server
    /// must deny the request with an HTTP status code 401 Unauthorized.
    pub destination: Option<OwnedServerName>,
    /// The ID - including the algorithm name - of the sending server's key that was used to sign
    /// the request.
    pub key: OwnedServerSigningKeyId,
    /// The signature of the JSON.
    pub sig: String,
}

impl XMatrix {
    /// Construct a new X-Matrix Authorization header.
    pub fn new(
        origin: OwnedServerName,
        destination: Option<OwnedServerName>,
        key: OwnedServerSigningKeyId,
        sig: String,
    ) -> Self {
        Self { origin, destination, key, sig }
    }
}

fn parse_token<'a>(tokens: &mut impl Tokens<Item = &'a u8>) -> Option<Vec<u8>> {
    tokens.optional(|t| {
        let token: Vec<u8> = t.take_while(|c| is_tchar(**c)).as_iter().copied().collect();
        if !token.is_empty() {
            Some(token)
        } else {
            debug!("Returning early because of empty token at {}", t.location().offset());
            None
        }
    })
}

fn parse_token_with_colons<'a>(tokens: &mut impl Tokens<Item = &'a u8>) -> Option<Vec<u8>> {
    tokens.optional(|t| {
        let token: Vec<u8> =
            t.take_while(|c| is_tchar(**c) || **c == b':').as_iter().copied().collect();
        if !token.is_empty() {
            Some(token)
        } else {
            debug!("Returning early because of empty token at {}", t.location().offset());
            None
        }
    })
}

fn parse_quoted<'a>(tokens: &mut impl Tokens<Item = &'a u8>) -> Option<Vec<u8>> {
    tokens.optional(|t| {
        if !(t.token(&b'"')) {
            return None;
        }
        let mut buffer = Vec::new();
        loop {
            match t.next()? {
                // quoted pair
                b'\\' => {
                    let escaped = t.next().filter(|c| {
                        if is_quoted_pair(**c) {
                            true
                        } else {
                            debug!(
                                "Encountered an illegal character {} at location {}",
                                **c as char,
                                t.location().offset()
                            );
                            false
                        }
                    })?;
                    buffer.push(*escaped);
                }
                // end of quote
                b'"' => break,
                // regular character
                c if is_qdtext(*c) => buffer.push(*c),
                // Invalid character
                c => {
                    debug!(
                        "Encountered an illegal character {} at location {}",
                        *c as char,
                        t.location().offset()
                    );
                    return None;
                }
            }
        }
        Some(buffer)
    })
}

fn parse_xmatrix_field<'a>(tokens: &mut impl Tokens<Item = &'a u8>) -> Option<(String, Vec<u8>)> {
    tokens.optional(|t| {
        let name = parse_token(t).and_then(|name| {
            let name = std::str::from_utf8(&name).ok()?.to_ascii_lowercase();
            match name.as_str() {
                "origin" | "destination" | "key" | "sig" => Some(name),
                name => {
                    debug!(
                        "Encountered an invalid field name {} at location {}",
                        name,
                        t.location().offset()
                    );
                    None
                }
            }
        })?;

        if !t.token(&b'=') {
            return None;
        }

        let value = parse_quoted(t).or_else(|| parse_token_with_colons(t))?;

        Some((name, value))
    })
}

fn parse_xmatrix<'a>(tokens: &mut impl Tokens<Item = &'a u8>) -> Option<XMatrix> {
    tokens.optional(|t| {
        if !t.tokens(b"X-Matrix ") {
            debug!("Failed to parse X-Matrix credentials, didn't start with 'X-Matrix '");
            return None;
        }
        let mut origin = None;
        let mut destination = None;
        let mut key = None;
        let mut sig = None;

        for (name, value) in t.sep_by(|t| parse_xmatrix_field(t), |t| t.token(&b',')).as_iter() {
            match name.as_str() {
                "origin" => {
                    if origin.is_some() {
                        debug!("Field origin duplicated in X-Matrix Authorization header");
                    }
                    origin = Some(std::str::from_utf8(&value).ok()?.try_into().ok()?);
                }
                "destination" => {
                    if destination.is_some() {
                        debug!("Field destination duplicated in X-Matrix Authorization header");
                    }
                    destination = Some(std::str::from_utf8(&value).ok()?.try_into().ok()?);
                }
                "key" => {
                    if key.is_some() {
                        debug!("Field key duplicated in X-Matrix Authorization header");
                    }
                    key = Some(std::str::from_utf8(&value).ok()?.try_into().ok()?);
                }
                "sig" => {
                    if sig.is_some() {
                        debug!("Field sig duplicated in X-Matrix Authorization header");
                    }
                    sig = Some(std::str::from_utf8(&value).ok()?.to_owned());
                }
                name => {
                    debug!("Unknown field {} found in X-Matrix Authorization header", name);
                }
            }
        }

        Some(XMatrix { origin: origin?, destination, key: key?, sig: sig? })
    })
}

fn is_alpha(c: u8) -> bool {
    (0x41..=0x5A).contains(&c) || (0x61..=0x7A).contains(&c)
}

fn is_digit(c: u8) -> bool {
    (0x30..=0x39).contains(&c)
}

fn is_tchar(c: u8) -> bool {
    const TOKEN_CHARS: [u8; 15] =
        [b'!', b'#', b'$', b'%', b'&', b'\'', b'*', b'+', b'-', b'.', b'^', b'_', b'`', b'|', b'~'];
    is_alpha(c) || is_digit(c) || TOKEN_CHARS.contains(&c)
}

fn is_qdtext(c: u8) -> bool {
    c == b'\t'
        || c == b' '
        || c == 0x21
        || (0x23..=0x5B).contains(&c)
        || (0x5D..=0x7E).contains(&c)
        || is_obs_text(c)
}

fn is_obs_text(c: u8) -> bool {
    c >= 0x80 // The spec does contain an upper limit of 0xFF here, but that's enforced by the type
}

fn is_vchar(c: u8) -> bool {
    (0x21..=0x7E).contains(&c)
}

fn is_quoted_pair(c: u8) -> bool {
    c == b'\t' || c == b' ' || is_vchar(c) || is_obs_text(c)
}

impl Credentials for XMatrix {
    const SCHEME: &'static str = "X-Matrix";

    fn decode(value: &HeaderValue) -> Option<Self> {
        let value: Vec<u8> = value.as_bytes().to_vec();
        parse_xmatrix(&mut value.into_tokens())
    }

    fn encode(&self) -> HeaderValue {
        if let Some(destination) = &self.destination {
            format!(
                "X-Matrix origin=\"{}\",destination=\"{destination}\",key=\"{}\",sig=\"{}\"",
                self.origin, self.key, self.sig
            )
        } else {
            format!("X-Matrix origin=\"{}\",key=\"{}\",sig=\"{}\"", self.origin, self.key, self.sig)
        }
        .try_into()
        .expect("header format is static")
    }
}

#[cfg(test)]
mod tests {
    use headers::{authorization::Credentials, HeaderValue};
    use ruma_common::OwnedServerName;

    use super::XMatrix;

    #[test]
    fn xmatrix_auth_pre_1_3() {
        let header = HeaderValue::from_static(
            "X-Matrix origin=\"origin.hs.example.com\",key=\"ed25519:key1\",sig=\"ABCDEF...\"",
        );
        let origin = "origin.hs.example.com".try_into().unwrap();
        let key = "ed25519:key1".try_into().unwrap();
        let sig = "ABCDEF...".to_owned();
        let credentials: XMatrix = Credentials::decode(&header).unwrap();
        assert_eq!(credentials.origin, origin);
        assert_eq!(credentials.destination, None);
        assert_eq!(credentials.key, key);
        assert_eq!(credentials.sig, sig);

        let credentials = XMatrix::new(origin, None, key, sig);

        assert_eq!(credentials.encode(), header);
    }

    #[test]
    fn xmatrix_auth_1_3() {
        let header = HeaderValue::from_static("X-Matrix origin=\"origin.hs.example.com\",destination=\"destination.hs.example.com\",key=\"ed25519:key1\",sig=\"ABCDEF...\"");
        let origin: OwnedServerName = "origin.hs.example.com".try_into().unwrap();
        let destination: OwnedServerName = "destination.hs.example.com".try_into().unwrap();
        let key = "ed25519:key1".try_into().unwrap();
        let sig = "ABCDEF...".to_owned();
        let credentials: XMatrix = Credentials::decode(&header).unwrap();
        assert_eq!(credentials.origin, origin);
        assert_eq!(credentials.destination, Some(destination.clone()));
        assert_eq!(credentials.key, key);
        assert_eq!(credentials.sig, sig);

        let credentials = XMatrix::new(origin, Some(destination), key, sig);

        assert_eq!(credentials.encode(), header);
    }
}
