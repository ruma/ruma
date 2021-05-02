#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
//! (De)serializable types for the [Matrix Push Gateway API][push-api].
//! These types can be shared by push gateway and server code.
//!
//! [push-api]: https://matrix.org/docs/spec/push_gateway/r0.1.1.html

#![warn(missing_docs)]

pub mod send_event_notification;
