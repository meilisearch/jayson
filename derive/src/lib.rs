#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::unseparated_literal_suffix
)]

extern crate proc_macro;

mod bound;
mod derive_enum;
mod derive_struct;

use std::slice::Iter;

use convert_case::{Case, Casing};
use derive_enum::DerivedEnum;
// use derive_enum::DerivedEnum;
use derive_struct::DerivedStruct;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DataEnum, DataStruct, DeriveInput, Error,
    Meta, MetaList,
};

#[derive(Debug)]
enum RenameAll {
    CamelCase,
}

#[derive(Default, Debug)]
struct FieldAttrs {
    rename: Option<String>,
}

impl FieldAttrs {
    fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut this = Self::default();
        for attr in attrs.iter() {
            match attr.parse_meta()? {
                Meta::List(MetaList { path, nested, .. }) => {
                    if path.get_ident().unwrap() == "jayson" {
                        for nested in nested.iter() {
                            match nested {
                                syn::NestedMeta::Meta(meta) => match meta {
                                    Meta::NameValue(nv) => {
                                        match nv.path.get_ident().unwrap().to_string().as_str() {
                                            "rename" => {
                                                let name = match &nv.lit {
                                                    syn::Lit::Str(v) => v.value(),
                                                    _ => {
                                                        return Err(Error::new(
                                                            nv.lit.span(),
                                                            "error should be a string literal",
                                                        ))
                                                    }
                                                };

                                                this.rename.replace(name);
                                            }
                                            _ => {
                                                return Err(Error::new(
                                                    nv.path.span(),
                                                    "Unknown serde attribute",
                                                ))
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(Error::new(
                                            nested.span(),
                                            "Unexpected attribute",
                                        ))
                                    }
                                },
                                syn::NestedMeta::Lit(lit) => {
                                    return Err(Error::new(lit.span(), "Unexpected attribute"))
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(this)
    }
}

#[derive(Debug)]
struct Field<'a> {
    field_name: &'a syn::Ident,
    field_ty: &'a syn::Type,
    attrs: FieldAttrs,
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

        let attrs = FieldAttrs::parse(&field.attrs)?;

        Ok(Self {
            field_name,
            attrs,
            field_ty,
        })
    }

    /// The name of the fields, with potential applied trasformations
    fn str_name(&self, rename_all: Option<&RenameAll>) -> String {
        match self.attrs.rename {
            Some(ref name) => name.clone(),
            None => match rename_all {
                Some(RenameAll::CamelCase) => self.field_name.to_string().to_case(Case::Camel),
                None => self.field_name.to_string(),
            },
        }
    }
}

#[derive(Default, Debug)]
enum TagType {
    Internal(String),
    #[default]
    External,
}

#[derive(Default, Debug)]
struct DataAttrs {
    err_ty: Option<String>,
    rename_all: Option<RenameAll>,
    tag: TagType,
}

impl DataAttrs {
    fn parse(attrs: &[Attribute], is_enum: bool) -> syn::Result<Self> {
        let mut struct_attrs = DataAttrs::default();

        for attr in attrs.iter() {
            match attr.parse_meta()? {
                Meta::List(MetaList { path, nested, .. }) => {
                    if path.get_ident().unwrap() == "jayson" {
                        for nested in nested.iter() {
                            match nested {
                                syn::NestedMeta::Meta(meta) => {
                                    match meta {
                                        Meta::NameValue(nv) => {
                                            match nv.path.get_ident().unwrap().to_string().as_str()
                                            {
                                                "error" => {
                                                    let ty =
                                                        match &nv.lit {
                                                            syn::Lit::Str(v) => v.value(),
                                                            _ => return Err(Error::new(
                                                                nv.lit.span(),
                                                                "error should be a string literal",
                                                            )),
                                                        };
                                                    struct_attrs.err_ty.replace(ty);
                                                }
                                                "rename_all" => {
                                                    let case =
                                                        match &nv.lit {
                                                            syn::Lit::Str(v) => v.value(),
                                                            _ => return Err(Error::new(
                                                                nv.lit.span(),
                                                                "error should be a string literal",
                                                            )),
                                                        };

                                                    let rename_all = match case.as_str() {
                                                    "CamelCase" => RenameAll::CamelCase,
                                                    _ => {
                                                        return Err(Error::new(
                                                            nv.lit.span(),
                                                            "invalid rename all rule. Valid rename rules are: CamelCase",
                                                        ))
                                                    }
                                                };

                                                    struct_attrs.rename_all.replace(rename_all);
                                                }
                                                "tag" => {
                                                    if !is_enum {
                                                        return Err(Error::new(
                                                            nv.path.span(),
                                                            "tag is only supported on enums.",
                                                        ));
                                                    }

                                                    let tag =
                                                        match &nv.lit {
                                                            syn::Lit::Str(v) => v.value(),
                                                            _ => return Err(Error::new(
                                                                nv.lit.span(),
                                                                "tag should be a string literal",
                                                            )),
                                                        };

                                                    struct_attrs.tag = TagType::Internal(tag);
                                                }
                                                _ => {
                                                    return Err(Error::new(
                                                        nv.path.span(),
                                                        "Unknown serde attribute",
                                                    ))
                                                }
                                            }
                                        }
                                        _ => {
                                            return Err(Error::new(
                                                nested.span(),
                                                "Unexpected attribute",
                                            ))
                                        }
                                    }
                                }
                                syn::NestedMeta::Lit(lit) => {
                                    return Err(Error::new(lit.span(), "Unexpected attribute"))
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(struct_attrs)
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

#[proc_macro_derive(Jayson, attributes(jayson))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    match Derived::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
        Ok(derived) => derived
            .gen()
            .unwrap_or_else(|e| e.to_compile_error())
            .into(),
        Err(e) => e.to_compile_error().into(),
    }
}
