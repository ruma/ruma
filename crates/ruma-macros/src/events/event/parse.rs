//! Parsing helpers specific to the `Event` derive macro.

use proc_macro2::Span;
use syn::meta::ParseNestedMeta;

use super::{Event, EventField};
use crate::{
    events::enums::{EventKind, EventVariation},
    util::{ParseNestedMetaExt, RumaEvents},
};

impl Event {
    /// Parse the given input as an `Event`.
    pub(super) fn parse(input: syn::DeriveInput) -> syn::Result<Self> {
        let (kind, variation) = event_ident_to_kind_and_variation(&input.ident)?;

        let syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
            ..
        }) = input.data
        else {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "the `Event` derive only supports structs with named fields",
            ));
        };

        let fields = named.into_iter().map(EventField::parse).collect::<Result<_, _>>()?;

        let event = Self {
            ident: input.ident,
            generics: input.generics,
            kind,
            variation,
            fields,
            ruma_events: RumaEvents::new(),
        };

        event.validate()?;

        Ok(event)
    }

    /// Validate the fields of this `Event`.
    fn validate(&self) -> syn::Result<()> {
        let Some(content_field) = self.fields.iter().find(|field| *field.ident() == "content")
        else {
            return Err(syn::Error::new(
                Span::call_site(),
                "`Event` struct must contain at least a `content` field",
            ));
        };

        // We assume that if there are generics, they must be at least for the `content` type, and
        // its generic should be named `C`.
        if !self.generics.params.is_empty() && (!self.generics.params.iter().any(|param| matches!(param, syn::GenericParam::Type(syn::TypeParam { ident, ..}) if *ident == "C")) || !matches!(&content_field.inner.ty, syn::Type::Path(syn::TypePath{ path, ..}) if path.is_ident("C"))) {
            return Err(syn::Error::new(
                Span::call_site(),
                "Generics on `Event` struct are only supported for \
                 the `content` field and must be named `C`",
            ));
        }

        Ok(())
    }
}

/// Get the event kind and variation from the given struct name.
///
/// Returns `None` if the value of `ident` is not supported.
fn event_ident_to_kind_and_variation(
    ident: &syn::Ident,
) -> syn::Result<(EventKind, EventVariation)> {
    Ok(match ident.to_string().as_str() {
        "GlobalAccountDataEvent" => (EventKind::GlobalAccountData, EventVariation::None),
        "RoomAccountDataEvent" => (EventKind::RoomAccountData, EventVariation::None),
        "EphemeralRoomEvent" => (EventKind::EphemeralRoom, EventVariation::None),
        "SyncEphemeralRoomEvent" => (EventKind::EphemeralRoom, EventVariation::Sync),
        "OriginalMessageLikeEvent" => (EventKind::MessageLike, EventVariation::Original),
        "OriginalSyncMessageLikeEvent" => (EventKind::MessageLike, EventVariation::OriginalSync),
        "RedactedMessageLikeEvent" => (EventKind::MessageLike, EventVariation::Redacted),
        "RedactedSyncMessageLikeEvent" => (EventKind::MessageLike, EventVariation::RedactedSync),
        "OriginalStateEvent" => (EventKind::State, EventVariation::Original),
        "OriginalSyncStateEvent" => (EventKind::State, EventVariation::OriginalSync),
        "StrippedStateEvent" => (EventKind::State, EventVariation::Stripped),
        "InitialStateEvent" => (EventKind::State, EventVariation::Initial),
        "RedactedStateEvent" => (EventKind::State, EventVariation::Redacted),
        "RedactedSyncStateEvent" => (EventKind::State, EventVariation::RedactedSync),
        "ToDeviceEvent" => (EventKind::ToDevice, EventVariation::None),
        "HierarchySpaceChildEvent" => (EventKind::HierarchySpaceChild, EventVariation::Stripped),
        "OriginalRoomRedactionEvent" => (EventKind::RoomRedaction, EventVariation::None),
        "OriginalSyncRoomRedactionEvent" => {
            (EventKind::RoomRedaction, EventVariation::OriginalSync)
        }
        "RedactedRoomRedactionEvent" => (EventKind::RoomRedaction, EventVariation::Redacted),
        "RedactedSyncRoomRedactionEvent" => {
            (EventKind::RoomRedaction, EventVariation::RedactedSync)
        }
        "DecryptedOlmV1Event" | "DecryptedMegolmV1Event" => {
            (EventKind::Decrypted, EventVariation::None)
        }
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "not a supported `Event` struct identifier",
            ));
        }
    })
}

