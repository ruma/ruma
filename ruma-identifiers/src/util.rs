use std::num::NonZeroU8;

#[derive(Clone, Copy, Debug)]
pub struct CommonIdentHeader {
    pub colon_idx: NonZeroU8,
}
