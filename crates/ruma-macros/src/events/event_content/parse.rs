//! Parsing helpers for the `EventContent` derive macro.

use as_variant::as_variant;
use proc_macro2::Span;
use syn::{
    meta::ParseNestedMeta,
    parse::{Parse, ParseStream},
    parse_quote,
};

use super::{EventContent, EventContentField, EventContentKind};
use crate::{
    events::common::{CommonEventKind, EventType, EventTypes},
    util::{ParseNestedMetaExt, RumaEvents, SerdeMetaItem, StructFieldExt},
};

impl EventContent {
    pub(super) fn parse(input: syn::DeriveInput) -> syn::Result<Self> {
        let ruma_events = RumaEvents::new();

        let mut event_content_attrs = EventContentAttrs::default();

        for attr in &input.attrs {
            if !attr.path().is_ident("ruma_event") {
                continue;
            }

            attr.parse_nested_meta(|meta| event_content_attrs.try_merge(meta, attr))?;
        }

        let EventContentAttrs {
            event_type,
            aliases,
            kind,
            state_key_type,
            unsigned_type,
            has_custom_redacted,
            has_custom_possibly_redacted,
            has_without_relation,
        } = event_content_attrs;

        let event_type = event_type.ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "missing `type` attribute, \
                 add `#[ruma_event(type = \"m.event_type\", kind = Kind)]` \
                 below the event content derive",
            )
        })?;
        let types = EventTypes::try_from_parts(event_type, aliases)?;

        let fields =
            as_variant!(input.data, syn::Data::Struct(syn::DataStruct { fields, ..}) => fields)
                .map(|fields| {
                    fields.into_iter().map(|field| field.try_into()).collect::<syn::Result<_>>()
                })
                .transpose()?;

        let kind = EventContentKind::try_from_parts(
            kind,
            state_key_type,
            unsigned_type,
            has_custom_redacted,
            has_custom_possibly_redacted,
            &ruma_events,
        )?;

        let event_content = Self {
            types,
            ident: input.ident,
            vis: input.vis,
            fields,
            kind,
            has_without_relation,
            ruma_events,
        };

        event_content.validate()?;

        Ok(event_content)
    }
}

impl EventContent {
    /// Validate the data inside this
    fn validate(&self) -> syn::Result<()> {
        // Ident check.
        if !self.ident.to_string().ends_with("Content") {
            return Err(syn::Error::new_spanned(
                &self.ident,
                "event content struct name must end with `Content`",
            ));
        }

        // Type suffix checks.
        let has_type_suffix = self.types.is_prefix();

        if has_type_suffix && !self.kind.is_account_data() {
            return Err(syn::Error::new_spanned(
                &self.types.ev_type,
                "only account data events may contain a `.*` suffix",
            ));
        }

        if let Some(fields) = &self.fields {
            let type_fragment_fields_count =
                fields.iter().filter(|field| field.is_type_fragment).count();

            if has_type_suffix && type_fragment_fields_count == 0 {
                return Err(syn::Error::new_spanned(
                    &self.types.ev_type,
                    "event type with a `.*` suffix requires there to be a \
                     `#[ruma_event(type_fragment)]` field",
                ));
            }

            if !has_type_suffix && type_fragment_fields_count > 0 {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "`#[ruma_event(type_fragment)]` field is only valid for an event type with a `.*` suffix",
                ));
            }

