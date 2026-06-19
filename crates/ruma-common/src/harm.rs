//! Types of harmful content, as defined by [MSC4456].
//!
//! [MSC4456]: https://github.com/matrix-org/matrix-spec-proposals/pull/4456

use ruma_macros::StringEnum;

use crate::PrivOwnedStr;

/// A type of harmful content.
#[derive(Clone, StringEnum)]
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[non_exhaustive]
#[allow(missing_docs, clippy::enum_variant_names)]
pub enum Harm {
    // m.spam
    #[ruma_enum(rename = "org.matrix.msc4456.spam")]
    Spam,
    #[ruma_enum(rename = "org.matrix.msc4456.spam.fraud")]
    Fraud,
    #[ruma_enum(rename = "org.matrix.msc4456.spam.impersonation")]
    Impersonation,
    #[ruma_enum(rename = "org.matrix.msc4456.spam.election_interference")]
    ElectionInterference,
    #[ruma_enum(rename = "org.matrix.msc4456.spam.flooding")]
    Flooding,

    // m.adult
    #[ruma_enum(rename = "org.matrix.msc4456.adult")]
    Adult,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.sexual_abuse")]
    SexualAbuse,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.ncii")]
    Ncii,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.deepfake")]
    Deepfake,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.animal_sexual_abuse")]
    AnimalSexualAbuse,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.sexual_violence")]
    SexualViolence,

    // m.harassment
    #[ruma_enum(rename = "org.matrix.msc4456.harassment")]
    Harassment,
    #[ruma_enum(rename = "org.matrix.msc4456.harassment.trolling")]
    Trolling,
    #[ruma_enum(rename = "org.matrix.msc4456.harassment.targeted")]
    TargetedHarassment,
    #[ruma_enum(rename = "org.matrix.msc4456.harassment.hate")]
    Hate,
    #[ruma_enum(rename = "org.matrix.msc4456.harassment.doxxing")]
    Doxxing,

    // m.violence
    #[ruma_enum(rename = "org.matrix.msc4456.violence")]
    Violence,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.animal_welfare")]
    AnimalWelfare,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.threats")]
    Threats,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.graphic")]
    Graphic,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.glorification")]
    GlorificationOfViolence,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.extremism")]
    Extremism,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.human_trafficking")]
    HumanTrafficking,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.domestic")]
    DomesticViolence,

    // m.child_safety
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety")]
    ChildSafety,
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety.csam")]
    Csam,
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety.grooming")]
    Grooming,
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety.privacy_violation")]
    ChildPrivacyViolation,
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety.harassment")]
    ChildHarassment,

    // m.danger
    #[ruma_enum(rename = "org.matrix.msc4456.danger")]
    Danger,
    #[ruma_enum(rename = "org.matrix.msc4456.danger.self_harm")]
    SelfHarm,
    #[ruma_enum(rename = "org.matrix.msc4456.danger.eating_disorder")]
    EatingDisorder,
    #[ruma_enum(rename = "org.matrix.msc4456.danger.challenges")]
    Challenges,
    #[ruma_enum(rename = "org.matrix.msc4456.danger.substance_abuse")]
    SubstanceAbuse,

    // m.tos
    #[ruma_enum(rename = "org.matrix.msc4456.tos")]
    TermsOfService,
    #[ruma_enum(rename = "org.matrix.msc4456.tos.hacking")]
    Hacking,
    #[ruma_enum(rename = "org.matrix.msc4456.tos.prohibited")]
    ProhibitedItems,
    #[ruma_enum(rename = "org.matrix.msc4456.tos.ban_evasion")]
    BanEvasion,

    // m.other
    #[ruma_enum(rename = "org.matrix.msc4456.other")]
    Other,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
