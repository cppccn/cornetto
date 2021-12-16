#![feature(proc_macro_quote)]
mod fields;
use anyhow::{bail, Result};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};

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
        syn::Data::Struct(ds) => impl_quotes_cornetto(&input.ident, ds),
        _ => bail!("`#![derive(Cornetto)]` cannot be applied to other than structs"),
    }
}

fn quote_default_value(
    name: &syn::Ident,
    field: &fields::CornettoField,
    constants: &mut TokenStream,
) -> Ident {
    let name = format!(
        "DEFAULT_{}_{}",
        name.to_string().to_uppercase(),
        field.ident.to_string().to_uppercase()
    );
    let ident = Ident::new(&name, Span::mixed_site());
    let ty = field.ty.clone();
    let value = field.value.clone();
    constants.extend(quote_spanned! {Span::mixed_site()=>
        const #ident: #ty = #value;
    });
    ident
}

fn impl_quotes_cornetto(name: &syn::Ident, ds: &syn::DataStruct) -> Result<TokenStream> {
    // Build CornettoField for each field by parsing attributes
    let fields = fields::CornettoField::parse(ds)?;
    // Start building Output TokenTree
    let mut constants = TokenStream::new();
    let mut test_mut_fields_lock = vec![];
    let mut test_mut_fields = vec![];
    let mut default_fields = vec![];
    let mut fn_impl = vec![];
    for field in fields {
        let const_name = quote_default_value(name, &field, &mut constants);
        let name = field.ident;
        let ty = field.ty;
        default_fields.push(quote! {
            #name: #const_name,
        });
        match field.kind {
            fields::CornettoKind::Testmutable => {
                fn_impl.push(quote_spanned! {Span::mixed_site()=>
                    #[cfg(not(test))]
                    pub fn #name(&self) -> #ty {
                        #const_name
                    }
                    #[cfg(test)]
                    pub fn #name(&self) -> u64 {
                        self.p_fields.lock().unwrap().#name
                    }
                });
                test_mut_fields.push(quote!(#name: #ty,));
                test_mut_fields_lock.push(quote!(lock.#name = #name;));
            }
            fields::CornettoKind::Constant => {
                fn_impl.push(quote_spanned! {Span::mixed_site()=>
                    pub fn #name(&self) -> #ty {
                        #const_name
                    }
                });
            }
        }
    }
    let default_impl = quote! {
        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(#default_fields)*
                }
            }
        }
    };
    let reset_fn = quote! {
        #[cfg(test)]
        pub fn _reset(&self, #(#test_mut_fields)*) {
            let mut lock = self.p_fields.lock().unwrap();
            #(#test_mut_fields_lock)*
        }
    };
    let cornetto = Ident::new(&format!("Cornetto{}", name), Span::mixed_site());
    let cornetto_static = Ident::new(&name.to_string().to_uppercase(), Span::mixed_site());
    let cornetto_impl = quote_spanned! {Span::mixed_site()=>
        #constants
        #default_impl
        #[derive(Default)]
        pub struct #cornetto {
            #[cfg(test)]
            p_fields: std::sync::Mutex<#name>
        }
        impl #cornetto {
            #(#fn_impl)*
            #reset_fn
        }
        lazy_static::lazy_static! {
            pub static ref #cornetto_static: #cornetto = #cornetto::default();
        }
    };
    Ok(cornetto_impl)
}
