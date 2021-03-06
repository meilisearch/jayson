use proc_macro2::TokenStream;
use quote::quote;

use crate::parse_type::{
    CommonDerivedTypeInfo,
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
        validate,
    } = info;

    quote! {
         #impl_trait_tokens {
            fn deserialize_from_value<V: jayson::IntoValue>(jayson_value__: jayson::Value<V>, jayson_location__: jayson::ValuePointerRef) -> ::std::result::Result<Self, #err_ty> {
                // The value must always be a map
                let jayson_final__ = match jayson_value__ {
                    jayson::Value::Map(mut jayson_map__) => {
                        let tag_value = jayson::Map::remove(&mut jayson_map__, #tag).ok_or_else(|| {
                            jayson::take_result_content(<#err_ty as jayson::DeserializeError>::missing_field(
                                None,
                                #tag,
                                jayson_location__
                            ))
                        })?;
                        let tag_value_string = match tag_value.into_value() {
                            jayson::Value::String(x) => x,
                            v @ _ => {
                                return ::std::result::Result::Err(
                                    <#err_ty as jayson::DeserializeError>::incorrect_value_kind(
                                        None,
                                        v.kind(),
                                        &[jayson::ValueKind::String],
                                        jayson_location__.push_key(#tag)
                                    )?
                                );
                            }
                        };

                        match tag_value_string.as_str() {
                            #(#variants_impls)*
                            // this is the case where the tag exists and is a string, but its value does not
                            // correspond to any valid enum variant name
                            _ => {
                                ::std::result::Result::Err(
                                    <#err_ty as jayson::DeserializeError>::unexpected(
                                        None,
                                        // TODO: expected one of {expected_tags_list}, found {actual_tag} error message
                                        "Incorrect tag value",
                                        jayson_location__
                                    )?
                                )
                            }
                        }
                    }
                    // this is the case where the value is not a map
                    v @ _ => {
                        ::std::result::Result::Err(
                            <#err_ty as jayson::DeserializeError>::incorrect_value_kind(
                                None,
                                v.kind(),
                                &[jayson::ValueKind::Map],
                                jayson_location__
                            )?
                        )
                    }
                }?;
                #validate
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
    let CommonDerivedTypeInfo { err_ty, .. } = info;

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
            let fields_impl = crate::generate_named_fields_impl(
                fields,
                err_ty,
                quote! { Self :: #variant_ident },
            );
            // The code here is virtually identical to the code of `generate_derive_struct_impl`
            quote! {
                #variant_key_name => {
                    let mut jayson_error__ = None;
                    #fields_impl
                }
            }
        }
    }
}
