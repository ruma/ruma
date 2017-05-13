use syn::{
    Attribute,
    AttrStyle,
    Expr,
    Field,
    Ident,
    Item,
    Lit,
    MetaItem,
    NestedMetaItem,
    StrStyle,
    Token,
    TokenTree,
    Visibility,
};
use syn::parse::{expr, ident, lit, ty};
use synom::space::{block_comment, whitespace};

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
        keyword!("metadata") >>
        punct!("{") >>
        fields: many0!(struct_init_field) >>
        punct!("}") >>
        (Entry::Metadata(fields))
    )
    |
    do_parse!(
        keyword!("request") >>
        punct!("{") >>
        fields: terminated_list!(punct!(","), struct_field) >>
        punct!("}") >>
        (Entry::Request(fields))
    )
    |
    do_parse!(
        keyword!("response") >>
        punct!("{") >>
        fields: terminated_list!(punct!(","), struct_field) >>
        punct!("}") >>
        (Entry::Response(fields))
    )
));

// Everything below copy/pasted from syn 0.11.11.

named!(struct_init_field -> (Ident, Expr), do_parse!(
    ident: ident >>
    punct!(":") >>
    expr: expr >>
    punct!(",") >>
    (ident, expr)
));

named!(pub struct_like_body -> Vec<Field>, do_parse!(
    punct!("{") >>
    fields: terminated_list!(punct!(","), struct_field) >>
    punct!("}") >>
    (fields)
));

named!(struct_field -> Field, do_parse!(
    attrs: many0!(outer_attr) >>
    vis: visibility >>
    id: ident >>
    punct!(":") >>
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
        punct!("#") >>
        punct!("[") >>
        meta_item: meta_item >>
        punct!("]") >>
        (Attribute {
            style: AttrStyle::Outer,
            value: meta_item,
            is_sugared_doc: false,
        })
    )
    |
    do_parse!(
        punct!("///") >>
        not!(tag!("/")) >>
        content: take_until!("\n") >>
        (Attribute {
            style: AttrStyle::Outer,
            value: MetaItem::NameValue(
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
            value: MetaItem::NameValue(
                "doc".into(),
                com.into(),
            ),
            is_sugared_doc: true,
        })
    )
));

named!(meta_item -> MetaItem, alt!(
    do_parse!(
        id: ident >>
        punct!("(") >>
        inner: terminated_list!(punct!(","), nested_meta_item) >>
        punct!(")") >>
        (MetaItem::List(id, inner))
    )
    |
    do_parse!(
        name: ident >>
        punct!("=") >>
        value: lit >>
        (MetaItem::NameValue(name, value))
    )
    |
    map!(ident, MetaItem::Word)
));

named!(nested_meta_item -> NestedMetaItem, alt!(
    meta_item => { NestedMetaItem::MetaItem }
    |
    lit => { NestedMetaItem::Literal }
));

named!(visibility -> Visibility, alt!(
    keyword!("pub") => { |_| Visibility::Public }
    |
    epsilon!() => { |_| Visibility::Inherited }
));
