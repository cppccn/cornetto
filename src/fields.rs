use anyhow::{bail, Result};
use proc_macro2::Ident;
use syn::{Lit, Type};

pub struct CornettoField {
    pub ident: Ident,
    pub value: Lit,
    pub kind: CornettoKind,
    pub ty: Type,
}

pub enum CornettoKind {
    Constant,
    Testmutable,
}

impl CornettoField {
    // Parse attribute for each fields
    pub fn parse(ds: &syn::DataStruct) -> Result<Vec<CornettoField>> {
        let mut ret = vec![];
        for field in &ds.fields {
            let mut kind = None;
            let mut value = None;
            for attr in &field.attrs {
                let meta_list = match attr.parse_meta() {
                    Ok(syn::Meta::List(list)) => list,
                    _ => bail!("Cannot parse other than a metalist"),
                };
                for nested_meta in meta_list.nested {
                    match nested_meta {
                        syn::NestedMeta::Meta(meta) => {
                            if kind.is_some() {
                                bail!("Duplication of fields const, mut")
                            }
                            kind = if meta.path().is_ident("const") {
                                Some(CornettoKind::Constant)
                            } else if meta.path().is_ident("mut") {
                                Some(CornettoKind::Testmutable)
                            } else {
                                bail!("Unexpected term {:?}", meta.path().get_ident())
                            };
                        }
                        syn::NestedMeta::Lit(lit) => {
                            if value.is_some() {
                                bail!("Duplication of fields value")
                            }
                            value = Some(lit)
                        } //_ => bail!("Not covered meta type")
                    };
                }
            }
            ret.push(CornettoField {
                kind: kind.expect("Field needs to be const or mut"),
                ident: field.ident.clone().unwrap(),
                value: value.expect("Expected field value literal"),
                ty: field.ty.clone(),
            });
        }
        Ok(ret)
    }
}
