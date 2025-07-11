//! Parsing helpers specific to the `Event` derive macro.

use syn::Ident;

use crate::events::enums::{EventKind, EventKindVariation};

/// Checks that the given `ident` is a supported event struct name and returns the corresponding
/// tuple of `(kind, variation)`.
///
/// Returns `None` if the value of `ident` is not supported.
pub(super) fn parse_event_struct_ident_to_kind_variation(
    ident: &Ident,
) -> Option<(EventKind, EventKindVariation)> {
    let ident_str = ident.to_string();
    match ident_str.as_str() {
        "GlobalAccountDataEvent" => Some((EventKind::GlobalAccountData, EventKindVariation::None)),
        "RoomAccountDataEvent" => Some((EventKind::RoomAccountData, EventKindVariation::None)),
        "EphemeralRoomEvent" => Some((EventKind::EphemeralRoom, EventKindVariation::None)),
        "SyncEphemeralRoomEvent" => Some((EventKind::EphemeralRoom, EventKindVariation::Sync)),
        "OriginalMessageLikeEvent" => Some((EventKind::MessageLike, EventKindVariation::Original)),
        "OriginalSyncMessageLikeEvent" => {
            Some((EventKind::MessageLike, EventKindVariation::OriginalSync))
        }
        "RedactedMessageLikeEvent" => Some((EventKind::MessageLike, EventKindVariation::Redacted)),
        "RedactedSyncMessageLikeEvent" => {
            Some((EventKind::MessageLike, EventKindVariation::RedactedSync))
        }
        "OriginalStateEvent" => Some((EventKind::State, EventKindVariation::Original)),
        "OriginalSyncStateEvent" => Some((EventKind::State, EventKindVariation::OriginalSync)),
        "StrippedStateEvent" => Some((EventKind::State, EventKindVariation::Stripped)),
        "InitialStateEvent" => Some((EventKind::State, EventKindVariation::Initial)),
        "RedactedStateEvent" => Some((EventKind::State, EventKindVariation::Redacted)),
        "RedactedSyncStateEvent" => Some((EventKind::State, EventKindVariation::RedactedSync)),
        "ToDeviceEvent" => Some((EventKind::ToDevice, EventKindVariation::None)),
        "HierarchySpaceChildEvent" => {
            Some((EventKind::HierarchySpaceChild, EventKindVariation::Stripped))
        }
        "OriginalRoomRedactionEvent" => Some((EventKind::RoomRedaction, EventKindVariation::None)),
        "OriginalSyncRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventKindVariation::OriginalSync))
        }
        "RedactedRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventKindVariation::Redacted))
        }
        "RedactedSyncRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventKindVariation::RedactedSync))
        }
        "DecryptedOlmV1Event" | "DecryptedMegolmV1Event" => {
            Some((EventKind::Decrypted, EventKindVariation::None))
        }
        _ => None,
    }
}
