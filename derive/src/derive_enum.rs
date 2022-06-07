use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, DeriveInput, Error, Generics, Ident, Type,
};

use crate::{
    attribute_parser::JaysonDataAttributes,
    attribute_parser::{
        read_jayson_data_attributes, DenyUnknownFields, JaysonDefaultFieldAttribute,
    },
    bound, str_name, Fields, RenameAll,
};

#[derive(Debug)]
pub struct DerivedEnum<'a> {
    name: &'a Ident,
    generics: &'a Generics,
    attrs: JaysonDataAttributes,
    variants: Vec<Variant<'a>>,
}

#[derive(Debug)]
enum Variant<'a> {
    Unit {
        name: &'a Ident,
        attributes: JaysonDataAttributes,
    },
    Named {
        name: &'a Ident,
        fields: Fields<'a>,
        attributes: JaysonDataAttributes,
    },
}

impl<'a> Variant<'a> {
    fn attributes(&self) -> &JaysonDataAttributes {
        match self {
            Variant::Unit { attributes, .. } => attributes,
            Variant::Named { attributes, .. } => attributes,
        }
    }
}

impl<'a> Variant<'a> {
    fn gen(
        &self,
        enum_ident: &Ident,
        err_ty: &Type,
        tag_name: &str,
        rename_all: Option<&RenameAll>,
        deny_unknown_fields: Option<&DenyUnknownFields>,
    ) -> syn::Result<TokenStream> {
        match self {
            Variant::Unit { name, .. } => {
                let name_str = str_name(name.to_string(), rename_all, None);
                Ok(quote! {
                    #name_str => {
                        self.__out.replace(#enum_ident::#name);
                    }
                })
            }
            Variant::Named { name, fields, .. } => {
                let name_str = str_name(name.to_string(), rename_all, None);
                let field_defaults = fields.iter().map(|f| {
                    if let Some(default) = &f.attrs.default {
                        match default {
                            JaysonDefaultFieldAttribute::DefaultTrait => {
                                quote! {
                                    jayson::__private::Option::Some(::std::default::Default::default())
                                }
                            }
                            JaysonDefaultFieldAttribute::Function(
                                expr,
                            ) => {
                                quote! {
                                    jayson::__private::Option::Some(#expr)
                                }
                            }
                        }
                    } else {
                        quote! {
                            jayson::Jayson::<#err_ty>::default()
                        }
                    }
                });

                let unknown_key = match deny_unknown_fields {
                    Some(DenyUnknownFields::DefaultError) => quote! {
                        return jayson::__private::Err(<#err_ty>::unexpected("Found unexpected field: {key}"));
                    },
                    Some(DenyUnknownFields::Function(func)) => quote! {
                        return jayson::__private::Err(#func (key));
                    },
                    None => quote! {},
                };

                let field_idents_decl = fields.iter().map(|f| {
                    let ident = f.field_name;
                    quote! {
                        let mut #ident = jayson::__private::None;
                    }
                });

                let fieldstrs = fields
                    .iter()
                    .map(|f| {
                        str_name(
                            f.field_name.to_string(),
                            self.attributes().rename_all.as_ref(),
                            f.attrs.rename.as_ref().map(|i| i.value()).as_deref(),
                        )
                    })
                    .collect::<Vec<_>>();

                let missing_field_errors =  fields
                    .iter()
                    .zip(fieldstrs.iter())
                    .map(|(f, fieldstr)| match &f.attrs.missing_field_error {
                        Some(error_expr) => {
                            quote! { #error_expr }
                        }
                        None => {
                            quote! { <#err_ty as jayson::de::VisitorError>::missing_field(#fieldstr) }
                        }
                    });

                let field_matches = fields.iter().zip(fieldstrs.iter()).map(|(f, name)| {
                    let ident = f.field_name;

                    quote! {
                        #name => {
                            let v = jayson::Jayson::begin(&mut #ident);
                            let value = std::mem::replace(value, jayson::json::Value::Null);
                            jayson::__private::apply_object_to_visitor(v, value)?;
                        }
                    }
                });

                let fields_impl = quote! {
                    #(#field_idents_decl)*
                    for (key, value) in self.object.iter_mut() {
                        match key.as_str() {
                            #(#field_matches)*
                            #tag_name => {}
                            key => {
                                #unknown_key
                            }
                        }
                    }
                };

                let field_names = fields.iter().map(|f| f.field_name);
                Ok(quote! {
                    #name_str => {
                        #fields_impl
                        self.__out.replace(#enum_ident::#name {
                            #(
                                #field_names: #field_names
                                    .or_else(|| #field_defaults)
                                    .ok_or_else(|| { #missing_field_errors })?,
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
                    let attributes = read_jayson_data_attributes(&variant.attrs)?;
                    // TODO: return error when tag or error or deny_unknown_fields is present
                    let name = &variant.ident;
                    let fields = Fields::parse(named)?;
                    Variant::Named {
                        name,
                        fields,
                        attributes,
                    }
                }
                syn::Fields::Unit => {
                    let name = &variant.ident;
                    let attributes = read_jayson_data_attributes(&variant.attrs)?;
                    // TODO: return error when tag or error or rename_all or deny_unknown_fields is present
                    Variant::Unit { name, attributes }
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
        let attrs = read_jayson_data_attributes(&input.attrs)?;
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

    fn gen_internally_tagged(&self, tag_name: &str) -> syn::Result<TokenStream> {
        let dummy = Ident::new(
            &format!("_IMPL_JAYSON_FOR_{}", self.name),
            Span::call_site(),
        );
        let err_ty: &Type = self
            .attrs
            .err_ty
            .as_ref()
            .ok_or_else(|| Error::new(Span::call_site(), "Missing associated error type."))?;

        let variant_match_branch = self
            .variants
            .iter()
            .map(|v| {
                v.gen(
                    self.name,
                    &err_ty,
                    tag_name,
                    self.attrs.rename_all.as_ref(),
                    self.attrs.deny_unknown_fields.as_ref(),
                )
            })
            .collect::<syn::Result<Vec<_>>>()?;

        let ident = self.name;

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let bound = parse_quote!(jayson::Deserialize);
        let bounded_where_clause = bound::where_clause_with_bound(&self.generics, bound);

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
                                found => return Err(#err_ty::unexpected(&format!("unexpected value for `{}`: `{}`", #tag_name ,found))),
                            }
                            None => return Err(#err_ty::missing_field(#tag_name)),
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
