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
    let dummy = Ident::new(&format!("_IMPL_JAYSON_FOR_{}", ident), Span::call_site());

    let fieldname = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldty = fields.named.iter().map(|f| &f.ty);
    let fieldstr = fields
        .named
        .iter()
        .map(attr::name_of_field)
        .collect::<Result<Vec<_>>>()?;

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();

    let bound = parse_quote!(jayson::Jayson);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

    let err_ty: Type = syn::parse_str(dbg!(conf.error_ty.as_deref().unwrap_or("jayson::Error")))?;

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: jayson::__private::Option<#ident #ty_generics>,
            }

            impl #impl_generics jayson::Jayson<#err_ty> for #ident #ty_generics #bounded_where_clause {
                fn begin(__out: &mut jayson::__private::Option<Self>) -> &mut dyn jayson::de::Visitor<#err_ty> {
                    unsafe {
                        &mut *{
                            __out
                            as *mut jayson::__private::Option<Self>
                            as *mut __Visitor #ty_generics
                        }
                    }
                }
            }

            impl #impl_generics jayson::de::Visitor<#err_ty> for __Visitor #ty_generics #bounded_where_clause {
                fn map(&mut self) -> Result<jayson::__private::Box<dyn jayson::de::Map<#err_ty> + '_>, #err_ty> {

                    Ok(jayson::__private::Box::new(__State {
                        #(
                            #fieldname: jayson::Jayson::<#err_ty>::default(),
                        )*
                        __out: &mut self.__out,
                    }))
                }
            }

            impl #wrapper_impl_generics jayson::de::Map<#err_ty> for __State #wrapper_ty_generics #bounded_where_clause {
                fn key(&mut self, __k: &jayson::__private::str) -> Result<&mut dyn ::jayson::de::Visitor<#err_ty>, #err_ty> {
                    match __k {
                        #(
                            #fieldstr => jayson::__private::Ok(jayson::Jayson::begin(&mut self.#fieldname)),
                        )*
                        _ => jayson::__private::Ok(<dyn jayson::de::Visitor<#err_ty>>::ignore()),
                    }
                }

                fn finish(&mut self) -> Result<(), #err_ty> {
                    #(
                        let #fieldname = self.#fieldname.take().ok_or(#err_ty::unexpected())?;
                    )*
                    *self.__out = jayson::__private::Some(#ident {
                        #(
                            #fieldname,
                        )*
                    });
                    jayson::__private::Ok(())
                }
            }

            struct __State #wrapper_impl_generics #where_clause {
                #(
                    #fieldname: jayson::__private::Option<#fieldty>,
                )*
                __out: &'__a mut jayson::__private::Option<#ident #ty_generics>,
            }

        };
    })
}

pub fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Enums with generics are not supported",
        ));
    }

    let ident = &input.ident;
    let dummy = Ident::new(&format!("_IMPL_JAYSON_FOR_{}", ident), Span::call_site());

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
                __out: jayson::__private::Option<#ident>,
            }

            impl jayson::Jayson for #ident {
                fn begin(__out: &mut jayson::__private::Option<Self>) -> &mut dyn jayson::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut jayson::__private::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }

            impl jayson::de::Visitor for __Visitor {
                fn string(&mut self, s: &jayson::__private::str) -> jayson::Result<()> {
                    let value = match s {
                        #( #names => #ident::#var_idents, )*
                        _ => return jayson::__private::Err(jayson::Error),
                    };
                    self.__out = jayson::__private::Some(value);
                    jayson::__private::Ok(())
                }
            }
        };
    })
}
