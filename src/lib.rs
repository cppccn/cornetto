#![feature(proc_macro_quote)]
mod fields;
mod quotes;
use anyhow::{bail, Result};
use fields::CornettoField;
use proc_macro2::TokenStream;

#[proc_macro_derive(Cornetto, attributes(cornetto))]
pub fn cornetto_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    impl_cornetto(&ast).unwrap().into()
}

fn impl_cornetto(input: &syn::DeriveInput) -> Result<TokenStream> {
    if !input.generics.params.is_empty() {
        bail!("`#![derive(Cornetto)]` cannot be applied to types with generic parameters")
    }
    match &input.data {
        syn::Data::Struct(ds) => quotes::write(&input.ident, &CornettoField::parse(ds)?),
        _ => bail!("`#![derive(Cornetto)]` cannot be applied to other than structs"),
    }
}
