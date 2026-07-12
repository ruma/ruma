use std::convert::Infallible;

use bytes::BufMut;

use crate::serde::slice_to_buf;

/// HTTP message body pre-serialization.
pub trait OutgoingBody {
    /// The type of error that can happen in `try_info_buf`.
    type Error;

    /// Turn `self` into a byte buffer (copying a raw body or serializing a JSON one).
    fn try_into_buf<T: Default + BufMut + AsRef<[u8]>>(self) -> Result<T, Self::Error>;
}

/// "Empty" body type, used mostly for GET requests.
///
/// If `TRULY_EMPTY` is `true`, serializes to an empty buffer.
/// If `TRULY_EMPTY` is `false`, serializes to an empty JSON object.
/// (that case is not encoded as a separate type due to macro requirements)
#[expect(clippy::exhaustive_structs)]
pub struct EmptyBody<const TRULY_EMPTY: bool = true>;

impl<const TRULY_EMPTY: bool> OutgoingBody for EmptyBody<TRULY_EMPTY> {
    type Error = Infallible;

    fn try_into_buf<T: Default + BufMut + AsRef<[u8]>>(self) -> Result<T, Infallible> {
        if TRULY_EMPTY { Ok(Default::default()) } else { Ok(slice_to_buf(b"{}")) }
    }
}

/// Raw-bytes body type, used for some media endpoints.
#[expect(clippy::exhaustive_structs)]
pub struct BytesBody(pub Vec<u8>);

impl OutgoingBody for BytesBody {
    type Error = Infallible;

    fn try_into_buf<T: Default + BufMut + AsRef<[u8]>>(self) -> Result<T, Infallible> {
        Ok(slice_to_buf(&self.0))
    }
}
