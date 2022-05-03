#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::unseparated_literal_suffix
)]

extern crate proc_macro;

mod attr;
mod bound;
mod de;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput,
    Error, Fields, FieldsNamed, Meta, MetaList,
};

#[derive(Debug)]
enum Rename {
    Name(String),
    CamelCase,
}

#[derive(Default, Debug)]
struct FieldAttrs {
    rename: Option<Rename>,
}

impl FieldAttrs {
    fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut this = Self::default();
        for attr in attrs.iter() {
            match attr.parse_meta()? {
                Meta::List(MetaList { path, nested, .. }) => {
                    if path.get_ident().unwrap() == "serde" {
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

                                                this.rename.replace(Rename::Name(name));
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
    fn str_name(&self) -> String {
        match self.attrs.rename {
            Some(Rename::CamelCase) => todo!("camel case not yet supported"),
            Some(Rename::Name(ref name)) => name.clone(),
            None => self.field_name.to_string(),
        }
    }
}

#[derive(Default, Debug)]
struct StructAttrs {
    err_ty: Option<String>,
}

#[derive(Debug)]
struct DerivedStruct<'a> {
    name: &'a syn::Ident,
    fields: Vec<Field<'a>>,
    attrs: StructAttrs,

    generics: &'a syn::Generics,
}

impl<'a> DerivedStruct<'a> {
    fn parse_stuct_attrs(attrs: &[Attribute]) -> syn::Result<StructAttrs> {
        let mut struct_attrs = StructAttrs::default();

        for attr in attrs.iter() {
            match attr.parse_meta()? {
                Meta::List(MetaList { path, nested, .. }) => {
                    if path.get_ident().unwrap() == "serde" {
                        for nested in nested.iter() {
                            match nested {
                                syn::NestedMeta::Meta(meta) => match meta {
                                    Meta::NameValue(nv) => {
                                        match nv.path.get_ident().unwrap().to_string().as_str() {
                                            "error" => {
                                                let ty = match &nv.lit {
                                                    syn::Lit::Str(v) => v.value(),
                                                    _ => {
                                                        return Err(Error::new(
                                                            nv.lit.span(),
                                                            "error should be a string literal",
                                                        ))
                                                    }
                                                };
                                                struct_attrs.err_ty.replace(ty);
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
        Ok(struct_attrs)
    }

    fn parse_fields(fields: &FieldsNamed) -> syn::Result<Vec<Field>> {
        let mut out_fields = Vec::new();
        for field in fields.named.iter() {
            let field = Field::parse(field)?;

            out_fields.push(field);
        }

        Ok(out_fields)
    }

    fn parse(input: &'a DeriveInput, fields: &'a FieldsNamed) -> syn::Result<Self> {
        let attrs = Self::parse_stuct_attrs(&input.attrs)?;
        let fields = Self::parse_fields(fields)?;
        let name = &input.ident;
        let generics = &input.generics;

        Ok(dbg!(Self {
            fields,
            attrs,
            name,
            generics,
        }))
    }

    fn gen(&self) -> syn::Result<proc_macro2::TokenStream> {
        let dummy = syn::Ident::new(
            &format!("_IMPL_MINIDESERIALIZE_FOR_{}", self.name),
            Span::call_site(),
        );

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let ident = self.name;

        let err_ty: syn::Type =
            syn::parse_str(self.attrs.err_ty.as_ref().ok_or_else(|| {
                Error::new(Span::call_site(), "Missing ascociasted error type.")
            })?)?;

        let bound = parse_quote!(miniserde::Deserialize);
        let bounded_where_clause = bound::where_clause_with_bound(&self.generics, bound);

        let wrapper_generics = bound::with_lifetime_bound(&self.generics, "'__a");
        let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();

        let fieldname = self
            .fields
            .iter()
            .map(|f| &f.field_name)
            .collect::<Vec<_>>();
        let fieldstr = self.fields.iter().map(|f| f.str_name());
        let fieldty = self.fields.iter().map(|f| &f.field_ty);

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
}

enum Derived<'a> {
    Struct(DerivedStruct<'a>),
}

impl<'a> Derived<'a> {
    fn from_derive_input(input: &'a DeriveInput) -> syn::Result<Self> {
        match &input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => Ok(Self::Struct(DerivedStruct::parse(&input, fields)?)),
            Data::Enum(_enumeration) => todo!("support enums"),
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

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    match Derived::from_derive_input(&parse_macro_input!(input as DeriveInput)) {
        Ok(derived) => match derived {
            Derived::Struct(s) => s.gen().unwrap().into(),
        },
        Err(e) => e.to_compile_error().into(),
    }
}
