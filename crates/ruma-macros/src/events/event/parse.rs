//! Parsing helpers specific to the `Event` derive macro.

use syn::{Field, Ident};

use crate::events::enums::{EventKind, EventVariation};

/// Checks that the given `ident` is a supported event struct name and returns the corresponding
/// tuple of `(kind, variation)`.
///
/// Returns `None` if the value of `ident` is not supported.
pub(super) fn parse_event_struct_ident_to_kind_variation(
    ident: &Ident,
) -> Option<(EventKind, EventVariation)> {
    let ident_str = ident.to_string();
    match ident_str.as_str() {
        "GlobalAccountDataEvent" => Some((EventKind::GlobalAccountData, EventVariation::None)),
        "RoomAccountDataEvent" => Some((EventKind::RoomAccountData, EventVariation::None)),
        "EphemeralRoomEvent" => Some((EventKind::EphemeralRoom, EventVariation::None)),
        "SyncEphemeralRoomEvent" => Some((EventKind::EphemeralRoom, EventVariation::Sync)),
        "OriginalMessageLikeEvent" => Some((EventKind::MessageLike, EventVariation::Original)),
        "OriginalSyncMessageLikeEvent" => {
            Some((EventKind::MessageLike, EventVariation::OriginalSync))
        }
        "RedactedMessageLikeEvent" => Some((EventKind::MessageLike, EventVariation::Redacted)),
        "RedactedSyncMessageLikeEvent" => {
            Some((EventKind::MessageLike, EventVariation::RedactedSync))
        }
        "OriginalStateEvent" => Some((EventKind::State, EventVariation::Original)),
        "OriginalSyncStateEvent" => Some((EventKind::State, EventVariation::OriginalSync)),
        "StrippedStateEvent" => Some((EventKind::State, EventVariation::Stripped)),
        "InitialStateEvent" => Some((EventKind::State, EventVariation::Initial)),
        "RedactedStateEvent" => Some((EventKind::State, EventVariation::Redacted)),
        "RedactedSyncStateEvent" => Some((EventKind::State, EventVariation::RedactedSync)),
        "ToDeviceEvent" => Some((EventKind::ToDevice, EventVariation::None)),
        "HierarchySpaceChildEvent" => {
            Some((EventKind::HierarchySpaceChild, EventVariation::Stripped))
        }
        "OriginalRoomRedactionEvent" => Some((EventKind::RoomRedaction, EventVariation::None)),
        "OriginalSyncRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventVariation::OriginalSync))
        }
        "RedactedRoomRedactionEvent" => Some((EventKind::RoomRedaction, EventVariation::Redacted)),
        "RedactedSyncRoomRedactionEvent" => {
            Some((EventKind::RoomRedaction, EventVariation::RedactedSync))
        }
        "DecryptedOlmV1Event" | "DecryptedMegolmV1Event" => {
            Some((EventKind::Decrypted, EventVariation::None))
        }
        _ => None,
    }
}

pub(super) trait EventFieldExt {
    /// Whether the given field as the `#[ruma_event(default)]` attribute.
    fn has_default_attr(&self) -> Result<bool, syn::Error>;
}

impl EventFieldExt for Field {
    fn has_default_attr(&self) -> Result<bool, syn::Error> {
        for attr in &self.attrs {
            if !attr.path().is_ident("ruma_event") {
                continue;
            }

            let mut has_default = false;

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("default") {
                    has_default = true;
                    return Ok(());
                }

                Err(meta.error("unsupported attribute, only `default` is supported"))
            })?;

            if has_default {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
