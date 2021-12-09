use bytes::BufMut;
use ruma_serde::Outgoing;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::IntoHttpError;

/// TODO: DOCS
#[allow(clippy::exhaustive_structs)]
#[derive(Outgoing)]
#[incoming_derive(!Deserialize)]
pub struct RawHttpBody<'a>(pub &'a [u8]);

/// Types that can be converted to the raw bytes of an http request or response body.
pub trait IntoHttpBody {
    /// Serialize `self` to a buffer of the given type.
    fn to_buf<B>(&self) -> Result<B, IntoHttpError>
    where
        B: Default + BufMut;

    // FIXME: To support alternative transports, the user should be allowed to pass a serialization
    // function similar to a `for<T: Serialize> FnOnce(&mut buf::Writer<B>, T) -> Result<B, E>`
    // where `E: Into<IntoHttpError>`. To make this work without higher-ranked trait bounds which
    // are only implemented for lifetimes so far, a separate trait will have to be involved.
}

impl<T: Serialize> IntoHttpBody for T {
    fn to_buf<B>(&self) -> Result<B, IntoHttpError>
    where
        B: Default + BufMut,
    {
        let mut buf = B::default().writer();
        serde_json::to_writer(&mut buf, &self)?;
        Ok(buf.into_inner())
    }
}

impl IntoHttpBody for RawHttpBody<'_> {
    fn to_buf<B>(&self) -> Result<B, IntoHttpError>
    where
        B: Default + BufMut,
    {
        let mut buf = B::default();
        buf.put_slice(self.0);
        Ok(buf)
    }
}

// Until the borrowing type can be used in responses as well.
impl IntoHttpBody for IncomingRawHttpBody {
    fn to_buf<B>(&self) -> Result<B, IntoHttpError>
    where
        B: Default + BufMut,
    {
        let mut buf = B::default();
        buf.put_slice(&self.0);
        Ok(buf)
    }
}

/// Types that can be converted from the raw bytes of an http request or response body.
pub trait FromHttpBody<Error>: Sized {
    /// Deserialize `Self` from the given bytes.
    fn from_buf(body: &[u8]) -> Result<Self, Error>;
}

impl<T: DeserializeOwned, Error> FromHttpBody<Error> for T
where
    Error: From<serde_json::Error>,
{
    fn from_buf(body: &[u8]) -> Result<Self, Error> {
        Ok(serde_json::from_slice(body)?)
    }
}

impl<Error> FromHttpBody<Error> for IncomingRawHttpBody {
    fn from_buf(body: &[u8]) -> Result<Self, Error> {
        Ok(Self(body.to_owned()))
    }
}
