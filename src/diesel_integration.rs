//! Implements traits from Diesel, allowing identifiers to be used as database fields.

use std::{convert::TryFrom, error::Error as StdError, io::Write};

use diesel::{
    backend::Backend,
    deserialize::{FromSql, Result as DeserializeResult},
    serialize::{Output, Result as SerializeResult, ToSql},
    sql_types::Text,
};

macro_rules! diesel_impl {
    ($name:ident) => {
        impl<DB> ToSql<Text, DB> for $crate::$name
        where
            DB: Backend,
        {
            fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> SerializeResult {
                ToSql::<Text, DB>::to_sql(&self.to_string(), out)
            }
        }

        impl<DB> FromSql<Text, DB> for $crate::$name
        where
            String: FromSql<Text, DB>,
            DB: Backend,
        {
            fn from_sql(value: Option<&<DB as Backend>::RawValue>) -> DeserializeResult<Self> {
                let string = <String as FromSql<Text, DB>>::from_sql(value)?;
                Self::try_from(string.as_str())
                    .map_err(|error| Box::new(error) as Box<StdError + Send + Sync>)
            }
        }
    };
}

diesel_impl!(EventId);
diesel_impl!(RoomAliasId);
diesel_impl!(RoomId);
diesel_impl!(RoomIdOrAliasId);
diesel_impl!(UserId);
