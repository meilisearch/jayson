use proc_macro2::Ident;
use syn::{parenthesized, parse::ParseStream, parse2, Attribute, Expr, ExprPath, LitStr, Token};

#[derive(Debug)]
pub enum JaysonDefaultFieldAttribute {
    DefaultTrait,
    Function(Expr),
}

#[derive(Default, Debug)]
pub struct FieldAttributes {
    pub rename: Option<LitStr>,
    pub default: Option<JaysonDefaultFieldAttribute>,
    pub missing_field_error: Option<Expr>,
}

impl FieldAttributes {
    fn overwrite(&mut self, other: FieldAttributes) {
        if let Some(rename) = other.rename {
            self.rename = Some(rename)
        }
        if let Some(default) = other.default {
            self.default = Some(default)
        }
        if let Some(missing_field_error) = other.missing_field_error {
            self.missing_field_error = Some(missing_field_error)
        }
    }
}

impl syn::parse::Parse for FieldAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut this = FieldAttributes::default();
        // parse starting right after #[jayson .... ]
        // so first get the content inside the parentheses

        let content;
        let _ = parenthesized!(content in input);
        let input = content;
        // consumed input: #[jayson( .... )]

        loop {
            let attr_name = input.parse::<Ident>()?;
            // consumed input: #[jayson( ... attr_name ... )]
            match attr_name.to_string().as_str() {
                "rename" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let ident = input.parse::<LitStr>()?;
                    // #[jayson( ... rename = ident )]
                    this.rename = Some(ident);
                }
                "default" => {
                    if input.peek(Token![=]) {
                        let _eq = input.parse::<Token![=]>()?;
                        let expr = input.parse::<Expr>()?;
                        // #[jayson( ... default = expr )]
                        this.default = Some(JaysonDefaultFieldAttribute::Function(expr));
                    } else {
                        this.default = Some(JaysonDefaultFieldAttribute::DefaultTrait);
                    }
                }
                "missing_field_error" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let expr = input.parse::<Expr>()?;
                    // #[jayson( ... missing_field_error = expr )]
                    this.missing_field_error = Some(expr);
                }
                _ => {
                    let message = format!("Unknown jayson attribute: {}", attr_name);
                    return Result::Err(syn::Error::new_spanned(attr_name, message));
                }
            }

            if input.peek(Token![,]) {
                let _comma = input.parse::<Token![,]>()?;
                if input.is_empty() {
                    break;
                }
                continue;
            } else if input.is_empty() {
                break;
            } else {
                // TODO: error message here
                break;
            }
        }
        Ok(this)
    }
}

pub fn read_jayson_field_attributes(
    attributes: &[Attribute],
) -> Result<FieldAttributes, syn::Error> {
    let mut this = FieldAttributes::default();
    for attribute in attributes {
        if let Some(ident) = attribute.path.get_ident() {
            if ident != "jayson" {
                continue;
            }
            let other = parse2::<FieldAttributes>(attribute.tokens.clone())?;
            this.overwrite(other);
        } else {
            continue;
        }
    }
    Ok(this)
}

#[derive(Debug)]
pub enum RenameAll {
    CamelCase,
    LowerCase,
}
#[derive(Debug)]
pub enum TagType {
    Internal(String),
    External,
}
impl Default for TagType {
    fn default() -> Self {
        Self::External
    }
}

#[derive(Debug)]
pub enum DenyUnknownFields {
    DefaultError,
    Function(syn::ExprPath),
}

#[derive(Default, Debug)]
pub struct JaysonDataAttributes {
    pub rename_all: Option<RenameAll>,
    pub err_ty: Option<syn::Type>,
    pub tag: TagType,
    pub deny_unknown_fields: Option<DenyUnknownFields>,
}
impl JaysonDataAttributes {
    fn overwrite(&mut self, other: Self) {
        if let Some(rename) = other.rename_all {
            self.rename_all = Some(rename)
        }
        if let Some(err_ty) = other.err_ty {
            self.err_ty = Some(err_ty)
        }
        if let TagType::Internal(x) = other.tag {
            self.tag = TagType::Internal(x)
        }
        if let Some(x) = other.deny_unknown_fields {
            self.deny_unknown_fields = Some(x)
        }
    }
}
impl syn::parse::Parse for JaysonDataAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut this = JaysonDataAttributes::default();
        // parse starting right after #[jayson .... ]
        // so first get the content inside the parentheses

        let content;
        let _ = parenthesized!(content in input);
        let input = content;
        // consumed input: #[jayson( .... )]

        loop {
            let attr_name = input.parse::<Ident>()?;
            // consumed input: #[jayson( ... attr_name ... )]
            match attr_name.to_string().as_str() {
                "rename_all" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let ident = input.parse::<Ident>()?;
                    // #[jayson( ... rename_all = ident )]
                    let rename_all = match ident.to_string().as_str() {
                        "camelCase" => RenameAll::CamelCase,
                        "lowercase" => RenameAll::LowerCase,
                        _ => {
                            todo!("return good error message")
                        }
                    };
                    this.rename_all = Some(rename_all);
                }
                "tag" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let lit = input.parse::<LitStr>()?;
                    // #[jayson( ... tag = "lit" )]
                    this.tag = TagType::Internal(lit.value());
                }
                "error" => {
                    let _eq = input.parse::<Token![=]>()?;
                    let err_ty = input.parse::<syn::Type>()?;
                    // #[jayson( ... error = err_ty )]
                    this.err_ty = Some(err_ty);
                }
                "deny_unknown_fields" => {
                    if input.peek(Token![=]) {
                        let _eq = input.parse::<Token![=]>()?;
                        let func = input.parse::<ExprPath>()?;
                        // #[jayson( ... deny_unknown_fields = func )]
                        this.deny_unknown_fields = Some(DenyUnknownFields::Function(func));
                    } else {
                        this.deny_unknown_fields = Some(DenyUnknownFields::DefaultError);
                    }
                }
                _ => {
                    let message = format!("Unknown jayson attribute: {}", attr_name);
                    return Result::Err(syn::Error::new_spanned(attr_name, message));
                }
            }

            if input.peek(Token![,]) {
                let _comma = input.parse::<Token![,]>()?;
                if input.is_empty() {
                    break;
                }
                continue;
            } else if input.is_empty() {
                break;
            } else {
                // TODO: error message here
                break;
            }
        }
        Ok(this)
    }
}

pub fn read_jayson_data_attributes(
    attributes: &[Attribute],
) -> Result<JaysonDataAttributes, syn::Error> {
    let mut this = JaysonDataAttributes::default();
    for attribute in attributes {
        if let Some(ident) = attribute.path.get_ident() {
            if ident != "jayson" {
                continue;
            }
            let other = parse2::<JaysonDataAttributes>(attribute.tokens.clone())?;
            this.overwrite(other);
        } else {
            continue;
        }
    }
    Ok(this)
}
