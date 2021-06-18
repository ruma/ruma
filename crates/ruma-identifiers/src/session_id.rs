//! Matrix session ID.
use std::{convert::TryFrom, fmt, mem, rc::Rc, str::FromStr, sync::Arc};

use ruma_identifiers_validation::session_id::validate;

/// A session ID.
///
/// Session IDs in Matrix are opaque character sequences of `[0-9a-zA-Z.=_-]. Their length must
/// must not exceed 255 characters.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent, crate = "serde"))]
pub struct SessionId(str);

/// An owned Session Id.
pub type SessionIdBox = Box<SessionId>;

impl SessionId {
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

    /// Creates a string slice from this `SessionId`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates a byte slice from this `SessionId`.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Clone for Box<SessionId> {
    fn clone(&self) -> Self {
        (**self).to_owned()
    }
}

impl ToOwned for SessionId {
    type Owned = Box<SessionId>;

    fn to_owned(&self) -> Self::Owned {
        Self::from_owned(self.0.to_owned().into_boxed_str())
    }
}

impl From<&SessionId> for Rc<SessionId> {
    fn from(s: &SessionId) -> Self {
        let rc = Rc::<str>::from(s.as_str());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const SessionId) }
    }
}

impl From<&SessionId> for Arc<SessionId> {
    fn from(s: &SessionId) -> Self {
        let arc = Arc::<str>::from(s.as_str());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const SessionId) }
    }
}

fn try_from<S>(session_id: S) -> Result<Box<SessionId>, crate::Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    validate(session_id.as_ref())?;
    Ok(SessionId::from_owned(session_id.into()))
}

impl AsRef<str> for SessionId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Box<SessionId> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Box<SessionId>> for String {
    fn from(s: Box<SessionId>) -> Self {
        s.into_owned().into()
    }
}

impl<'a> TryFrom<&'a str> for &'a SessionId {
    type Error = crate::Error;

    fn try_from(session_id: &'a str) -> Result<Self, Self::Error> {
        validate(session_id)?;
        Ok(SessionId::from_borrowed(session_id))
    }
}

impl FromStr for Box<SessionId> {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        try_from(s)
    }
}

impl TryFrom<&str> for Box<SessionId> {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl TryFrom<String> for Box<SessionId> {
    type Error = crate::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Box<SessionId> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::deserialize_id(deserializer, "A valid session ID")
    }
}

partial_eq_string!(SessionId);
partial_eq_string!(Box<SessionId>);
