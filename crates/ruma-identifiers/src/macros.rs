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

macro_rules! as_str_based_impls {
    ($id:ty) => {
        impl AsRef<str> for $id {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl PartialEq for $id {
            fn eq(&self, other: &Self) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl Eq for $id {}

        impl PartialOrd for $id {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                PartialOrd::partial_cmp(self.as_str(), other.as_str())
            }
        }

        impl Ord for $id {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                Ord::cmp(self.as_str(), other.as_str())
            }
        }

        impl std::hash::Hash for $id {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.as_str().hash(state);
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for $id {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }
    };
}

macro_rules! common_impls {
    ($id:ty, $try_from:ident, $desc:literal) => {
        impl $id {
            doc_concat! {
                #[doc = concat!("Creates a string slice from this `", stringify!($id), "`")]
                pub fn as_str(&self) -> &str {
                    &self.full_id
                }
            }

            doc_concat! {
                #[doc = concat!("Creates a byte slice from this `", stringify!($id), "`")]
                pub fn as_bytes(&self) -> &[u8] {
                    self.full_id.as_bytes()
                }
            }

            doc_concat! {
                #[doc = concat!("Converts this `", stringify!($id), "` into a `String`")]
                pub fn into_string(self) -> String {
                    self.full_id.into()
                }
            }

            doc_concat! {
                #[doc = concat!("Converts this `", stringify!($id), "` into a `Vec<u8>`")]
                pub fn into_bytes(self) -> Vec<u8> {
                    Box::<[u8]>::from(self.full_id).into()
                }
            }
        }

        impl From<$id> for String {
            fn from(id: $id) -> Self {
                id.into_string()
            }
        }

        impl From<$id> for Vec<u8> {
            fn from(id: $id) -> Self {
                id.into_bytes()
            }
        }

        impl std::str::FromStr for $id {
            type Err = crate::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $try_from(s)
            }
        }

        impl std::convert::TryFrom<&str> for $id {
            type Error = crate::Error;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        impl std::convert::TryFrom<String> for $id {
            type Error = crate::Error;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $id {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                crate::deserialize_id(deserializer, $desc)
            }
        }

        as_str_based_impls!($id);
        partial_eq_string!($id);
    };
}

macro_rules! opaque_identifier_common_impls {
    ($id:ty) => {
        impl $id {
            fn from_borrowed(s: &str) -> &Self {
                unsafe { std::mem::transmute(s) }
            }

            pub(super) fn from_owned(s: Box<str>) -> Box<Self> {
                unsafe { Box::from_raw(Box::into_raw(s) as _) }
            }

            fn into_owned(self: Box<Self>) -> Box<str> {
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

        impl std::fmt::Debug for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <str as std::fmt::Debug>::fmt(self.as_str(), f)
            }
        }

        impl Clone for Box<$id> {
            fn clone(&self) -> Self {
                (**self).to_owned()
            }
        }

        impl ToOwned for $id {
            type Owned = Box<$id>;

            fn to_owned(&self) -> Self::Owned {
                Self::from_owned(self.0.into())
            }
        }

        impl From<&$id> for Box<$id> {
            fn from(id: &$id) -> Self {
                id.to_owned()
            }
        }

        impl AsRef<str> for Box<$id> {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl From<&$id> for std::rc::Rc<$id> {
            fn from(s: &$id) -> std::rc::Rc<$id> {
                let rc = std::rc::Rc::<str>::from(s.as_str());
                unsafe { std::rc::Rc::from_raw(std::rc::Rc::into_raw(rc) as *const $id) }
            }
        }

        impl From<&$id> for std::sync::Arc<$id> {
            fn from(s: &$id) -> std::sync::Arc<$id> {
                let arc = std::sync::Arc::<str>::from(s.as_str());
                unsafe { std::sync::Arc::from_raw(std::sync::Arc::into_raw(arc) as *const $id) }
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

        as_str_based_impls!($id);
        partial_eq_string!($id);
        partial_eq_string!(Box<$id>);
    };
}

macro_rules! opaque_identifier {
    (
        $( #[doc = $docs:literal] )*
        $vis:vis type $id:ident;
    ) => {
        $( #[doc = $docs] )*
        #[repr(transparent)]
        pub struct $id(str);

        opaque_identifier_common_impls!($id);

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

        impl From<String> for Box<$id> {
            fn from(s: String) -> Self {
                $id::from_owned(s.into())
            }
        }

        impl From<Box<$id>> for String {
            fn from(id: Box<$id>) -> Self {
                id.into_owned().into()
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for Box<$id> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Box::<str>::deserialize(deserializer).map($id::from_owned)
            }
        }
    };
}

macro_rules! opaque_identifier_validated {
    (
        $( #[doc = $docs:literal] )*
        $vis:vis type $id:ident [ $validate_id:ident ];
    ) => {
        $( #[doc = $docs] )*
        #[repr(transparent)]
        pub struct $id(str);

        opaque_identifier_common_impls!($id);

        impl From<Box<$id>> for String {
            fn from(id: Box<$id>) -> Self {
                id.into_owned().into()
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for Box<$id> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;

                let s = String::deserialize(deserializer)?;

                match try_from(s) {
                    Ok(o) => Ok(o),
                    Err(e) => Err(D::Error::custom(e)),
                }
            }
        }

        fn try_from<S>(s: S) -> Result<Box<$id>, crate::Error>
        where
            S: AsRef<str> + Into<Box<str>>,
        {
            $validate_id(s.as_ref())?;
            Ok($id::from_owned(s.into()))
        }

        impl<'a> std::convert::TryFrom<&'a str> for &'a $id {
            type Error = crate::Error;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                $validate_id(s)?;
                Ok($id::from_borrowed(s))
            }
        }

        impl std::str::FromStr for Box<$id> {
            type Err = crate::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                try_from(s)
            }
        }

        impl std::convert::TryFrom<&str> for Box<$id> {
            type Error = crate::Error;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                try_from(s)
            }
        }

        impl std::convert::TryFrom<String> for Box<$id> {
            type Error = crate::Error;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                try_from(s)
            }
        }
    }
}
