#![allow(unexpected_cfgs)]

mod not_transparent {
    #[derive(ruma_macros::IdDst)]
    pub struct NotTransparentId(str);
}

mod inactive_cfg_transparent {
    #[derive(ruma_macros::IdDst)]
    #[cfg_attr(any(), repr(transparent))]
    pub struct InactiveCfgTransparentId(str);
}

mod malformed_repr {
    #[derive(ruma_macros::IdDst)]
    #[repr(transparent, align = 8)]
    pub struct MalformedReprId(str);
}

mod string_field {
    #[derive(ruma_macros::IdDst)]
    #[repr(transparent)]
    pub struct StringId(String);
}

fn main() {}
