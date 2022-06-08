use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput};

use crate::attribute_parser::{DenyUnknownFields, JaysonDefaultFieldAttribute, TagType};
use crate::{
    attribute_parser::{
        read_jayson_data_attributes, read_jayson_field_attributes, JaysonDataAttributes, RenameAll,
    },
    bound,
};

#[derive(Debug)]
pub struct NamedFields {
    pub field_names: Vec<syn::Ident>,
    pub field_tys: Vec<syn::Type>,
    pub field_defaults: Vec<TokenStream>,
    pub missing_field_errors: Vec<TokenStream>,
    pub key_names: Vec<String>,
}

impl NamedFields {
    fn parse(
        fields: syn::FieldsNamed,
        data_attrs: &JaysonDataAttributes,
        err_ty: &syn::Type,
    ) -> syn::Result<Self> {
        let mut field_names = vec![];
        let mut field_tys = vec![];
        let mut key_names = vec![];
        let mut field_defaults = vec![];
        let mut missing_field_errors = vec![];

        for field in fields.named.iter() {
            let field_name = field.ident.clone().unwrap();
            let field_ty = &field.ty;

            let attrs = read_jayson_field_attributes(&field.attrs)?;
            let renamed = attrs.rename.as_ref().map(|i| i.value());
            let key_name = key_name_for_ident(
                field_name.to_string(),
                data_attrs.rename_all.as_ref(),
                renamed.as_deref(),
            );

            let field_default = if let Some(default) = &attrs.default {
                match default {
                    JaysonDefaultFieldAttribute::DefaultTrait => {
                        quote! { ::std::option::Option::Some(::std::default::Default::default()) }
                    }
                    JaysonDefaultFieldAttribute::Function(expr) => {
                        quote! { ::std::option::Option::Some(#expr) }
                    }
                }
            } else {
                quote! { jayson::DeserializeFromValue::<#err_ty>::default() }
            };

            let missing_field_error = match attrs.missing_field_error {
                Some(error_expr) => {
                    quote! { #error_expr }
                }
                None => {
                    quote! { <#err_ty as jayson::DeserializeError>::missing_field(#key_name) }
                }
            };

            field_names.push(field_name);
            field_tys.push(field_ty.clone());
            key_names.push(key_name.clone());
            field_defaults.push(field_default);
            missing_field_errors.push(missing_field_error);
        }

        Ok(Self {
            field_names,
            field_tys,
            key_names,
            field_defaults,
            missing_field_errors,
        })
    }
}

pub struct DerivedTypeInfo {
    pub common: CommonDerivedTypeInfo,
    pub data: DerivedTypeData,
}

pub struct CommonDerivedTypeInfo {
    pub impl_trait_tokens: TokenStream,
    pub unknown_key: TokenStream,
    pub err_ty: syn::Type,
}

pub enum DerivedTypeData {
    Struct(NamedFields),
    Enum {
        tag: TagType,
        variants: Vec<Variant>,
    },
}

pub struct Variant {
    pub ident: Ident,
    pub data: VariantData,
    pub key_name: String,
}

#[derive(Debug)]
pub enum VariantData {
    Unit,
    Named(NamedFields),
}

impl DerivedTypeInfo {
    pub fn parse(input: DeriveInput) -> syn::Result<Self> {
        let attrs = read_jayson_data_attributes(&input.attrs)?;

        let ident = input.ident;
        let (impl_generics, ty_generics, ..) = input.generics.split_for_impl();

        let err_ty: &syn::Type = attrs
            .err_ty
            .as_ref()
            .ok_or_else(|| syn::Error::new(Span::call_site(), "Missing associated error type."))?;

        let bound = quote! { jayson::DeserializeFromValue };
        let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

        let impl_trait_tokens = quote! {
            impl #impl_generics jayson::DeserializeFromValue<#err_ty> for #ident #ty_generics #bounded_where_clause
        };
        {}; // the `impl` above breaks my text editor's syntax highlighting, inserting a pair
            // of curly braces here fixes it

        let data = match input.data {
            Data::Struct(s) => match s.fields {
                syn::Fields::Named(fields) => {
                    DerivedTypeData::Struct(NamedFields::parse(fields, &attrs, err_ty)?)
                }
                syn::Fields::Unnamed(_) => todo!(),
                syn::Fields::Unit => todo!(),
            },
            Data::Enum(e) => {
                let mut parsed_variants = vec![];
                for variant in e.variants {
                    let variant_attrs = read_jayson_data_attributes(&variant.attrs)?;
                    let key_name = key_name_for_ident(
                        variant.ident.to_string(),
                        attrs.rename_all.as_ref(),
                        None,
                    );
                    let data = match variant.fields {
                        syn::Fields::Named(fields) => {
                            VariantData::Named(NamedFields::parse(fields, &variant_attrs, err_ty)?)
                        }
                        syn::Fields::Unnamed(_) => todo!(),
                        syn::Fields::Unit => VariantData::Unit,
                    };
                    parsed_variants.push(Variant {
                        ident: variant.ident,
                        key_name,
                        data,
                    });
                }
                DerivedTypeData::Enum {
                    tag: attrs.tag,
                    variants: parsed_variants,
                }
            }
            Data::Union(_) => todo!(),
        };

        let unknown_key = match &attrs.deny_unknown_fields {
            Some(DenyUnknownFields::DefaultError) => {
                quote! {
                    return ::std::result::Result::Err(<#err_ty as jayson::DeserializeError>::unexpected(&format!("Found unexpected field: {}", key)));
                }
            }
            Some(DenyUnknownFields::Function(func)) => quote! {
                return ::std::result::Result::Err(#func (key));
            },
            None => quote! {},
        };

        Ok(Self {
            common: CommonDerivedTypeInfo {
                impl_trait_tokens,
                unknown_key,
                err_ty: err_ty.clone(),
            },
            data,
        })
    }
}

fn key_name_for_ident(
    ident: String,
    rename_all: Option<&RenameAll>,
    rename: Option<&str>,
) -> String {
    match rename {
        Some(name) => name.to_string(),
        None => match rename_all {
            Some(RenameAll::CamelCase) => ident.to_case(Case::Camel),
            Some(RenameAll::LowerCase) => ident.to_lowercase(),
            None => ident,
        },
    }
}
