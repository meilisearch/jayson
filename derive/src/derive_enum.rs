use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, DeriveInput, Error, Generics, Ident, Lit,
    LitStr, Type,
};

use crate::{bound, DataAttrs, Fields};

#[derive(Debug)]
pub struct DerivedEnum<'a> {
    name: &'a Ident,
    generics: &'a Generics,
    attrs: DataAttrs,
    variants: Vec<Variant<'a>>,
}

#[derive(Debug)]
enum Variant<'a> {
    Unit { name: &'a Ident },
    Named { name: &'a Ident, fields: Fields<'a> },
}

impl<'a> Variant<'a> {
    fn gen(&self, enum_ident: &Ident, err_ty: &Type) -> syn::Result<TokenStream> {
        match self {
            Variant::Unit { name } => {
                let name_str = Lit::Str(LitStr::new(&name.to_string(), Span::call_site()));
                Ok(quote! {
                    #name_str => {
                        self.__out.replace(#enum_ident::#name);
                    }
                })
            }
            Variant::Named { name, fields } => {
                let name_str = Lit::Str(LitStr::new(&name.to_string(), Span::call_site()));
                let field_impls = fields.iter().map(|f| {
                    let ident = f.field_name;
                    // TODO: handle rename all
                    let name = f.str_name(None);
                    quote! {
                        let mut #ident = None;
                        let v = jayson::Jayson::begin(&mut #ident);
                        let val = std::mem::replace(
                            self.object
                                .get_mut(#name)
                                .ok_or_else(|| #err_ty::missing_field(#name))?,
                            jayson::json::Value::Null,
                        );
                        jayson::__private::apply_object_to_visitor(v, val)?;
                    }
                });

                let field_names = fields.iter().map(|f| f.field_name);
                Ok(quote! {
                    #name_str => {
                        #(#field_impls)*
                        self.__out.replace(#enum_ident::#name {
                            #(
                                #field_names: #field_names.unwrap(),
                            )*
                        });
                    }
                })
            }
        }
    }
}

impl<'a> DerivedEnum<'a> {
    fn parse_variants(
        variants: &'a Punctuated<syn::Variant, Comma>,
    ) -> syn::Result<Vec<Variant<'a>>> {
        let mut out_variants = Vec::new();
        for variant in variants.iter() {
            let variant = match variant.fields {
                syn::Fields::Named(ref named) => {
                    let name = &variant.ident;
                    let fields = Fields::parse(named)?;

                    Variant::Named { name, fields }
                }
                syn::Fields::Unit => {
                    let name = &variant.ident;

                    Variant::Unit { name }
                }
                syn::Fields::Unnamed(_) => unimplemented!("unsupported unit struct variant"),
            };

            out_variants.push(variant);
        }

        Ok(out_variants)
    }

    pub fn parse(
        input: &'a DeriveInput,
        variants: &'a Punctuated<syn::Variant, Comma>,
    ) -> syn::Result<Self> {
        let name = &input.ident;
        let generics = &input.generics;
        let attrs = DataAttrs::parse(&input.attrs, true)?;
        let variants = Self::parse_variants(variants)?;

        Ok(Self {
            name,
            generics,
            attrs,
            variants,
        })
    }

    pub fn gen(&self) -> syn::Result<TokenStream> {
        match self.attrs.tag {
            crate::TagType::Internal(ref name) => self.gen_internally_tagged(name),
            crate::TagType::External => {
                unimplemented!("externally tagged enums are not supported yet")
            }
        }
    }

    fn gen_internally_tagged(&self, name: &str) -> syn::Result<TokenStream> {
        let dummy = Ident::new(
            &format!("_IMPL_JAYSON_FOR_{}", self.name),
            Span::call_site(),
        );
        let err_ty: Type =
            syn::parse_str(self.attrs.err_ty.as_ref().ok_or_else(|| {
                Error::new(Span::call_site(), "Missing ascociasted error type.")
            })?)?;

        let tag_name = Lit::Str(LitStr::new(name, Span::call_site()));
        let variant_match_branch = self
            .variants
            .iter()
            .map(|v| v.gen(self.name, &err_ty))
            .collect::<syn::Result<Vec<_>>>()?;

        let ident = self.name;

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let bound = parse_quote!(jayson::Deserialize);
        let bounded_where_clause = bound::where_clause_with_bound(&self.generics, bound);

        dbg!();

        Ok(quote! {
            #[allow(non_upper_case_globals)]
            const #dummy: () = {
                #[repr(C)]
                struct __Visitor #impl_generics #where_clause {
                    __out: jayson::__private::Option<#ident #ty_generics>,
                }

                impl #impl_generics jayson::de::Visitor<#err_ty> for __Visitor #ty_generics #bounded_where_clause {
                    fn map(&mut self) -> Result<jayson::__private::Box<dyn jayson::de::Map<#err_ty> + '_>, #err_ty> {
                        Ok(jayson::__private::Box::new(__Builder {
                            __out: &mut self.__out,
                            object: jayson::json::Object::new(),
                            key: jayson::__private::None,
                            value: jayson::__private::None,
                        }))
                    }
                }

                struct __Builder<'a> {
                    __out: &'a mut jayson::__private::Option<#ident>,
                    object: jayson::json::Object,
                    key: Option<jayson::__private::String>,
                    value: Option<jayson::json::Value>,
                }

                impl<'a> __Builder<'a> {
                    fn shift(&mut self) {
                        if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                            self.object.insert(k, v);
                        }
                    }
                }

                impl<'a> jayson::de::Map<#err_ty> for __Builder<'a> {
                    fn key(&mut self, k: &str) -> jayson::__private::Result<&mut dyn jayson::de::Visitor<#err_ty>, #err_ty> {
                        self.shift();
                        self.key = Some(k.to_owned());
                        Ok(jayson::Jayson::begin(&mut self.value))
                    }

                    fn finish(&mut self) -> jayson::__private::Result<(), #err_ty> {
                        self.shift();
                        match self.object.get(#tag_name).and_then(|o| o.as_str()) {
                            Some(variant) => match variant {
                                #(#variant_match_branch)*
                                _ => todo!(),
                            }
                            None => todo!(),
                        }

                        Ok(())
                    }
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
           };
        })
    }
}
