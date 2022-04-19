use crate::{attr, bound};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed,
    Ident, Lit, Meta, Result, Type,
};

#[derive(Default)]
pub struct DeserConfig {
    error_ty: Option<String>,
}

fn parse_config(attrs: &[Attribute]) -> Result<DeserConfig> {
    let mut out = DeserConfig::default();
    for attr in attrs {
        let meta = attr.parse_meta()?;
        if let Meta::List(list) = meta {
            for nested in list.nested {
                match nested {
                    syn::NestedMeta::Lit(_) => todo!(),
                    syn::NestedMeta::Meta(meta) => match meta {
                        Meta::NameValue(nv) => {
                            if nv.path.get_ident().unwrap().to_string() == "error" {
                                match nv.lit {
                                    Lit::Str(t) => {
                                        out.error_ty.replace(t.value());
                                    }
                                    _ => todo!("blabla"),
                                }
                            }
                        }
                        Meta::List(_) => todo!("list"),
                        Meta::Path(_) => todo!("path"),
                    },
                }
            }
        }
    }

    Ok(out)
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let conf = parse_config(&input.attrs)?;
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => derive_struct(&input, fields, conf),
        Data::Enum(enumeration) => derive_enum(&input, enumeration),
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

pub fn derive_struct(
    input: &DeriveInput,
    fields: &FieldsNamed,
    conf: DeserConfig,
) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let fieldname = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldty = fields.named.iter().map(|f| &f.ty);
    let fieldstr = fields
        .named
        .iter()
        .map(attr::name_of_field)
        .collect::<Result<Vec<_>>>()?;

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();

    let bound = parse_quote!(miniserde::Deserialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

    let err_ty: Type =
        syn::parse_str(dbg!(conf.error_ty.as_deref().unwrap_or("miniserde::Error")))?;

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: miniserde::__private::Option<#ident #ty_generics>,
            }

            impl #impl_generics miniserde::Deserialize<#err_ty> for #ident #ty_generics #bounded_where_clause {
                fn begin(__out: &mut miniserde::__private::Option<Self>) -> &mut dyn miniserde::de::Visitor<#err_ty> {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::__private::Option<Self>
                            as *mut __Visitor #ty_generics
                        }
                    }
                }
            }

            impl #impl_generics miniserde::de::Visitor<#err_ty> for __Visitor #ty_generics #bounded_where_clause {
                fn map(&mut self) -> Result<miniserde::__private::Box<dyn miniserde::de::Map<#err_ty> + '_>, #err_ty> {

                    Ok(miniserde::__private::Box::new(__State {
                        #(
                            #fieldname: miniserde::Deserialize::<#err_ty>::default(),
                        )*
                        __out: &mut self.__out,
                    }))
                }
            }

            impl #wrapper_impl_generics miniserde::de::Map<#err_ty> for __State #wrapper_ty_generics #bounded_where_clause {
                fn key(&mut self, __k: &miniserde::__private::str) -> Result<&mut dyn ::miniserde::de::Visitor<#err_ty>, #err_ty> {
                    match __k {
                        #(
                            #fieldstr => miniserde::__private::Ok(miniserde::Deserialize::begin(&mut self.#fieldname)),
                        )*
                        _ => miniserde::__private::Ok(<dyn miniserde::de::Visitor<#err_ty>>::ignore()),
                    }
                }

                fn finish(&mut self) -> Result<(), #err_ty> {
                    #(
                        let #fieldname = self.#fieldname.take().ok_or(#err_ty::unexpected())?;
                    )*
                    *self.__out = miniserde::__private::Some(#ident {
                        #(
                            #fieldname,
                        )*
                    });
                    miniserde::__private::Ok(())
                }
            }

            struct __State #wrapper_impl_generics #where_clause {
                #(
                    #fieldname: miniserde::__private::Option<#fieldty>,
                )*
                __out: &'__a mut miniserde::__private::Option<#ident #ty_generics>,
            }

        };
    })
}

