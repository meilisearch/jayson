use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_quote, DeriveInput, Error, FieldsNamed, Generics, Ident};

use crate::{bound, str_name, DataAttrs, Fields};

#[derive(Debug)]
pub struct DerivedStruct<'a> {
    name: &'a Ident,
    fields: Fields<'a>,
    attrs: DataAttrs,
    generics: &'a Generics,
}

impl<'a> DerivedStruct<'a> {
    pub fn parse(input: &'a DeriveInput, fields: &'a FieldsNamed) -> syn::Result<Self> {
        let attrs = DataAttrs::parse(&input.attrs, false)?;
        let fields = Fields::parse(fields)?;
        let name = &input.ident;
        let generics = &input.generics;

        Ok(Self {
            fields,
            attrs,
            name,
            generics,
        })
    }

    pub fn gen(&self) -> syn::Result<TokenStream> {
        let dummy = Ident::new(
            &format!("_IMPL_MINIDESERIALIZE_FOR_{}", self.name),
            Span::call_site(),
        );

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let ident = self.name;

        let err_ty: syn::Type =
            syn::parse_str(self.attrs.err_ty.as_ref().ok_or_else(|| {
                Error::new(Span::call_site(), "Missing ascociasted error type.")
            })?)?;

        let bound = parse_quote!(jayson::Deserialize);
        let bounded_where_clause = bound::where_clause_with_bound(&self.generics, bound);

        let wrapper_generics = bound::with_lifetime_bound(&self.generics, "'__a");
        let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();

        let fieldname = self
            .fields
            .iter()
            .map(|f| &f.field_name)
            .collect::<Vec<_>>();
        let fieldstr = self
            .fields
            .iter()
            .map(|f| {
                str_name(
                    f.field_name.to_string(),
                    self.attrs.rename_all.as_ref(),
                    f.attrs.rename.as_deref(),
                )
            })
            .collect::<Vec<_>>();
        let fieldty = self.fields.iter().map(|f| &f.field_ty);

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
                    fn map(&mut self) -> jayson::__private::Result<jayson::__private::Box<dyn jayson::de::Map<#err_ty> + '_>, #err_ty> {

                        Ok(jayson::__private::Box::new(__State {
                            #(
                                #fieldname: jayson::Jayson::<#err_ty>::default(),
                            )*
                            __out: &mut self.__out,
                        }))
                    }
                }

                impl #wrapper_impl_generics jayson::de::Map<#err_ty> for __State #wrapper_ty_generics #bounded_where_clause {
                    fn key(&mut self, __k: &jayson::__private::str) -> jayson::__private::Result<&mut dyn ::jayson::de::Visitor<#err_ty>, #err_ty> {
                        match __k {
                            #(
                                #fieldstr => jayson::__private::Ok(jayson::Jayson::begin(&mut self.#fieldname)),
                            )*
                            _ => jayson::__private::Ok(<dyn jayson::de::Visitor<#err_ty>>::ignore()),
                        }
                    }

                    fn finish(&mut self) -> jayson::__private::Result<(), #err_ty> {
                        #(
                            let #fieldname = self.#fieldname.take().ok_or(#err_ty::missing_field(#fieldstr))?;
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
}
