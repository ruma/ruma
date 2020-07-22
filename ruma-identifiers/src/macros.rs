/// Declares an item with a doc attribute computed by some macro expression.
/// This allows documentation to be dynamically generated based on input.
/// Necessary to work around https://github.com/rust-lang/rust/issues/52607.
macro_rules! doc_concat {
    ( $( #[doc = $doc:expr] $( $thing:tt )* )* ) => ( $( #[doc = $doc] $( $thing )* )* );
}

macro_rules! partial_eq_string {
    ($id:ty) => {
        partial_eq_string!(@imp, $id, str);
        partial_eq_string!(@imp, $id, &str);
        partial_eq_string!(@imp, $id, String);
        partial_eq_string!(@imp, str, $id);
        partial_eq_string!(@imp, &str, $id);
        partial_eq_string!(@imp, String, $id);
    };
    (@imp, $l:ty, $r:ty) => {
        impl ::std::cmp::PartialEq<$r> for $l {
            fn eq(&self, other: &$r) -> bool {
                ::std::convert::AsRef::<str>::as_ref(self)
                    == ::std::convert::AsRef::<str>::as_ref(other)
            }
        }
    }
}

macro_rules! common_impls {
    ($id:ty, $try_from:ident, $desc:literal) => {
        impl $id {
            doc_concat! {
                #[doc = concat!("Creates a string slice from this `", stringify!($id), "`")]
                pub fn as_str(&self) -> &str {
                    self.full_id.as_ref()
                }
            }
        }

        impl ::std::convert::AsRef<str> for $id {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl ::std::convert::From<$id> for ::std::string::String {
            fn from(id: $id) -> Self {
                id.full_id.into()
            }
        }

        impl ::std::convert::TryFrom<&str> for $id {
            type Error = crate::error::Error;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        impl ::std::convert::TryFrom<String> for $id {
            type Error = crate::error::Error;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        impl ::std::fmt::Display for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl ::std::cmp::PartialEq for $id {
            fn eq(&self, other: &Self) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl ::std::cmp::Eq for $id {}

        impl ::std::cmp::PartialOrd for $id {
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                ::std::cmp::PartialOrd::partial_cmp(self.as_str(), other.as_str())
            }
        }

        impl ::std::cmp::Ord for $id {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                ::std::cmp::Ord::cmp(self.as_str(), other.as_str())
            }
        }

        impl ::std::hash::Hash for $id {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                self.as_str().hash(state);
            }
        }

        #[cfg(feature = "serde")]
        impl ::serde::Serialize for $id {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for $id {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                crate::deserialize_id(deserializer, $desc)
            }
        }

        partial_eq_string!($id);
    };
}
