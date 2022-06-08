extern crate proc_macro;

mod attribute_parser;
mod bound;
mod derive_enum;
mod derive_struct;
mod parse_type;

use attribute_parser::TagType;
use parse_type::DerivedTypeInfo;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DeserializeFromValue, attributes(jayson, serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match DerivedTypeInfo::parse(input) {
        Ok(derived_type_info) => match derived_type_info.data {
            parse_type::DerivedTypeData::Struct(fields) => {
                derive_struct::generate_derive_struct_impl(derived_type_info.common, fields).into()
            }
            parse_type::DerivedTypeData::Enum { tag, variants } => match tag {
                TagType::Internal(tag_key) => derive_enum::generate_derive_tagged_enum_impl(
                    derived_type_info.common,
                    tag_key,
                    variants,
                )
                .into(),
                TagType::External => todo!(),
            },
        },
        Err(e) => e.to_compile_error().into(),
    }
}