            if type_fragment_fields_count > 1 {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "There can only be one `#[ruma_event(type_fragment)]` field",
                ));
            }
        } else if has_type_suffix {
            return Err(syn::Error::new_spanned(
                &self.types.ev_type,
                "event type with a `.*` suffix is required to be a struct",
            ));
        }

        // Checks for generated structs.
        if self.kind.should_generate_redacted() && self.fields.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                "To generate a redacted event content, \
                 the event content type needs to be a struct. \
                 Disable this with the `custom_redacted` attribute",
            ));
        }

        if self.kind.should_generate_possibly_redacted() {
            if self
                .fields
                .as_ref()
                .is_none_or(|fields| fields.iter().any(|field| field.inner.ident.is_none()))
            {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "To generate a possibly redacted event content, \
                     the event content type needs to be a struct with named fields. \
                     Disable this with the `custom_possibly_redacted` attribute",
                ));
            }

            if let Some(fields) = &self.fields
                && let Some(field_with_unsupported_serde_attribute) = fields.iter().find(|field| {
                    if field.keep_in_possibly_redacted() {
                        return false;
                    }

                    field.inner.serde_meta_items().any(|serde_meta| {
                        serde_meta != SerdeMetaItem::Rename && serde_meta != SerdeMetaItem::Alias
                    })
                })
            {
                return Err(syn::Error::new_spanned(
                    field_with_unsupported_serde_attribute,
                    "To generate a possibly redacted event content, \
                     the fields that are redacted must either use the `default` \
                     serde attribute with any other attribute, or only the \
                     following serde attributes: `rename` or `alias`. \
                     Disable this with the `custom_possibly_redacted` attribute",
                ));
            }
        }

        if self.has_without_relation
            && self.fields.as_ref().is_none_or(|fields| {
                !fields.iter().any(|field| {
                    field.inner.ident.as_ref().is_some_and(|ident| *ident == "relates_to")
                })
            })
        {
            return Err(syn::Error::new(
                Span::call_site(),
                "To generate an event content without relation, \
                 the event content type needs to be a struct with a field named `relates_to`. \
                 Disable this by removing the `without_relation` attribute",
            ));
        }

        Ok(())
    }
}

/// Parsed container attributes for the `EventContent` macro.
#[derive(Default)]
pub struct EventContentAttrs {
    /// The main event type.
    event_type: Option<EventType>,

    /// The alternative event types.
    aliases: Vec<EventType>,

    /// The event content kind.
    kind: Option<EventContentKindAttr>,

    /// The type of the state key.
    state_key_type: Option<syn::Type>,

    /// The type of the unsigned data.
    unsigned_type: Option<syn::Type>,

    /// Whether the `Redacted*EventContent` type is implemented manually rather than generated by
    /// this macro.
    has_custom_redacted: bool,

    /// Whether the `PossiblyRedacted*EventContent` type is implemented manually rather than
    /// generated by this macro.
    has_custom_possibly_redacted: bool,

    /// Whether this macro should generate an `*EventContentWithoutRelation` type.
    has_without_relation: bool,
}

impl EventContentAttrs {
    /// Set the main event type.
    ///
    /// Returns an error if it is already set.
    fn set_event_type(&mut self, event_type: EventType, attr: &syn::Attribute) -> syn::Result<()> {
        if self.event_type.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple values for `type` attribute",
            ));
        }

        self.event_type = Some(event_type);
        Ok(())
    }

    /// Add an alternative event type.
    fn push_alias(&mut self, event_type: EventType) {
        self.aliases.push(event_type);
    }

    /// Set the event content kind.
    ///
    /// Returns an error if it is already set.
    fn set_kind(&mut self, kind: EventContentKindAttr, attr: &syn::Attribute) -> syn::Result<()> {
        if self.kind.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple values for `kind` attribute",
            ));
        }

        self.kind = Some(kind);
        Ok(())
    }

    /// Set the type of the state key.
    ///
    /// Returns an error if it is already set.
    fn set_state_key_type(
        &mut self,
        state_key_type: syn::Type,
        attr: &syn::Attribute,
    ) -> syn::Result<()> {
        if self.state_key_type.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple values for `state_key_type` attribute",
            ));
        }

        self.state_key_type = Some(state_key_type);
        Ok(())
    }

    /// Set the type of the unsigned data.
    ///
    /// Returns an error if it is already set.
    fn set_unsigned_type(
        &mut self,
        unsigned_type: syn::Type,
        attr: &syn::Attribute,
    ) -> syn::Result<()> {
        if self.unsigned_type.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple values for `unsigned_type` attribute",
            ));
        }

        self.unsigned_type = Some(unsigned_type);
        Ok(())
    }

    /// Set that the `Redacted*EventContent` type is implemented manually rather than generated by
    /// this macro.
    ///
    /// Returns an error if it is already set.
    fn set_custom_redacted(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if self.has_custom_redacted {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple occurrences of `custom_redacted` attribute",
            ));
        }

        self.has_custom_redacted = true;
        Ok(())
    }

    /// Set that the `PossiblyRedacted*EventContent` type is implemented manually rather than
    /// generated by this macro.
    ///
    /// Returns an error if it is already set.
    fn set_custom_possibly_redacted(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if self.has_custom_possibly_redacted {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple occurrences of `custom_possibly_redacted` attribute",
            ));
        }

        self.has_custom_possibly_redacted = true;
        Ok(())
    }

    /// Set that this macro should generate an `*EventContentWithoutRelation` type.
    ///
    /// Returns an error if it is already set.
    fn set_without_relation(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if self.has_without_relation {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple occurrences of `without_relation` attribute",
            ));
        }

        self.has_without_relation = true;
        Ok(())
    }

    fn try_merge(&mut self, meta: ParseNestedMeta<'_>, attr: &syn::Attribute) -> syn::Result<()> {
        if meta.path.is_ident("type") {
            return self.set_event_type(meta.value()?.parse()?, attr);
        }

        if meta.path.is_ident("alias") {
            self.push_alias(meta.value()?.parse()?);
            return Ok(());
        }

        if meta.path.is_ident("kind") {
            return self.set_kind(meta.value()?.parse()?, attr);
        }

        if meta.path.is_ident("state_key_type") {
            return self.set_state_key_type(meta.value()?.parse()?, attr);
        }

        if meta.path.is_ident("unsigned_type") {
            return self.set_unsigned_type(meta.value()?.parse()?, attr);
        }

        if meta.path.is_ident("custom_redacted") {
            if meta.has_value() {
                return Err(meta.error("`custom_redacted` attribute doesn't expect a value"));
            }

            return self.set_custom_redacted(attr);
        }

        if meta.path.is_ident("custom_possibly_redacted") {
            if meta.has_value() {
                return Err(
                    meta.error("`custom_possibly_redacted` attribute doesn't expect a value")
                );
            }

            return self.set_custom_possibly_redacted(attr);
        }

        if meta.path.is_ident("without_relation") {
            if meta.has_value() {
                return Err(meta.error("`without_relation` attribute doesn't expect a value"));
            }

            return self.set_without_relation(attr);
        }

        Err(meta.error("unsupported `ruma_event` attribute"))
    }
}

