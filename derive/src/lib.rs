#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::unseparated_literal_suffix
)]

extern crate proc_macro;

mod attribute_parser;
mod bound;
mod derive_enum;
mod derive_struct;

use attribute_parser::{RenameAll, TagType};
use std::slice::Iter;

use attribute_parser::{read_jayson_field_attributes, JaysonFieldAttributes};
use convert_case::{Case, Casing};
use derive_enum::DerivedEnum;
use derive_struct::DerivedStruct;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Error};

#[derive(Debug)]
struct Field<'a> {
    field_name: &'a syn::Ident,
    field_ty: &'a syn::Type,
    attrs: JaysonFieldAttributes,
}

#[derive(Debug)]
struct Fields<'a>(Vec<Field<'a>>);

impl<'a> Fields<'a> {
    fn parse(fields: &'a syn::FieldsNamed) -> syn::Result<Self> {
        let mut out_fields = Vec::new();
        for field in fields.named.iter() {
            let field = Field::parse(field)?;

            out_fields.push(field);
        }

        Ok(Self(out_fields))
    }

    fn iter(&self) -> Iter<Field<'a>> {
        self.0.iter()
    }
}

impl<'a> Field<'a> {
    fn parse(field: &'a syn::Field) -> syn::Result<Self> {
        let field_name = match field.ident {
            Some(ref ident) => ident,
            None => {
                return Err(Error::new(
                    Span::call_site(),
                    "currently only structs and enums are supported by this derive",
                ))
            }
        };

        let field_ty = &field.ty;

        let attrs = read_jayson_field_attributes(&field.attrs)?;

        Ok(Self {
            field_name,
            attrs,
            field_ty,
        })
    }
}

fn str_name(name: String, rename_all: Option<&RenameAll>, rename: Option<&str>) -> String {
    match rename {
        Some(name) => name.to_string(),
        None => match rename_all {
            Some(RenameAll::CamelCase) => name.to_case(Case::Camel),
            Some(RenameAll::LowerCase) => name.to_lowercase(),
            None => name,
        },
    }
}

enum Derived<'a> {
    Struct(DerivedStruct<'a>),
    Enum(DerivedEnum<'a>),
}

impl<'a> Derived<'a> {
    fn gen(&self) -> syn::Result<proc_macro2::TokenStream> {
        match self {
            Derived::Struct(s) => s.gen(),
            Derived::Enum(e) => e.gen(),
        }
    }
}

impl<'a> Derived<'a> {
    fn from_derive_input(input: &'a DeriveInput) -> syn::Result<Self> {
        match &input.data {
            Data::Struct(DataStruct {
                fields: syn::Fields::Named(fields),
                ..
            }) => Ok(Self::Struct(DerivedStruct::parse(&input, fields)?)),
            Data::Enum(DataEnum { variants, .. }) => {
                Ok(Self::Enum(DerivedEnum::parse(&input, variants)?))
            }
            Data::Struct(_) => Err(Error::new(
                Span::call_site(),
                "currently only structs with named fields are supported",
            )),
            Data::Union(_) => Err(Error::new(
                Span::call_site(),
                "currently only structs and enums are supported by this derive",
            )),
        }
    }
}

#[proc_macro_derive(Jayson, attributes(jayson, serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    match Derived::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
        Ok(derived) => derived
            .gen()
            .unwrap_or_else(|e| e.to_compile_error())
            .into(),
        Err(e) => e.to_compile_error().into(),
    }
}
