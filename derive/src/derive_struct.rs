use proc_macro2::TokenStream;
use quote::quote;

use crate::parse_type::{CommonDerivedTypeInfo, NamedFields};

pub fn generate_derive_struct_impl(
    info: CommonDerivedTypeInfo,
    fields: NamedFields,
) -> TokenStream {
    let CommonDerivedTypeInfo {
        impl_trait_tokens,
        unknown_key,
        err_ty,
    } = info;

    let NamedFields {
        field_names,
        field_tys,
        field_defaults,
        missing_field_errors,
        key_names,
    } = fields;

    quote! {
         #impl_trait_tokens {
            fn deserialize_from_value<V: jayson::IntoValue>(value: jayson::Value<V>) -> ::std::result::Result<Self, #err_ty> {
                match value {
                    jayson::Value::Map(map) => {
                        #(
                            let mut #field_names: Option<_> = #field_defaults;
                        )*
                        for (key, value) in jayson::Map::into_iter(map) {
                            match key.as_str() {
                                #(
                                    #key_names => {
                                        #field_names = ::std::option::Option::Some(
                                            <#field_tys as jayson::DeserializeFromValue<#err_ty>>::deserialize_from_value(
                                                jayson::IntoValue::into_value(value)
                                            )?
                                        );
                                    }
                                )*
                                key => { #unknown_key }
                            }
                        }
                        ::std::result::Result::Ok(Self {
                            #(
                                #field_names : #field_names.ok_or_else(|| #missing_field_errors)?,
                            )*
                        })
                    }
                    _ => {
                        ::std::result::Result::Err(
                            <#err_ty as jayson::DeserializeError>::incorrect_value_kind(
                                &[jayson::ValueKind::Map]
                            )
                        )
                    }
                }
            }
        }
    }
}
