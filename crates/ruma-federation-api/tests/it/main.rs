// This crate is not useful without either of those features, so export nothing if they are not
// enabled to avoid errors when running checks wrongly without enabling any of them.
#![cfg(any(feature = "client", feature = "server"))]

mod authentication;
