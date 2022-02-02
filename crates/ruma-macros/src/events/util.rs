use super::event_parse::{EventKind, EventKindVariation};

pub(crate) fn is_non_stripped_room_event(kind: EventKind, var: EventKindVariation) -> bool {
    matches!(kind, EventKind::MessageLike | EventKind::State)
        && matches!(
            var,
            EventKindVariation::Full
                | EventKindVariation::Sync
                | EventKindVariation::Redacted
                | EventKindVariation::RedactedSync
        )
}

pub(crate) fn has_prev_content(kind: EventKind, var: EventKindVariation) -> bool {
    matches!(kind, EventKind::State)
        && matches!(var, EventKindVariation::Full | EventKindVariation::Sync)
}

pub(crate) type EventKindFn = fn(EventKind, EventKindVariation) -> bool;

/// This const is used to generate the accessor methods for the `Any*Event` enums.
///
/// DO NOT alter the field names unless the structs in `ruma_common::events::event_kinds` have
/// changed.
pub(crate) const EVENT_FIELDS: &[(&str, EventKindFn)] = &[
    ("origin_server_ts", is_non_stripped_room_event),
    ("room_id", |kind, var| {
        matches!(kind, EventKind::MessageLike | EventKind::State | EventKind::Ephemeral)
            && matches!(var, EventKindVariation::Full | EventKindVariation::Redacted)
    }),
    ("event_id", is_non_stripped_room_event),
    ("sender", |kind, var| {
        matches!(kind, EventKind::MessageLike | EventKind::State | EventKind::ToDevice)
            && var != EventKindVariation::Initial
    }),
    ("state_key", |kind, _| matches!(kind, EventKind::State)),
];
