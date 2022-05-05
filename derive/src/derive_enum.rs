use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, token::Comma, DeriveInput, Error, Generics, Ident};

use crate::{bound, DataAttrs, Fields};

pub struct DerivedEnum<'a> {
    name: &'a Ident,
    generics: &'a Generics,
    attrs: DataAttrs,
    variants: Vec<Variant<'a>>,
}

enum Variant<'a> {
    Unit { name: &'a Ident },
    Named { name: &'a Ident, fields: Fields<'a> },
}

impl<'a> DerivedEnum<'a> {
    fn parse_variants(
        variants: &'a Punctuated<syn::Variant, Comma>,
    ) -> syn::Result<Vec<Variant<'a>>> {
        let mut out_variants = Vec::new();
        for variant in variants.iter() {
            let variant = match variant.fields {
                syn::Fields::Named(ref named) => {
                    let name = &variant.ident;
                    let fields = Fields::parse(named)?;

                    Variant::Named { name, fields }
                }
                syn::Fields::Unit => {
                    let name = &variant.ident;

                    Variant::Unit { name }
                }
                syn::Fields::Unnamed(_) => unimplemented!("unsupported unit struct variant"),
            };

            out_variants.push(variant);
        }

        Ok(out_variants)
    }

    pub fn parse(
        input: &'a DeriveInput,
        variants: &'a Punctuated<syn::Variant, Comma>,
    ) -> syn::Result<Self> {
        let name = &input.ident;
        let generics = &input.generics;
        let attrs = DataAttrs::parse(&input.attrs, true)?;
        let variants = Self::parse_variants(variants)?;

        Ok(Self {
            name,
            generics,
            attrs,
            variants,
        })
    }

    pub fn gen(&self) -> syn::Result<TokenStream> {
        match self.attrs.tag {
            crate::TagType::Internal(ref name) => self.gen_internally_tagged(name),
            crate::TagType::External => {
                unimplemented!("externally tagged enums are not supported yet")
            }
        }
    }

    fn gen_internally_tagged(&self, _name: &str) -> syn::Result<TokenStream> {
        Ok(quote! {})
    }
}