impl TryFrom<syn::Field> for EventContentField {
    type Error = syn::Error;

    fn try_from(mut inner: syn::Field) -> Result<Self, Self::Error> {
        let mut field_attrs = EventContentFieldAttrs::default();

        for attr in inner.attrs.extract_if(.., |attr| attr.path().is_ident("ruma_event")) {
            attr.parse_nested_meta(|meta| field_attrs.try_merge(meta, &attr))?;
        }

        let EventContentFieldAttrs { skip_redaction, is_type_fragment } = field_attrs;

        if skip_redaction && is_type_fragment {
            return Err(syn::Error::new_spanned(
                inner,
                "the `skip_redaction` attribute is not valid on a field with the `type_fragment` attribute",
            ));
        }

        Ok(Self { inner, skip_redaction, is_type_fragment })
    }
}

/// Parsed field attributes for the `EventContent` macro.
#[derive(Default)]
struct EventContentFieldAttrs {
    /// Whether this field should be kept during redaction.
    skip_redaction: bool,

    /// Whether this field represents the suffix of the event type.
    is_type_fragment: bool,
}

impl EventContentFieldAttrs {
    /// Set that this field should be kept during redaction.
    ///
    /// Returns an error if it is already set.
    fn set_skip_redaction(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if self.skip_redaction {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple occurrences of `skip_redaction` attribute",
            ));
        }

        self.skip_redaction = true;
        Ok(())
    }

    /// Set that this field represents the suffix of the event type.
    ///
    /// Returns an error if it is already set.
    fn set_type_fragment(&mut self, attr: &syn::Attribute) -> syn::Result<()> {
        if self.is_type_fragment {
            return Err(syn::Error::new_spanned(
                attr,
                "cannot have multiple occurrences of `type_fragment` attribute",
            ));
        }

        self.is_type_fragment = true;
        Ok(())
    }

    fn try_merge(&mut self, meta: ParseNestedMeta<'_>, attr: &syn::Attribute) -> syn::Result<()> {
        if meta.path.is_ident("skip_redaction") {
            if !meta.input.is_empty() {
                return Err(meta.error("`skip_redaction` attribute doesn't expect a value"));
            }

            return self.set_skip_redaction(attr);
        }

        if meta.path.is_ident("type_fragment") {
            if !meta.input.is_empty() {
                return Err(meta.error("`type_fragment` attribute doesn't expect a value"));
            }

            return self.set_type_fragment(attr);
        }

        Err(meta.error("unsupported `ruma_event` attribute"))
    }
}

