#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! (De)serializable types for the [Matrix Push Gateway API][push-api].
//! These types can be shared by push gateway and server code.
//!
//! [push-api]: https://spec.matrix.org/latest/push-gateway-api/

#![warn(missing_docs)]

pub mod send_event_notification;

ruma_common::priv_owned_str!();
