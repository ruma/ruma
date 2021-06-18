//! Client secret identifier.

use std::{convert::TryFrom, fmt, mem, rc::Rc, str::FromStr, sync::Arc};

use ruma_identifiers_validation::client_secret::validate;

/// A client secret.
///
/// Session IDs in Matrix are opaque character sequences of `[0-9a-zA-Z.=_-]. Their length must
/// must not exceed 255 characters.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent, crate = "serde"))]
pub struct ClientSecret(str);

/// An owned client secret.
pub type ClientSecretBox = Box<ClientSecret>;

impl ClientSecret {
    #[allow(clippy::transmute_ptr_to_ptr)]
    fn from_borrowed(s: &str) -> &Self {
        unsafe { mem::transmute(s) }
    }

    fn from_owned(s: Box<str>) -> Box<Self> {
        unsafe { mem::transmute(s) }
    }

    fn into_owned(self: Box<Self>) -> Box<str> {
        unsafe { mem::transmute(self) }
    }

    /// Creates a string slice from this `ClientSecret`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a byte slice from this `ClientSecret`.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for ClientSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Clone for Box<ClientSecret> {
    fn clone(&self) -> Self {
        (**self).to_owned()
    }
}

impl ToOwned for ClientSecret {
    type Owned = Box<ClientSecret>;

    fn to_owned(&self) -> Self::Owned {
        Self::from_owned(self.0.to_owned().into_boxed_str())
    }
}

impl From<&ClientSecret> for Box<ClientSecret> {
    fn from(s: &ClientSecret) -> Self {
        s.to_owned()
    }
}

impl From<&ClientSecret> for Rc<ClientSecret> {
    fn from(s: &ClientSecret) -> Self {
        let rc = Rc::<str>::from(s.as_str());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const ClientSecret) }
    }
}

impl From<&ClientSecret> for Arc<ClientSecret> {
    fn from(s: &ClientSecret) -> Self {
        let arc = Arc::<str>::from(s.as_str());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const ClientSecret) }
    }
}

fn try_from<S>(client_secret: S) -> Result<Box<ClientSecret>, crate::Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    validate(client_secret.as_ref())?;
    Ok(ClientSecret::from_owned(client_secret.into()))
}

impl AsRef<str> for ClientSecret {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Box<ClientSecret> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Box<ClientSecret>> for String {
    fn from(s: Box<ClientSecret>) -> Self {
        s.into_owned().into()
    }
}

impl<'a> TryFrom<&'a str> for &'a ClientSecret {
    type Error = crate::Error;

    fn try_from(client_secret: &'a str) -> Result<Self, Self::Error> {
        validate(client_secret)?;
        Ok(ClientSecret::from_borrowed(client_secret))
    }
}

impl FromStr for Box<ClientSecret> {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        try_from(s)
    }
}

impl TryFrom<&str> for Box<ClientSecret> {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl TryFrom<String> for Box<ClientSecret> {
    type Error = crate::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl fmt::Display for ClientSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Box<ClientSecret> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::deserialize_id(deserializer, "A client secret")
    }
}

partial_eq_string!(ClientSecret);
partial_eq_string!(Box<ClientSecret>);

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::ClientSecret;

    #[test]
    fn valid_secret() {
        assert!(<&ClientSecret>::try_from("this_=_a_valid_secret_1337").is_ok())
    }
}
