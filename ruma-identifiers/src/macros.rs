/// Declares an item with a doc attribute computed by some macro expression.
/// This allows documentation to be dynamically generated based on input.
/// Necessary to work around https://github.com/rust-lang/rust/issues/52607.
macro_rules! doc_concat {
    ( $( #[doc = $doc:expr] $thing:item )* ) => ( $( #[doc = $doc] $thing )* );
}

macro_rules! common_impls {
    ($id:ident, $try_from:ident, $desc:literal) => {
        impl<T: ::core::convert::AsRef<str>> $id<T> {
            doc_concat! {
                #[doc = concat!("Creates a string slice from this `", stringify!($id), "`")]
                pub fn as_str(&self) -> &str {
                    self.full_id.as_ref()
                }
            }
        }

        #[cfg(feature = "alloc")]
        impl<'a> ::core::convert::From<&'a $id<::alloc::boxed::Box<str>>> for $id<&'a str> {
            fn from(id: &'a $id<::alloc::boxed::Box<str>>) -> Self {
                id.as_ref()
            }
        }

        #[cfg(feature = "alloc")]
        impl ::core::convert::From<$id<::alloc::boxed::Box<str>>> for ::alloc::string::String {
            fn from(id: $id<::alloc::boxed::Box<str>>) -> Self {
                id.full_id.into()
            }
        }

        impl<'a> ::core::convert::TryFrom<&'a str> for $id<&'a str> {
            type Error = crate::error::Error;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        impl ::core::convert::TryFrom<&str> for $id<::alloc::boxed::Box<str>> {
            type Error = crate::error::Error;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        impl ::core::convert::TryFrom<::alloc::string::String> for $id<::alloc::boxed::Box<str>> {
            type Error = crate::error::Error;

            fn try_from(s: ::alloc::string::String) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        impl<T: ::core::convert::AsRef<str>> ::core::convert::AsRef<str> for $id<T> {
            fn as_ref(&self) -> &str {
                self.full_id.as_ref()
            }
        }

        impl<T: ::core::fmt::Display> ::core::fmt::Display for $id<T> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(f, "{}", self.full_id)
            }
        }

        impl<T: ::core::cmp::PartialEq> ::core::cmp::PartialEq for $id<T> {
            fn eq(&self, other: &Self) -> bool {
                self.full_id == other.full_id
            }
        }

        impl<T: ::core::cmp::Eq> ::core::cmp::Eq for $id<T> {}

        impl<T: ::core::cmp::PartialOrd> ::core::cmp::PartialOrd for $id<T> {
            fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
                ::core::cmp::PartialOrd::partial_cmp(&self.full_id, &other.full_id)
            }
        }

        impl<T: ::core::cmp::Ord> ::core::cmp::Ord for $id<T> {
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                ::core::cmp::Ord::cmp(&self.full_id, &other.full_id)
            }
        }

        impl<T: ::core::hash::Hash> ::core::hash::Hash for $id<T> {
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.full_id.hash(state);
            }
        }

        #[cfg(feature = "serde")]
        impl<T: AsRef<str>> ::serde::Serialize for $id<T> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(self.full_id.as_ref())
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for $id<::alloc::boxed::Box<str>> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                crate::deserialize_id(deserializer, $desc)
            }
        }

        impl<T: AsRef<str>> ::core::cmp::PartialEq<&str> for $id<T> {
            fn eq(&self, other: &&str) -> bool {
                self.full_id.as_ref() == *other
            }
        }

        impl<T: AsRef<str>> ::core::cmp::PartialEq<$id<T>> for &str {
            fn eq(&self, other: &$id<T>) -> bool {
                *self == other.full_id.as_ref()
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: AsRef<str>> ::core::cmp::PartialEq<::alloc::string::String> for $id<T> {
            fn eq(&self, other: &::alloc::string::String) -> bool {
                self.full_id.as_ref() == &other[..]
            }
        }

        impl<T: AsRef<str>> ::core::cmp::PartialEq<$id<T>> for ::alloc::string::String {
            fn eq(&self, other: &$id<T>) -> bool {
                &self[..] == other.full_id.as_ref()
            }
        }
    };
}
