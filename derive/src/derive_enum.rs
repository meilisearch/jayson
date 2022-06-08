use proc_macro2::TokenStream;
use quote::quote;

use crate::parse_type::{
    CommonDerivedTypeInfo, NamedFields, Variant,
    VariantData::{Named, Unit},
};

pub fn generate_derive_tagged_enum_impl(
    info: CommonDerivedTypeInfo,
    tag: String,
    variants: Vec<Variant>,
) -> TokenStream {
    let variants_impls = variants
        .into_iter()
        .map(|v| generate_derive_tagged_enum_variant_impl(&info, &v))
        .collect::<Vec<_>>();

    let CommonDerivedTypeInfo {
        impl_trait_tokens,
        err_ty,
        ..
    } = info;

    quote! {
         #impl_trait_tokens {
            fn deserialize_from_value<V: jayson::IntoValue>(value: jayson::Value<V>) -> ::std::result::Result<Self, #err_ty> {
                match value {
                    jayson::Value::Map(mut map) => {
                        let tag_value = jayson::Map::remove(&mut map, #tag).ok_or_else(|| <#err_ty as jayson::DeserializeError>::missing_field(#tag))?;

                        let tag_value_string = if let jayson::Value::String(x) = tag_value.into_value() {
                            x
                        } else {
                            return ::std::result::Result::Err(<#err_ty as jayson::DeserializeError>::unexpected("todo"));
                        };

                        match tag_value_string.as_str() {
                            #(#variants_impls)*
                            _ => {
                                ::std::result::Result::Err(<#err_ty as jayson::DeserializeError>::unexpected("Incorrect tag value"))
                            }
                        }
                    }
                    _ => {
                        ::std::result::Result::Err(<#err_ty as jayson::DeserializeError>::incorrect_value_kind(&[jayson::ValueKind::Map]))
                    }
                }
            }
        }
    }
}

fn generate_derive_tagged_enum_variant_impl(
    info: &CommonDerivedTypeInfo,
    variant: &Variant,
) -> TokenStream {
    let CommonDerivedTypeInfo {
        unknown_key,
        err_ty,
        ..
    } = info;

    let Variant {
        ident: variant_ident,
        data,
        key_name: variant_key_name,
    } = variant;

    match data {
        Unit => {
            quote! {
                #variant_key_name => {
                    ::std::result::Result::Ok(Self::#variant_ident)
                }
            }
        }
        Named(fields) => {
            let NamedFields {
                field_names,
                field_tys,
                field_defaults,
                missing_field_errors,
                key_names,
            } = fields;

            quote! {
                #variant_key_name => {
                    #(
                        let mut #field_names = #field_defaults;
                    )*

                    for (key, value) in jayson::Map::into_iter(map) {
                        match key.as_str() {
                            #(
                                #key_names => {
                                    #field_names = ::std::option::Option::Some(<#field_tys as jayson::DeserializeFromValue<#err_ty>>::deserialize_from_value(jayson::IntoValue::into_value(value))?);
                                }
                            )*
                            key => { #unknown_key }
                        }
                    }

                    ::std::result::Result::Ok(Self::#variant_ident {
                        #(
                            #field_names : #field_names.ok_or_else(|| #missing_field_errors)?,
                        )*
                    })
                }
            }
        }
    }
}
