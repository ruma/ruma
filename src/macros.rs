macro_rules! common_impls {
    ($id:ident, $try_from:ident, $desc:literal) => {
        impl ::std::convert::From<$id<Box<str>>> for ::std::string::String {
            fn from(id: $id<Box<str>>) -> Self {
                id.full_id.into()
            }
        }

        impl<'a> ::std::convert::TryFrom<&'a str> for $id<&'a str> {
            type Error = crate::error::Error;

            fn try_from(s: &'a str) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        impl ::std::convert::TryFrom<&str> for $id<Box<str>> {
            type Error = crate::error::Error;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        impl ::std::convert::TryFrom<String> for $id<Box<str>> {
            type Error = crate::error::Error;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                $try_from(s)
            }
        }

        impl<T: ::std::convert::AsRef<str>> ::std::convert::AsRef<str> for $id<T> {
            fn as_ref(&self) -> &str {
                self.full_id.as_ref()
            }
        }

        impl<T: ::std::fmt::Display> ::std::fmt::Display for $id<T> {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.full_id)
            }
        }

        impl<T: ::std::cmp::PartialEq> ::std::cmp::PartialEq for $id<T> {
            fn eq(&self, other: &Self) -> bool {
                self.full_id == other.full_id
            }
        }

        impl<T: ::std::cmp::Eq> ::std::cmp::Eq for $id<T> {}

        impl<T: ::std::cmp::PartialOrd> ::std::cmp::PartialOrd for $id<T> {
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                ::std::cmp::PartialOrd::partial_cmp(&self.full_id, &other.full_id)
            }
        }

        impl<T: ::std::cmp::Ord> ::std::cmp::Ord for $id<T> {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                ::std::cmp::Ord::cmp(&self.full_id, &other.full_id)
            }
        }

        impl<T: ::std::hash::Hash> ::std::hash::Hash for $id<T> {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
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
        impl<'de> ::serde::Deserialize<'de> for $id<Box<str>> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                crate::deserialize_id(deserializer, $desc)
            }
        }

        impl<T: AsRef<str>> ::std::cmp::PartialEq<&str> for $id<T> {
            fn eq(&self, other: &&str) -> bool {
                self.full_id.as_ref() == *other
            }
        }

        impl<T: AsRef<str>> ::std::cmp::PartialEq<$id<T>> for &str {
            fn eq(&self, other: &$id<T>) -> bool {
                *self == other.full_id.as_ref()
            }
        }

        impl<T: AsRef<str>> ::std::cmp::PartialEq<::std::string::String> for $id<T> {
            fn eq(&self, other: &::std::string::String) -> bool {
                self.full_id.as_ref() == &other[..]
            }
        }

        impl<T: AsRef<str>> ::std::cmp::PartialEq<$id<T>> for ::std::string::String {
            fn eq(&self, other: &$id<T>) -> bool {
                &self[..] == other.full_id.as_ref()
            }
        }
    };
}
