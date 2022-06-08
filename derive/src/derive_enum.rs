use proc_macro2::TokenStream;
use quote::quote;

use crate::parse_type::{
    CommonDerivedTypeInfo, NamedFieldsInfo,
    VariantData::{Named, Unit},
    VariantInfo,
};

/// Return a token stream that implements `DeserializeFromValue<E>` for the given derived enum with internal tag
pub fn generate_derive_tagged_enum_impl(
    info: CommonDerivedTypeInfo,
    tag: String,
    variants: Vec<VariantInfo>,
) -> TokenStream {
    // `variant_impls` is the token stream of the code responsible for deserialising
    // all the fields of the enum variants and returning the fully deserialised enum.
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
                // The value must always be a map
                match value {
                    jayson::Value::Map(mut map) => {
                        let tag_value = jayson::Map::remove(&mut map, #tag).ok_or_else(|| <#err_ty as jayson::DeserializeError>::missing_field(#tag))?;

                        let tag_value_string = if let jayson::Value::String(x) = tag_value.into_value() {
                            x
                        } else {
                            // TODO: better error message
                            return ::std::result::Result::Err(<#err_ty as jayson::DeserializeError>::unexpected("The tag should be a string"));
                        };

                        match tag_value_string.as_str() {
                            #(#variants_impls)*
                            // this is the case where the tag exists and is a string, but its value does not
                            // correspond to any valid enum variant name
                            _ => {
                                ::std::result::Result::Err(<#err_ty as jayson::DeserializeError>::unexpected("Incorrect tag value"))
                            }
                        }
                    }
                    // this is the case where the value is not a map
                    _ => {
                        ::std::result::Result::Err(<#err_ty as jayson::DeserializeError>::incorrect_value_kind(&[jayson::ValueKind::Map]))
                    }
                }
            }
        }
    }
}

/// Create a token stream that deserialises all the fields of the enum variant and return
/// the fully deserialised enum.
///
/// The context of the token stream is:
///
/// ```ignore
/// let map: Map
/// match tag_value_string.as_str() {
///     === here ===
///     key => { .. }
/// }
/// ```
///
fn generate_derive_tagged_enum_variant_impl(
    info: &CommonDerivedTypeInfo,
    variant: &VariantInfo,
) -> TokenStream {
    let CommonDerivedTypeInfo {
        unknown_key,
        err_ty,
        ..
    } = info;

    let VariantInfo {
        ident: variant_ident,
        data,
        key_name: variant_key_name,
    } = variant;

    match data {
        Unit => {
            // If the enum variant is a unit variant, there is nothing else to do.
            quote! {
                #variant_key_name => {
                    ::std::result::Result::Ok(Self::#variant_ident)
                }
            }
        }
        Named(fields) => {
            let NamedFieldsInfo {
                field_names,
                field_tys,
                field_defaults,
                missing_field_errors,
                key_names,
            } = fields;

            // The code here is virtually identical to the code of `generate_derive_struct_impl`
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
