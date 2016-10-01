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
    }
}
