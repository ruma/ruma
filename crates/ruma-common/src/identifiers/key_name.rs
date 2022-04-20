/// A Matrix key identifier.
///
/// Key identifiers in Matrix are opaque character sequences of `[a-zA-Z_]`. This type is
/// provided simply for its semantic value.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyName(str);

owned_identifier!(OwnedKeyName, KeyName);

opaque_identifier!(KeyName, OwnedKeyName);