impl EventContentKind {
    fn try_from_parts(
        kind: Option<EventContentKindAttr>,
        state_key_type: Option<syn::Type>,
        unsigned_type: Option<syn::Type>,
        has_custom_redacted: bool,
        has_custom_possibly_redacted: bool,
        ruma_events: &RumaEvents,
    ) -> syn::Result<Self> {
        let kind = kind.ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "missing `kind` attribute, \
                 add `#[ruma_event(type = \"m.event_type\", kind = Kind)]` \
                 below the event content derive",
            )
        })?;

        let is_state = matches!(kind, EventContentKindAttr::Single(CommonEventKind::State));
        let is_message_like =
            matches!(kind, EventContentKindAttr::Single(CommonEventKind::MessageLike));

        if let Some(state_key_type) = &state_key_type
            && !is_state
        {
            return Err(syn::Error::new_spanned(
                state_key_type,
                "the `state_key_type` attribute is only valid for the State kind",
            ));
        }

        if let Some(unsigned_type) = &unsigned_type
            && !is_state
        {
            return Err(syn::Error::new_spanned(
                unsigned_type,
                "the `unsigned_type` attribute is only valid for the State kind",
            ));
        }

        if has_custom_redacted && !is_state && !is_message_like {
            return Err(syn::Error::new(
                Span::call_site(),
                "the `custom_redacted` attribute is only valid for the State and MessageLike kinds",
            ));
        }

        if has_custom_possibly_redacted && !is_state {
            return Err(syn::Error::new(
                Span::call_site(),
                "the `custom_possibly_redacted` attribute is only valid for the State kind",
            ));
        }

        Ok(match kind {
            EventContentKindAttr::Single(kind) => match kind {
                CommonEventKind::GlobalAccountData => EventContentKind::GlobalAccountData,
                CommonEventKind::RoomAccountData => EventContentKind::RoomAccountData,
                CommonEventKind::EphemeralRoom => EventContentKind::EphemeralRoom,
                CommonEventKind::MessageLike => {
                    EventContentKind::MessageLike { has_custom_redacted }
                }
                CommonEventKind::State => {
                    let state_key_type = state_key_type.ok_or_else(|| {
                        syn::Error::new(Span::call_site(), "missing `state_key_type` attribute")
                    })?;
                    let unsigned_type = unsigned_type.unwrap_or_else(|| {
                        parse_quote! {
                            #ruma_events::StateUnsigned<Self::PossiblyRedacted>
                        }
                    });

                    EventContentKind::State {
                        state_key_type,
                        unsigned_type,
                        has_custom_redacted,
                        has_custom_possibly_redacted,
                    }
                }
                CommonEventKind::ToDevice => EventContentKind::ToDevice,
            },
            EventContentKindAttr::BothAccountData => EventContentKind::BothAccountData,
        })
    }
}

/// The possible values of the `kind` attribute of an event content.
///
/// This is a wrapper around [`EventKind`] that allows to provide two kinds for the same event
/// content.
#[derive(Clone, Copy)]
enum EventContentKindAttr {
    /// The event content has a single kind.
    Single(CommonEventKind),

    /// The event content is of the two account data kinds.
    BothAccountData,
}

impl Parse for EventContentKindAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let first_event_kind: CommonEventKind = input.parse()?;

        let second_event_kind = input
            .peek(syn::Token![+])
            .then(|| {
                let _: syn::Token![+] = input.parse()?;
                input.parse::<CommonEventKind>()
            })
            .transpose()?;

        match (first_event_kind, second_event_kind) {
            (event_kind, None) => Ok(Self::Single(event_kind)),
            (CommonEventKind::GlobalAccountData, Some(CommonEventKind::RoomAccountData))
            | (CommonEventKind::RoomAccountData, Some(CommonEventKind::GlobalAccountData)) => {
                Ok(Self::BothAccountData)
            }
            _ => Err(syn::Error::new(Span::call_site(), "only account data can have two kinds")),
        }
    }
}