// pub fn derive_struct_default(input: &DeriveInput, fields: &FieldsNamed) -> Result<TokenStream> {
//     let ident = &input.ident;
//     let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
//     let dummy = Ident::new(
//         &format!("_IMPL_MINIDESERIALIZE_FOR_{}", ident),
//         Span::call_site(),
//     );
//
//     let fieldname = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
//     let fieldty = fields.named.iter().map(|f| &f.ty);
//     let fieldstr = fields
//         .named
//         .iter()
//         .map(attr::name_of_field)
//         .collect::<Result<Vec<_>>>()?;
//
//     let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
//     let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();
//
//     let params = wrapper_generics
//         .params
//         .iter()
//         .cloned()
//         .chain(
//             Some(GenericParam::Type(parse_quote!(
//                 E: miniserde::de::VisitorError
//             )))
//             .into_iter(),
//         )
//         .collect();
//
//     let wrapper_generics_map = Generics {
//         params,
//         ..wrapper_generics.clone()
//     };
//     let (wrapper_impl_generics_map, _, _) = wrapper_generics_map.split_for_impl();
//
//     let bound = parse_quote!(miniserde::Deserialize);
//     let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);
//
//     Ok(quote! {
//         #[allow(non_upper_case_globals)]
//         const #dummy: () = {
//             #[repr(C)]
//             struct __Visitor #impl_generics #where_clause {
//                 __out: miniserde::__private::Option<#ident #ty_generics>,
//             }
//
//             impl<E: miniserde::de::VisitorError> #impl_generics miniserde::Deserialize<E> for #ident #ty_generics #bounded_where_clause {
//                 fn begin(__out: &mut miniserde::__private::Option<Self>) -> &mut dyn miniserde::de::Visitor<E> {
//                     unsafe {
//                         &mut *{
//                             __out
//                             as *mut miniserde::__private::Option<Self>
//                             as *mut __Visitor #ty_generics
//                         }
//                     }
//                 }
//             }
//
//             impl<E: miniserde::de::VisitorError> #impl_generics miniserde::de::Visitor<E> for __Visitor #ty_generics #bounded_where_clause {
//                 fn map(&mut self) -> Result<miniserde::__private::Box<dyn miniserde::de::Map<E> + '_>, E> {
//
//                     Ok(miniserde::__private::Box::new(__State {
//                         #(
//                             #fieldname: miniserde::Deserialize::default(),
//                         )*
//                         __out: &mut self.__out,
//                     }))
//                 }
//             }
//
//             impl #wrapper_impl_generics_map miniserde::de::Map<E> for __State #wrapper_ty_generics #bounded_where_clause {
//                 fn key(&mut self, __k: &miniserde::__private::str) -> Result<&mut dyn ::miniserde::de::Visitor<E>, E> {
//                     match __k {
//                         #(
//                             #fieldstr => miniserde::__private::Ok(miniserde::Deserialize::begin(&mut self.#fieldname)),
//                         )*
//                         _ => miniserde::__private::Ok(<dyn miniserde::de::Visitor<E>>::ignore()),
//                     }
//                 }
//
//                 fn finish(&mut self) -> Result<(), E> {
//                     #(
//                         let #fieldname = self.#fieldname.take().ok_or(E::unexpected())?;
//                     )*
//                     *self.__out = miniserde::__private::Some(#ident {
//                         #(
//                             #fieldname,
//                         )*
//                     });
//                     miniserde::__private::Ok(())
//                 }
//             }
//
//             struct __State #wrapper_impl_generics #where_clause {
//                 #(
//                     #fieldname: miniserde::__private::Option<#fieldty>,
//                 )*
//                 __out: &'__a mut miniserde::__private::Option<#ident #ty_generics>,
//             }
//
//         };
//     })
// }

pub fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Enums with generics are not supported",
        ));
    }

    let ident = &input.ident;
    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let var_idents = enumeration
        .variants
        .iter()
        .map(|variant| match variant.fields {
            Fields::Unit => Ok(&variant.ident),
            _ => Err(Error::new_spanned(
                variant,
                "Invalid variant: only simple enum variants without fields are supported",
            )),
        })
        .collect::<Result<Vec<_>>>()?;
    let names = enumeration
        .variants
        .iter()
        .map(attr::name_of_variant)
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor {
                __out: miniserde::__private::Option<#ident>,
            }

            impl miniserde::Deserialize for #ident {
                fn begin(__out: &mut miniserde::__private::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::__private::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }

            impl miniserde::de::Visitor for __Visitor {
                fn string(&mut self, s: &miniserde::__private::str) -> miniserde::Result<()> {
                    let value = match s {
                        #( #names => #ident::#var_idents, )*
                        _ => return miniserde::__private::Err(miniserde::Error),
                    };
                    self.__out = miniserde::__private::Some(value);
                    miniserde::__private::Ok(())
                }
            }
        };
    })
}
