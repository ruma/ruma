macro_rules! common_impls {
    ($id:ident, $desc:literal) => {
        impl ::std::convert::From<$id> for ::std::string::String {
            fn from(id: $id) -> Self {
                id.full_id
            }
        }

        impl ::std::convert::TryFrom<&str> for $id {
            type Error = crate::error::Error;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                Self::try_from(::std::borrow::Cow::Borrowed(s))
            }
        }

        impl ::std::convert::TryFrom<String> for $id {
            type Error = crate::error::Error;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                Self::try_from(::std::borrow::Cow::Owned(s))
            }
        }

        impl ::std::convert::AsRef<str> for $id {
            fn as_ref(&self) -> &str {
                &self.full_id
            }
        }

        impl ::std::fmt::Display for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.full_id)
            }
        }

        impl ::std::cmp::PartialEq for $id {
            fn eq(&self, other: &Self) -> bool {
                self.full_id == other.full_id
            }
        }

        impl ::std::cmp::Eq for $id {}

        impl ::std::cmp::PartialOrd for $id {
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                <::std::string::String as ::std::cmp::PartialOrd>::partial_cmp(
                    &self.full_id,
                    &other.full_id,
                )
            }
        }

        impl ::std::cmp::Ord for $id {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                <::std::string::String as ::std::cmp::Ord>::cmp(&self.full_id, &other.full_id)
            }
        }

        impl ::std::hash::Hash for $id {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                self.full_id.hash(state);
            }
        }

        impl ::serde::Serialize for $id {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(&self.full_id)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $id {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                crate::deserialize_id(deserializer, $desc)
            }
        }

        impl ::std::cmp::PartialEq<str> for $id {
            fn eq(&self, other: &str) -> bool {
                self.full_id == other
            }
        }

        impl ::std::cmp::PartialEq<$id> for str {
            fn eq(&self, other: &$id) -> bool {
                self == other.full_id
            }
        }

        impl ::std::cmp::PartialEq<::std::string::String> for $id {
            fn eq(&self, other: &::std::string::String) -> bool {
                &self.full_id == other
            }
        }

        impl ::std::cmp::PartialEq<$id> for ::string::String {
            fn eq(&self, other: &$id) -> bool {
                self == &other.full_id
            }
        }
    };
}
