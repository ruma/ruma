use super::event_parse::{EventKind, EventKindVariation};

pub(crate) fn is_non_stripped_room_event(kind: EventKind, var: EventKindVariation) -> bool {
    matches!(kind, EventKind::MessageLike | EventKind::State)
        && matches!(
            var,
            EventKindVariation::Original
                | EventKindVariation::OriginalSync
                | EventKindVariation::Redacted
                | EventKindVariation::RedactedSync
        )
}
