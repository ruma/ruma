//! Type for the matrixRTC notify event ([MSC4075]).
//!
//! [MSC4075]: https://github.com/matrix-org/matrix-spec-proposals/pull/4075

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::member::Application;
use crate::Mentions;

/// The content of an `m.call.notify` event.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "m.call.notify", kind = MessageLike)]
pub struct CallNotifyEventContent {
    /// A unique identifier for the call.
    pub call_id: String,

    /// The application this notify event applies to.
    pub application: ApplicationType,

    /// How this notify event should notify the receiver.
    pub notify_type: NotifyType,

    /// The users that are notified by this event (See [MSC3952] (Intentional Mentions)).
    ///
    /// [MSC3952]: https://github.com/matrix-org/matrix-spec-proposals/pull/3952
    #[serde(rename = "m.mentions")]
    pub mentions: Mentions,
}

impl CallNotifyEventContent {
    /// Creates a new `CallNotifyEventContent` with the given configuration.
    pub fn new(
        call_id: String,
        application: ApplicationType,
        notify_type: NotifyType,
        mentions: Mentions,
    ) -> Self {
        Self { call_id, application, notify_type, mentions }
    }
}

/// How this notify event should notify the receiver.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum NotifyType {
    /// The receiving client should ring with an audible sound.
    #[serde(rename = "ring")]
    Ring,

    /// The receiving client should display a visual notification.
    #[serde(rename = "notify")]
    Notify,
}

/// The type of matrix RTC application.
///
/// This is different to [`Application`] because application contains all the information from the
/// `m.call.member` event.
///
/// An `Application` can be converted into an `ApplicationType` using `.into()`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum ApplicationType {
    /// A VoIP call.
    #[serde(rename = "m.call")]
    Call,
}

impl From<Application> for ApplicationType {
    fn from(val: Application) -> Self {
        match val {
            Application::Call(_) => ApplicationType::Call,
        }
    }
}
