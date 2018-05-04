use quote::{ToTokens, Tokens};
use syn::synom::Synom;
use syn::{Expr, Ident};

#[derive(Debug)]
pub struct Metadata {
    pub description: Tokens,
    pub method: Tokens,
    pub name: Tokens,
    pub path: Tokens,
    pub rate_limited: Tokens,
    pub requires_authentication: Tokens,
}

impl Synom for Metadata {
    named!(parse -> Self, do_parse!(
        ident: syn!(Ident) >>
        cond_reduce!(ident == "description") >>
        punct!(:) >>
        description: syn!(Expr) >>
        punct!(,) >>

        ident: syn!(Ident) >>
        cond_reduce!(ident == "method") >>
        punct!(:) >>
        method: syn!(Expr) >>
        punct!(,) >>

        ident: syn!(Ident) >>
        cond_reduce!(ident == "name") >>
        punct!(:) >>
        name: syn!(Expr) >>
        punct!(,) >>

        ident: syn!(Ident) >>
        cond_reduce!(ident == "path") >>
        punct!(:) >>
        path: syn!(Expr) >>
        punct!(,) >>

        ident: syn!(Ident) >>
        cond_reduce!(ident == "rate_limited") >>
        punct!(:) >>
        rate_limited: syn!(Expr) >>
        punct!(,) >>

        ident: syn!(Ident) >>
        cond_reduce!(ident == "requires_authentication") >>
        punct!(:) >>
        requires_authentication: syn!(Expr) >>
        punct!(,) >>

        (Metadata {
            description,
            method,
            name,
            path,
            rate_limited,
            requires_authentication,
        })
    ));
}

impl From<Vec<(Ident, Expr)>> for Metadata {
    fn from(fields: Vec<(Ident, Expr)>) -> Self {
        let mut description = None;
        let mut method = None;
        let mut name = None;
        let mut path = None;
        let mut rate_limited = None;
        let mut requires_authentication = None;

        for field in fields {
            let (identifier, expression) = field;

            if identifier == Ident::new("description") {
                description = Some(tokens_for(expression));
            } else if identifier == Ident::new("method") {
                method = Some(tokens_for(expression));
            } else if identifier == Ident::new("name") {
                name = Some(tokens_for(expression));
            } else if identifier == Ident::new("path") {
                path = Some(tokens_for(expression));
            } else if identifier == Ident::new("rate_limited") {
                rate_limited = Some(tokens_for(expression));
            } else if identifier == Ident::new("requires_authentication") {
                requires_authentication = Some(tokens_for(expression));
            } else {
                panic!("ruma_api! metadata included unexpected field: {}", identifier);
            }
        }

        Metadata {
            description: description.expect("ruma_api! metadata is missing description"),
            method: method.expect("ruma_api! metadata is missing method"),
            name: name.expect("ruma_api! metadata is missing name"),
            path: path.expect("ruma_api! metadata is missing path"),
            rate_limited: rate_limited.expect("ruma_api! metadata is missing rate_limited"),
            requires_authentication: requires_authentication
                .expect("ruma_api! metadata is missing requires_authentication"),
        }
    }
}

/// Helper method for turning a value into tokens.
fn tokens_for<T>(value: T) -> Tokens where T: ToTokens {
    let mut tokens = Tokens::new();

    value.to_tokens(&mut tokens);

    tokens
}
