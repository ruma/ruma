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
    SpamFraud,
    #[ruma_enum(rename = "org.matrix.msc4456.spam.impersonation")]
    SpamImpersonation,
    #[ruma_enum(rename = "org.matrix.msc4456.spam.election_interference")]
    SpamElectionInterference,
    #[ruma_enum(rename = "org.matrix.msc4456.spam.flooding")]
    SpamFlooding,

    // m.adult
    #[ruma_enum(rename = "org.matrix.msc4456.adult")]
    Adult,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.sexual_abuse")]
    AdultSexualAbuse,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.ncii")]
    AdultNcii,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.deepfake")]
    AdultDeepfake,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.animal_sexual_abuse")]
    AdultAnimalSexualAbuse,
    #[ruma_enum(rename = "org.matrix.msc4456.adult.sexual_violence")]
    AdultSexualViolence,

    // m.harassment
    #[ruma_enum(rename = "org.matrix.msc4456.harassment")]
    Harassment,
    #[ruma_enum(rename = "org.matrix.msc4456.harassment.trolling")]
    HarassmentTrolling,
    #[ruma_enum(rename = "org.matrix.msc4456.harassment.targeted")]
    HarassmentTargeted,
    #[ruma_enum(rename = "org.matrix.msc4456.harassment.hate")]
    HarassmentHate,
    #[ruma_enum(rename = "org.matrix.msc4456.harassment.doxxing")]
    HarassmentDoxxing,

    // m.violence
    #[ruma_enum(rename = "org.matrix.msc4456.violence")]
    Violence,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.animal_welfare")]
    ViolenceAnimalWelfare,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.threats")]
    ViolenceThreats,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.graphic")]
    ViolenceGraphic,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.glorification")]
    ViolenceGlorification,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.extremism")]
    ViolenceExtremism,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.human_trafficking")]
    ViolenceHumanTrafficking,
    #[ruma_enum(rename = "org.matrix.msc4456.violence.domestic")]
    ViolenceDomestic,

    // m.child_safety
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety")]
    ChildSafety,
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety.csam")]
    ChildSafetyCsam,
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety.grooming")]
    ChildSafetyGrooming,
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety.privacy_violation")]
    ChildSafetyPrivacyViolation,
    #[ruma_enum(rename = "org.matrix.msc4456.child_safety.harassment")]
    ChildSafetyHarassment,

    // m.danger
    #[ruma_enum(rename = "org.matrix.msc4456.danger")]
    Danger,
    #[ruma_enum(rename = "org.matrix.msc4456.danger.self_harm")]
    DangerSelfHarm,
    #[ruma_enum(rename = "org.matrix.msc4456.danger.eating_disorder")]
    DangerEatingDisorder,
    #[ruma_enum(rename = "org.matrix.msc4456.danger.challenges")]
    DangerChallenges,
    #[ruma_enum(rename = "org.matrix.msc4456.danger.substance_abuse")]
    DangerSubstanceAbuse,

    // m.tos
    #[ruma_enum(rename = "org.matrix.msc4456.tos")]
    TermsOfService,
    #[ruma_enum(rename = "org.matrix.msc4456.tos.hacking")]
    TermsOfServiceHacking,
    #[ruma_enum(rename = "org.matrix.msc4456.tos.prohibited")]
    TermsOfServiceProhibitedItems,
    #[ruma_enum(rename = "org.matrix.msc4456.tos.ban_evasion")]
    TermsOfServiceBanEvasion,

    // m.other
    #[ruma_enum(rename = "org.matrix.msc4456.other")]
    Other,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
