//! Details of parsing input for the `ruma_event` procedural macro.

use syn::{
    braced,
    parse::{self, Parse, ParseStream},
    token::Colon,
    Attribute, Expr, ExprLit, Field, FieldValue, Ident, Lit, LitStr, Member, Token, TypePath,
};

/// The entire `ruma_event!` macro structure directly as it appears in the source code..
pub struct RumaEventInput {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The name of the event.
    pub name: Ident,

    /// The kind of event, determined by the `kind` field.
    pub kind: EventKind,

    /// The value for the `type` field in the JSON representation of this event. There needs to be a
    /// corresponding variant in `ruma_events::EventType` for this event (converted to a valid
    /// Rust-style type name by stripping `m.`, replacing the remaining dots by underscores and then
    /// converting from snake_case to CamelCase).
    pub event_type: LitStr,

    /// Additional named struct fields in the top level event struct.
    pub fields: Option<Vec<Field>>,

    /// A struct definition or type alias to be used as the event's `content` field.
    pub content: Content,
}

impl Parse for RumaEventInput {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;
        let body;
        braced!(body in input);

        let mut kind = None;
        let mut event_type = None;
        let mut fields = None;
        let mut content = None;

        for field_value_inline_struct in
            body.parse_terminated::<RumaEventField, Token![,]>(RumaEventField::parse)?
        {
            match field_value_inline_struct {
                RumaEventField::Block(field_block) => {
                    let ident = match field_block.member {
                        Member::Named(ident) => ident,
                        Member::Unnamed(_) => panic!("fields with block values in `ruma_event!` must named `content_type_alias`"),
                    };

                    if ident == "content_type_alias" {
                        content = Some(Content::Typedef(field_block.typedef));
                    }
                }
                RumaEventField::InlineStruct(field_inline_struct) => {
                    let ident = match field_inline_struct.member {
                        Member::Named(ident) => ident,
                        Member::Unnamed(_) => panic!("fields with inline struct values in `ruma_event!` must be named `fields` or `content`."),
                    };

                    if ident == "fields" {
                        fields = Some(field_inline_struct.fields);
                    } else if ident == "content" {
                        content = Some(Content::Struct(field_inline_struct.fields));
                    }
                }
                RumaEventField::Value(field_value) => {
                    let ident = match field_value.member {
                        Member::Named(ident) => ident,
                        Member::Unnamed(_) => panic!("fields with expression values in `ruma_event!` must be named `kind` or `event_type`, ."),
                    };

                    if ident == "kind" {
                        let event_kind = match field_value.expr {
                            Expr::Path(expr_path) => {
                                if expr_path.path.is_ident("Event") {
                                    EventKind::Event
                                } else if expr_path.path.is_ident("RoomEvent") {
                                    EventKind::RoomEvent
                                } else if expr_path.path.is_ident("StateEvent") {
                                    EventKind::StateEvent
                                } else {
                                    panic!("value of field `kind` must be one of `Event`, `RoomEvent`, or `StateEvent`");
                                }
                            }
                            _ => panic!(
                                "value of field `kind` is required to be an ident by `ruma_event!`"
                            ),
                        };

                        kind = Some(event_kind);
                    } else if ident == "event_type" {
                        event_type = Some(match field_value.expr {
                            Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s,
                            // TODO: Span info
                            _ => panic!(
                                "value of field `event_type` is required to be a string literal by `ruma_event!`"
                            ),
                        })
                    } else {
                        panic!("unexpected field-value pair with field name `{}`", ident);
                    }
                }
            }
        }

        if kind.is_none() {
            panic!("field `kind` is required by `ruma_event!`");
        } else if event_type.is_none() {
            panic!("field `event_type` is required by `ruma_event!`");
        } else if content.is_none() {
            panic!(
                "one field named `content` or `content_type_alias` is required by `ruma_event!`"
            );
        }

        Ok(Self {
            attrs,
            name,
            kind: kind.unwrap(),
            event_type: event_type.unwrap(),
            fields,
            content: content.unwrap(),
        })
    }
}

/// Which kind of event is being generated.
///
/// Determined by the `kind` field in the macro body.
#[derive(PartialEq)]
pub enum EventKind {
    /// A basic event.
    Event,

    /// A room event.
    RoomEvent,

    /// A state event.
    StateEvent,
}

/// Information for generating the type used for the event's `content` field.
pub enum Content {
    /// A struct, e.g. `ExampleEventContent { ... }`.
    Struct(Vec<Field>),

    /// A type alias, e.g. `type ExampleEventContent = SomeExistingType`
    Typedef(Typedef),
}

/// The style of field within the macro body.
#[allow(clippy::large_enum_variant)]
enum RumaEventField {
    /// The value of a field is a block with a type alias in it.
    ///
    /// Used for `content_type_alias`.
    Block(FieldBlock),

    /// The value of a field is a block with named struct fields in it.
    ///
    /// Used for `content`.
    InlineStruct(FieldInlineStruct),

    /// A standard named struct field.
    ///
    /// Used for `kind` and `event_type`.
    Value(FieldValue),
}

impl Parse for RumaEventField {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let ahead = input.fork();
        let field_ident: Ident = ahead.parse()?;

        match field_ident.to_string().as_ref() {
            "content" | "fields" => {
                let attrs = input.call(Attribute::parse_outer)?;
                let member = input.parse()?;
                let colon_token = input.parse()?;
                let body;
                braced!(body in input);
                let fields = body
                    .parse_terminated::<Field, Token![,]>(Field::parse_named)?
                    .into_iter()
                    .collect();

                Ok(RumaEventField::InlineStruct(FieldInlineStruct {
                    attrs,
                    member,
                    colon_token,
                    fields,
                }))
            }
            "content_type_alias" => Ok(RumaEventField::Block(FieldBlock {
                attrs: input.call(Attribute::parse_outer)?,
                member: input.parse()?,
                colon_token: input.parse()?,
                typedef: input.parse()?,
            })),
            _ => Ok(RumaEventField::Value(input.parse()?)),
        }
    }
}

/// The value of a field is a block with a type alias in it.
///
/// Used for `content_type_alias`.
struct FieldBlock {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The name of the field.
    pub member: Member,

    /// The colon that appears between the field name and type.
    pub colon_token: Colon,

    /// The path to the type that will be used in a type alias for the event's `content` type.
    pub typedef: Typedef,
}

/// The value of a field is a block with named struct fields in it.
///
/// Used for `content`.
struct FieldInlineStruct {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// The name of the field.
    pub member: Member,

    /// The colon that appears between the field name and type.
    pub colon_token: Colon,

    /// The fields that define the `content` struct.
    pub fields: Vec<Field>,
}

/// Path to a type to be used in a type alias for an event's `content` type.
pub struct Typedef {
    /// Outer attributes on the field, such as a docstring.
    pub attrs: Vec<Attribute>,

    /// Path to the type.
    pub path: TypePath,
}

impl Parse for Typedef {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let body;
        braced!(body in input);

        Ok(Self {
            attrs: body.call(Attribute::parse_outer)?,
            path: body.parse()?,
        })
    }
}
