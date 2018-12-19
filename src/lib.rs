//! Crate ruma_client_api contains serializable types for the requests and responses for each
//! endpoint in the [Matrix](https://matrix.org/) client API specification. These types can be
//! shared by client and server code.

#![deny(missing_debug_implementations, missing_docs)]
#![feature(try_from)]

use futures;
use http;
use hyper;
use ruma_api;
use serde;
use serde_json;
use serde_urlencoded;
use url;

pub mod r0;
pub mod unversioned;
