//! Implementation details of parsing proc macro input.

use syn::{
    Attribute,
    AttrStyle,
    Expr,
    Field,
    Ident,
    Meta,
    NestedMeta,
    Visibility,
};
// use syn::parse::{expr, ident, lit, ty};
// use synom::space::{block_comment, whitespace};

#[derive(Debug)]
pub enum Entry {
    Metadata(Vec<(Ident, Expr)>),
    Request(Vec<Field>),
    Response(Vec<Field>),
}

named!(pub parse_entries -> Vec<Entry>, do_parse!(
    entries: many0!(entry) >>
    (entries)
));

named!(entry -> Entry, alt!(
    do_parse!(
        block_type: syn!(Ident) >>
        cond_reduce!(block_type == "metadata") >>
        brace_and_fields: braces!(many0!(struct_init_field)) >>
        (Entry::Metadata(brace_and_fields.1))
    )
    |
    do_parse!(
        block_type: syn!(Ident) >>
        cond_reduce!(block_type == "request") >>
        brace_and_fields: braces!(terminated_list!(punct!(","), struct_field)) >>
        (Entry::Request(brace_and_fields.1))
    )
    |
    do_parse!(
        block_type: syn!(Ident) >>
        cond_reduce!(block_type == "response") >>
        brace_and_fields: braces!(terminated_list!(punct!(","), struct_field)) >>
        (Entry::Response(brace_and_fields.1))
    )
));

// Everything below copy/pasted from syn 0.11.11.

named!(struct_init_field -> (Ident, Expr), do_parse!(
    ident: ident >>
    punct!(:) >>
    expr: expr >>
    punct!(,) >>
    (ident, expr)
));

named!(struct_field -> Field, do_parse!(
    attrs: many0!(outer_attr) >>
    visibility >>
    id: ident >>
    punct!(:) >>
    ty: ty >>
    (Field {
        ident: Some(id),
        vis: Visibility::Public, // Ignore declared visibility, always make fields public
        attrs: attrs,
        ty: ty,
    })
));

named!(outer_attr -> Attribute, alt!(
    do_parse!(
        punct!(#) >>
        brackets_and_meta_item: brackets!(meta_item) >>
        (Attribute {
            style: AttrStyle::Outer,
            value: brackets_and_meta_item.1,
            is_sugared_doc: false,
        })
    )
    |
    do_parse!(
        punct!(/) >>
        punct!(/) >>
        punct!(/) >>
        not!(tag!("/")) >>
        content: take_until!("\n") >>
        (Attribute {
            style: AttrStyle::Outer,
            value: Meta::NameValue(
                "doc".into(),
                format!("///{}", content).into(),
            ),
            is_sugared_doc: true,
        })
    )
    |
    do_parse!(
        option!(whitespace) >>
        peek!(tuple!(tag!("/**"), not!(tag!("*")))) >>
        com: block_comment >>
        (Attribute {
            style: AttrStyle::Outer,
            value: Meta::NameValue(
                "doc".into(),
                com.into(),
            ),
            is_sugared_doc: true,
        })
    )
));

named!(meta_item -> Meta, alt!(
    do_parse!(
        id: ident >>
        parens_and_inner: parens!(terminated_list!(punct!(,), nested_meta_item)) >>
        (Meta::List(id, parens_and_inner.1))
    )
    |
    do_parse!(
        name: ident >>
        punct!(=) >>
        value: lit >>
        (Meta::NameValue(name, value))
    )
    |
    map!(ident, Meta::Word)
));

named!(nested_meta_item -> NestedMeta, alt!(
    meta_item => { NestedMeta::Meta }
    |
    lit => { NestedMeta::Literal }
));

named!(visibility -> Visibility, alt!(
    keyword!(pub) => { |_| Visibility::Public }
    |
    epsilon!() => { |_| Visibility::Inherited }
));
