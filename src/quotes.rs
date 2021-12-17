// In this file you want to generate quotes from the nicely parsed fields
use super::fields::{CornettoField, CornettoKind};
use anyhow::Result;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::Lit;

fn quote_default_value(
    name: &syn::Ident,
    field: &CornettoField,
    cornetto_token_streams: &mut CornettoTokenStreams,
) -> Ident {
    let name = format!(
        "DEFAULT_{}_{}",
        name.to_string().to_uppercase(),
        field.ident.to_string().to_uppercase()
    );
    let ident = Ident::new(&name, Span::mixed_site());
    let ty = field.ty.clone();
    let value = field.value.clone();
    let field_name = &field.ident;
    match &value {
        Lit::Str(_) => {
            cornetto_token_streams
                .constants
                .extend(quote_spanned! {Span::mixed_site()=>
                    const #ident: &str = #value;
                });
            cornetto_token_streams.default_fields.push(quote! {
                #field_name: #ident.to_string(),
            });
        }
        _ => {
            cornetto_token_streams
                .constants
                .extend(quote_spanned! {Span::mixed_site()=>
                    const #ident: #ty = #value;
                });
            cornetto_token_streams.default_fields.push(quote! {
                #field_name: #ident,
            });
        }
    }
    ident
}

fn write_cornetto_test_mutable_param(
    constant_name: Ident,
    field: &CornettoField,
    cornetto_token_streams: &mut CornettoTokenStreams,
) {
    let name = &field.ident;
    let ty = &field.ty;
    match &field.value {
        Lit::Str(_) => {
            cornetto_token_streams
                .fn_implementations
                .push(quote_spanned! {Span::mixed_site()=>
                    #[cfg(not(test))]
                    pub fn #name(&self) -> #ty {
                        #constant_name.to_string()
                    }
                    #[cfg(test)]
                    pub fn #name(&self) -> #ty {
                        self.p_fields.lock().unwrap().#name.to_string()
                    }
                });
        }
        _ => {
            cornetto_token_streams
                .fn_implementations
                .push(quote_spanned! {Span::mixed_site()=>
                    #[cfg(not(test))]
                    pub fn #name(&self) -> #ty {
                        #constant_name
                    }
                    #[cfg(test)]
                    pub fn #name(&self) -> #ty {
                        self.p_fields.lock().unwrap().#name
                    }
                });
        }
    };
    cornetto_token_streams
        .test_mut_fields
        .push(quote!(#name: #ty,));
    cornetto_token_streams
        .test_mut_fields_lock
        .push(quote!(lock.#name = #name;));
}

fn write_cornetto_constant_param(
    constant_name: Ident,
    field: &CornettoField,
    cornetto_token_streams: &mut CornettoTokenStreams,
) {
    let ty = &field.ty;
    let name = &field.ident;
    match &field.value {
        Lit::Str(_) => {
            cornetto_token_streams
                .fn_implementations
                .push(quote_spanned! {Span::mixed_site()=>
                    pub fn #name(&self) -> #ty {
                        #constant_name.to_string()
                    }
                });
        }
        _ => {
            cornetto_token_streams
                .fn_implementations
                .push(quote_spanned! {Span::mixed_site()=>
                    pub fn #name(&self) -> #ty {
                        #constant_name
                    }
                });
        }
    }
}

#[derive(Default)]
struct CornettoTokenStreams {
    pub constants: TokenStream,
    pub test_mut_fields_lock: Vec<TokenStream>, // filled with code to dump in _reset argument
    pub test_mut_fields: Vec<TokenStream>,
    pub default_fields: Vec<TokenStream>,
    pub fn_implementations: Vec<TokenStream>, // functions implemented in Cornetto generated structure
}

pub fn write(name: &Ident, fields: &[CornettoField]) -> Result<TokenStream> {
    let mut cornetto_token_streams = CornettoTokenStreams::default();
    for field in fields {
        let constant_name = quote_default_value(name, field, &mut cornetto_token_streams);
        match &field.kind {
            CornettoKind::Testmutable => {
                write_cornetto_test_mutable_param(constant_name, field, &mut cornetto_token_streams)
            }
            CornettoKind::Constant => {
                write_cornetto_constant_param(constant_name, field, &mut cornetto_token_streams)
            }
        }
    }
    let default_fields = cornetto_token_streams.default_fields;
    let default_impl = quote! {
        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(#default_fields)*
                }
            }
        }
    };
    let test_mut_fields = cornetto_token_streams.test_mut_fields;
    let test_mut_fields_lock = cornetto_token_streams.test_mut_fields_lock;
    let constants = cornetto_token_streams.constants;
    let fn_implementations = cornetto_token_streams.fn_implementations;
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
            #(#fn_implementations)*
            #reset_fn
        }
        lazy_static::lazy_static! {
            pub static ref #cornetto_static: #cornetto = #cornetto::default();
        }
    };
    Ok(cornetto_impl)
}
