//! Types for the [`m.space_order`] event.
//!
//! [`m.space_order`]: https://github.com/matrix-org/matrix-spec-proposals/pull/3230

use ruma_common::OwnedSpaceChildOrder;
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an `m.space_order` event.
///
/// Whether the space has been explicitly ordered.
///
/// This event appears in the user's room account data for the space room in
/// question.
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3230.space_order", alias = "m.space_order", kind = RoomAccountData)]
pub struct SpaceOrderEventContent {
    /// The current space order.
    pub order: OwnedSpaceChildOrder,
}

impl SpaceOrderEventContent {
    /// Creates a new `SpaceOrderEventContent` with the given order.
    pub fn new(order: OwnedSpaceChildOrder) -> Self {
        Self { order }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::{SpaceChildOrder, canonical_json::assert_to_canonical_json_eq};
    use serde_json::{from_value as from_json_value, json};

    use super::SpaceOrderEventContent;
    use crate::{AnyRoomAccountDataEvent, RoomAccountDataEvent};

    #[test]
    fn deserialize() {
        let raw_unstable_space_order = json!({
            "type": "org.matrix.msc3230.space_order",
            "content": {
                "order": "a",
            },
        });
        let unstable_space_order_account_data =
            from_json_value::<AnyRoomAccountDataEvent>(raw_unstable_space_order).unwrap();
        assert_matches!(
            unstable_space_order_account_data,
            AnyRoomAccountDataEvent::SpaceOrder(unstable_space_order)
        );
        assert_eq!(unstable_space_order.content.order, SpaceChildOrder::parse("a").unwrap());

        let raw_space_order = json!({
            "type": "m.space_order",
            "content": {
                "order": "b",
            },
        });
        let space_order_account_data =
            from_json_value::<AnyRoomAccountDataEvent>(raw_space_order).unwrap();
        assert_matches!(space_order_account_data, AnyRoomAccountDataEvent::SpaceOrder(space_order));
        assert_eq!(space_order.content.order, SpaceChildOrder::parse("b").unwrap());
    }

    #[test]
    fn serialize() {
        let space_order = SpaceOrderEventContent::new(SpaceChildOrder::parse("a").unwrap());
        let space_order_account_data = RoomAccountDataEvent { content: space_order };
        assert_to_canonical_json_eq!(
            space_order_account_data,
            json!({
                "type": "org.matrix.msc3230.space_order",
                "content": {
                    "order": "a",
                },
            })
        );
    }
}
