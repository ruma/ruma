macro_rules! impl_enum {
    ($name:ident { $($variant:ident => $s:expr,)+ }) => {
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
                let variant = match *self {
                    $($name::$variant => $s,)*
                };

                write!(f, "{}", variant)
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = $crate::ParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($s => Ok($name::$variant),)*
                    _ => Err($crate::ParseError),
                }
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: ::serde::Serializer {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: ::serde::Deserializer {
                deserializer.deserialize_str($crate::Visitor::new())
            }
        }

        #[cfg(test)]
        mod serialization_tests {
            use serde_json::{from_str, to_string};

            use super::$name;

            #[test]
            fn serialization_to_display_form() {
                $(assert_eq!(to_string(&$name::$variant).unwrap(), stringify!($s));)*
            }

            #[test]
            fn deserialization_from_display_form() {
                $(assert_eq!(from_str::<$name>(stringify!($s)).unwrap(), $name::$variant);)*
            }

            #[test]
            fn deserialization_fails_for_invalid_string_value() {
                assert!(from_str::<$name>(r#""invalid variant name""#).is_err());
            }
        }
    }
}
