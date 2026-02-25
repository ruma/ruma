//! `m.space.child` order.

use ruma_macros::ruma_id;

/// The order of an [`m.space.child`] event.
///
/// Space child orders in Matrix are opaque character sequences consisting of ASCII characters
/// within the range `\x20` (space) and `\x7E` (~), inclusive. Their length must must not exceed 50
/// characters.
///
/// [`m.space.child`]: https://spec.matrix.org/latest/client-server-api/#mspacechild
#[ruma_id(validate = ruma_identifiers_validation::space_child_order::validate)]
pub struct SpaceChildOrder;

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use ruma_identifiers_validation::Error;

    use super::SpaceChildOrder;

    #[test]
    fn validate_space_child_order() {
        // Valid string.
        SpaceChildOrder::parse("aaa").unwrap();

        // String too long.
        let order = repeat_n('a', 60).collect::<String>();
        assert_eq!(SpaceChildOrder::parse(&order), Err(Error::MaximumLengthExceeded));

        // Invalid character.
        assert_eq!(SpaceChildOrder::parse("🔝"), Err(Error::InvalidCharacters));
    }
}
