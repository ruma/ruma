/// Declares an item with a doc attribute computed by some macro expression.
/// This allows documentation to be dynamically generated based on input.
/// Necessary to work around <https://github.com/rust-lang/rust/issues/52607>.
macro_rules! doc_concat {
    ( $( #[doc = $doc:expr] $( $thing:tt )* )* ) => ( $( #[doc = $doc] $( $thing )* )* );
}

macro_rules! partial_eq_string {
    ($id:ty $([$( $g:ident ),*])?) => {
        partial_eq_string!(@imp $(<$($g),*>)?, $id, str);
        partial_eq_string!(@imp $(<$($g),*>)?, $id, &str);
        partial_eq_string!(@imp $(<$($g),*>)?, $id, String);
        partial_eq_string!(@imp $(<$($g),*>)?, str, $id);
        partial_eq_string!(@imp $(<$($g),*>)?, &str, $id);
        partial_eq_string!(@imp $(<$($g),*>)?, String, $id);
    };
    (@imp $(<$( $g:ident ),*>)?, $l:ty, $r:ty) => {
        impl $(<$($g),*>)? PartialEq<$r> for $l {
            fn eq(&self, other: &$r) -> bool {
                AsRef::<str>::as_ref(self)
                    == AsRef::<str>::as_ref(other)
            }
        }
    }
}

macro_rules! owned_identifier {
    ($owned:ident, $id:ident) => {
        #[doc = concat!("Owned variant of ", stringify!($id))]
        ///
        /// The wrapper type for this type is variable, by default it'll use [`Box`],
        /// but you can change that by setting "`--cfg=ruma_identifiers_storage=...`" using
        /// `RUSTFLAGS` or `.cargo/config.toml` (under `[build]` -> `rustflags = ["..."]`)
        /// to the following;
        /// - `ruma_identifiers_storage="Arc"` to use [`Arc`](std::sync::Arc) as a wrapper type.
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $owned {
            #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
            inner: Box<$id>,
            #[cfg(ruma_identifiers_storage = "Arc")]
            inner: std::sync::Arc<$id>,
        }

        impl AsRef<$id> for $owned {
            fn as_ref(&self) -> &$id {
                &*self.inner
            }
        }

        impl AsRef<str> for $owned {
            fn as_ref(&self) -> &str {
                (*self.inner).as_ref()
            }
        }

        impl std::ops::Deref for $owned {
            type Target = $id;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl std::borrow::Borrow<$id> for $owned {
            fn borrow(&self) -> &$id {
                self.as_ref()
            }
        }

        impl From<&'_ $id> for $owned {
            fn from(id: &$id) -> $owned {
                $owned { inner: id.into() }
            }
        }

        impl From<Box<$id>> for $owned {
            fn from(b: Box<$id>) -> $owned {
                Self { inner: b.into() }
            }
        }

        impl From<std::sync::Arc<$id>> for $owned {
            fn from(a: std::sync::Arc<$id>) -> $owned {
                Self {
                    #[cfg(not(any(ruma_identifiers_storage = "Arc")))]
                    inner: a.as_ref().into(),
                    #[cfg(ruma_identifiers_storage = "Arc")]
                    inner: a,
                }
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for $owned {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_ref().as_str())
            }
        }

        partial_eq_string!($owned);

        impl PartialEq<Box<$id>> for $owned {
            fn eq(&self, other: &Box<$id>) -> bool {
                AsRef::<$id>::as_ref(self) == AsRef::<$id>::as_ref(other)
            }
        }

        impl PartialEq<$owned> for Box<$id> {
            fn eq(&self, other: &$owned) -> bool {
                AsRef::<$id>::as_ref(self) == AsRef::<$id>::as_ref(other)
            }
        }
    };
}

macro_rules! opaque_identifier_common_impls {
    ($id:ident, $owned:ident) => {
        impl $id {
            pub(super) fn from_borrowed(s: &str) -> &Self {
                unsafe { std::mem::transmute(s) }
            }

            pub(super) fn from_owned(s: Box<str>) -> Box<Self> {
                unsafe { Box::from_raw(Box::into_raw(s) as _) }
            }

            pub(super) fn from_rc(s: std::rc::Rc<str>) -> std::rc::Rc<Self> {
                unsafe { std::rc::Rc::from_raw(std::rc::Rc::into_raw(s) as _) }
            }

            pub(super) fn from_arc(s: std::sync::Arc<str>) -> std::sync::Arc<Self> {
                unsafe { std::sync::Arc::from_raw(std::sync::Arc::into_raw(s) as _) }
            }

            pub(super) fn into_owned(self: Box<Self>) -> Box<str> {
                unsafe { Box::from_raw(Box::into_raw(self) as _) }
            }

            doc_concat! {
                #[doc = concat!("Creates a string slice from this `", stringify!($id), "`.")]
                pub fn as_str(&self) -> &str {
                    &self.0
                }
            }

            doc_concat! {
                #[doc = concat!("Creates a byte slice from this `", stringify!($id), "`.")]
                pub fn as_bytes(&self) -> &[u8] {
                    self.0.as_bytes()
                }
            }
        }

        impl Clone for Box<$id> {
            fn clone(&self) -> Self {
                (**self).into()
            }
        }

        // impl ToOwned for $id {
        //     type Owned = $owned;

        //     fn to_owned(&self) -> Self::Owned {
        //         Self::from_owned(self.0.into()).into()
        //     }
        // }

        // TODO swap below with above after codebase has been converted
        //  to not use `to_owned` as equivalent to "into Box"
        impl ToOwned for $id {
            type Owned = Box<$id>;

            fn to_owned(&self) -> Self::Owned {
                Self::from_owned(self.0.into())
            }
        }

        impl AsRef<str> for $id {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl AsRef<str> for Box<$id> {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl From<&$id> for Box<$id> {
            fn from(id: &$id) -> Self {
                $id::from_owned(id.0.into())
            }
        }

        impl From<&$id> for std::rc::Rc<$id> {
            fn from(s: &$id) -> std::rc::Rc<$id> {
                let rc = std::rc::Rc::<str>::from(s.as_str());
                <$id>::from_rc(rc)
            }
        }

        impl From<&$id> for std::sync::Arc<$id> {
            fn from(s: &$id) -> std::sync::Arc<$id> {
                let arc = std::sync::Arc::<str>::from(s.as_str());
                <$id>::from_arc(arc)
            }
        }

        impl PartialEq<$id> for Box<$id> {
            fn eq(&self, other: &$id) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl PartialEq<&'_ $id> for Box<$id> {
            fn eq(&self, other: &&$id) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl PartialEq<Box<$id>> for $id {
            fn eq(&self, other: &Box<$id>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl PartialEq<Box<$id>> for &'_ $id {
            fn eq(&self, other: &Box<$id>) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl std::fmt::Debug for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <str as std::fmt::Debug>::fmt(self.as_str(), f)
            }
        }

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl serde::Serialize for $id {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        partial_eq_string!($id);
        partial_eq_string!(Box<$id>); // todo: Remove when all instances of Box have been converted to Owned
    };
}

macro_rules! opaque_identifier {
    ($id:ident, $owned:ident) => {
        opaque_identifier_common_impls!($id, $owned);

        impl<'a> From<&'a str> for &'a $id {
            fn from(s: &'a str) -> Self {
                $id::from_borrowed(s)
            }
        }

        impl From<&str> for Box<$id> {
            fn from(s: &str) -> Self {
                $id::from_owned(s.into())
            }
        }

        impl From<Box<str>> for Box<$id> {
            fn from(s: Box<str>) -> Self {
                $id::from_owned(s)
            }
        }

        impl From<String> for Box<$id> {
            fn from(s: String) -> Self {
                $id::from_owned(s.into())
            }
        }

        impl From<Box<$id>> for Box<str> {
            fn from(id: Box<$id>) -> Self {
                id.into_owned()
            }
        }

        impl From<Box<$id>> for String {
            fn from(id: Box<$id>) -> Self {
                id.into_owned().into()
            }
        }

        impl<'de> serde::Deserialize<'de> for Box<$id> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Box::<str>::deserialize(deserializer).map($id::from_owned)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $owned {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Box::<str>::deserialize(deserializer).map($id::from_owned).map(Into::into)
            }
        }
    };
}

macro_rules! opaque_identifier_validated {
    ($id:ident, $owned:ident, $validate_id:expr) => {
        impl $id {
            #[rustfmt::skip]
            doc_concat! {
                #[doc = concat!("\
                    Try parsing a `&str` into a `Box<", stringify!($id), ">`.\n\
                    \n\
                    The same can also be done using `FromStr`, `TryFrom` or `TryInto`.\n\
                    This function is simply more constrained and thus useful in generic contexts.\
                ")]
                pub fn parse(
                    s: impl AsRef<str> + Into<Box<str>>,
                ) -> Result<Box<Self>, crate::IdParseError> {
                    $validate_id(s.as_ref())?;
                    Ok($id::from_owned(s.into()))
                }
            }

            doc_concat! {
                #[doc = concat!("Try parsing a `&str` into an `Rc<", stringify!($id), ">`.")]
                pub fn parse_rc(
                    s: impl AsRef<str> + Into<std::rc::Rc<str>>,
                ) -> Result<std::rc::Rc<Self>, crate::IdParseError> {
                    $validate_id(s.as_ref())?;
                    Ok($id::from_rc(s.into()))
                }
            }

            doc_concat! {
                #[doc = concat!("Try parsing a `&str` into an `Arc<", stringify!($id), ">`.")]
                pub fn parse_arc(
                    s: impl AsRef<str> + Into<std::sync::Arc<str>>,
                ) -> Result<std::sync::Arc<Self>, crate::IdParseError> {
                    $validate_id(s.as_ref())?;
                    Ok($id::from_arc(s.into()))
                }
            }
        }

        opaque_identifier_common_impls!($id, $owned);

        impl From<Box<$id>> for String {
            fn from(id: Box<$id>) -> Self {
                id.into_owned().into()
            }
        }

        impl<'de> serde::Deserialize<'de> for Box<$id> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;

                let s = String::deserialize(deserializer)?;

                match $id::parse(s) {
                    Ok(o) => Ok(o),
                    Err(e) => Err(D::Error::custom(e)),
                }
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $owned {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;

                let s = String::deserialize(deserializer)?;

                match $id::parse(s) {
                    Ok(o) => Ok(o.into()),
                    Err(e) => Err(D::Error::custom(e)),
                }
            }
        }

        impl<'a> std::convert::TryFrom<&'a str> for &'a $id {
            type Error = crate::IdParseError;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                $validate_id(s)?;
                Ok($id::from_borrowed(s))
            }
        }

        impl std::str::FromStr for Box<$id> {
            type Err = crate::IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $id::parse(s)
            }
        }

        impl std::convert::TryFrom<&str> for Box<$id> {
            type Error = crate::IdParseError;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                $id::parse(s)
            }
        }

        impl std::convert::TryFrom<String> for Box<$id> {
            type Error = crate::IdParseError;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                $id::parse(s)
            }
        }
    };
}
