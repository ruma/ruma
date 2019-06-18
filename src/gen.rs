//! Details of generating code for the `ruma_event` procedural macro.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::parse::RumaEventInput;

/// The result of processing the `ruma_event` macro, ready for output back to source code.
pub struct RumaEvent;

impl From<RumaEventInput> for RumaEvent {
    // TODO: Provide an actual impl for this.
    fn from(_input: RumaEventInput) -> Self {
        Self
    }
}

impl ToTokens for RumaEvent {
    // TODO: Provide an actual impl for this.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let output = quote!(
            pub struct Foo {}
        );

        output.to_tokens(tokens);
    }
}