impl EventField {
    /// Parse the given field to construct an `EventField`.
    ///
    /// Returns an error if an unknown `ruma_event` attribute is encountered, or if an attribute
    /// that accepts a single value appears several times.
    fn parse(mut inner: syn::Field) -> syn::Result<Self> {
        let mut field_attrs = EventFieldAttrs::default();

        for attr in inner.attrs.extract_if(.., |attribute| attribute.path().is_ident("ruma_event"))
        {
            attr.parse_nested_meta(|meta| field_attrs.try_merge(meta, &attr))?;
        }

        let EventFieldAttrs { default, default_on_error, rename, aliases } = field_attrs;

        Ok(Self { inner, default, default_on_error, rename, aliases })
    }
}

/// A parsed attributes of a field of an [`Event`].
#[derive(Default)]
struct EventFieldAttrs {
    /// Whether this field should deserialize to the default value if it is missing.
    default: bool,

    /// Whether this field should deserialize to the default value if an error occurs during
    /// deserialization.
    default_on_error: bool,

    /// The name to use when (de)serializing this field.
    ///
    /// If this is not set, the name of the field will be used.
    rename: Option<syn::LitStr>,

    /// The alternate names to recognize when deserializing this field.
    aliases: Vec<syn::LitStr>,
}

impl EventFieldAttrs {
    /// Set that this field should deserialize to the default value if it is missing.
    ///
    /// Returns an error if it is already set.
    fn set_default(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if self.default {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple occurrences of `default` attribute",
            ));
        }

        self.default = true;
        Ok(())
    }

    /// Set that this field should deserialize to the default value if an error occurs during
    /// deserialization.
    ///
    /// Returns an error if it is already set.
    fn set_default_on_error(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if self.default_on_error {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple occurrences of `default_on_error` attribute",
            ));
        }

        self.default_on_error = true;
        Ok(())
    }

    /// Set the name to use when (de)serializing this field.
    ///
    /// Returns an error if it is already set.
    fn set_rename(&mut self, rename: syn::LitStr, attr: &syn::Attribute) -> syn::Result<()> {
        if self.rename.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple values for `rename` attribute",
            ));
        }

        self.rename = Some(rename);
        Ok(())
    }

    /// Try to parse the given meta item and merge it into this `EventFieldAttrs`.
    ///
    /// Returns an error if an unknown `ruma_event` attribute is encountered, or if an attribute
    /// that accepts a single value appears several times.
    fn try_merge(&mut self, meta: ParseNestedMeta<'_>, attr: &syn::Attribute) -> syn::Result<()> {
        if meta.path.is_ident("default") {
            if meta.has_value() {
                return Err(meta.error("`default` attribute doesn't expect a value"));
            }

            return self.set_default(attr);
        }

        if meta.path.is_ident("default_on_error") {
            if meta.has_value() {
                return Err(meta.error("`default_on_error` attribute doesn't expect a value"));
            }

            return self.set_default_on_error(attr);
        }

        if meta.path.is_ident("rename") {
            return self.set_rename(meta.value()?.parse()?, attr);
        }

        if meta.path.is_ident("alias") {
            self.aliases.push(meta.value()?.parse()?);
            return Ok(());
        }

        Err(meta.error("unsupported `ruma_event` attribute"))
    }
}
