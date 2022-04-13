use ruma_macros::IdZst;

/// A Matrix key identifier.
///
/// Key identifiers in Matrix are opaque character sequences of `[a-zA-Z_]`. This type is
/// provided simply for its semantic value.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
pub struct KeyName(str);
