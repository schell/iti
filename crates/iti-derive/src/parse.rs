//! Parsing logic for form struct definitions and attributes.

use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Field, Fields, Ident, Result as SynResult, Type};

/// Metadata about a form field extracted from attributes.
pub struct FormFieldMeta {
    pub name: String,
    #[allow(dead_code)]
    pub ty: Type,
    pub label: Option<String>,
    pub input_type: Option<String>,
    pub help: Option<String>,
    pub required: bool,
    pub placeholder: Option<String>,
    pub label_placement: Option<String>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
}

/// Metadata about a form struct.
pub struct FormMeta {
    pub struct_name: String,
    pub struct_ident: Ident,
    pub fields: Vec<FormFieldMeta>,
}

/// Parse a form struct definition and extract field metadata.
pub fn parse_form_struct(input: &DeriveInput) -> SynResult<FormMeta> {
    let struct_name = input.ident.to_string();
    let struct_ident = input.ident.clone();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let mut form_fields = Vec::new();
                for field in &fields.named {
                    form_fields.push(parse_field(field)?);
                }
                form_fields
            }
            _ => {
                return Err(Error::new_spanned(
                    input,
                    "Form derive only works with named struct fields",
                ))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                input,
                "Form derive only works with structs",
            ))
        }
    };

    Ok(FormMeta {
        struct_name,
        struct_ident,
        fields,
    })
}

/// Parse a single field and extract form attributes.
fn parse_field(field: &Field) -> SynResult<FormFieldMeta> {
    let name = field
        .ident
        .as_ref()
        .ok_or_else(|| Error::new_spanned(field, "Field must be named"))?
        .to_string();

    let ty = field.ty.clone();

    // Extract form attributes
    let mut label = None;
    let mut input_type = None;
    let mut help = None;
    let mut required = false;
    let mut placeholder = None;
    let mut label_placement = None;
    let mut min_length = None;
    let mut max_length = None;

    // Parse all #[form(...)] attributes using parse_nested_meta,
    // which handles both key=value pairs and bare flags like `required`.
    for attr in &field.attrs {
        if attr.path().is_ident("form") {
            parse_form_attributes(
                attr,
                &mut label,
                &mut input_type,
                &mut help,
                &mut required,
                &mut placeholder,
                &mut label_placement,
                &mut min_length,
                &mut max_length,
            )?;
        }
    }

    // Default label to field name (title-cased)
    if label.is_none() {
        label = Some(title_case(&name));
    }

    // Infer input type from Rust type if not specified
    if input_type.is_none() {
        input_type = Some(infer_input_type(&ty));
    }

    Ok(FormFieldMeta {
        name,
        ty,
        label,
        input_type,
        help,
        required,
        placeholder,
        label_placement,
        min_length,
        max_length,
    })
}

/// Parse form-specific attributes from the #[form(...)] attribute.
#[allow(clippy::too_many_arguments)]
fn parse_form_attributes(
    attr: &Attribute,
    label: &mut Option<String>,
    input_type: &mut Option<String>,
    help: &mut Option<String>,
    required: &mut bool,
    placeholder: &mut Option<String>,
    label_placement: &mut Option<String>,
    min_length: &mut Option<u32>,
    max_length: &mut Option<u32>,
) -> SynResult<()> {
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("label") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            *label = Some(lit.value());
        } else if meta.path.is_ident("input_type") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            *input_type = Some(lit.value());
        } else if meta.path.is_ident("help") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            *help = Some(lit.value());
        } else if meta.path.is_ident("placeholder") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            *placeholder = Some(lit.value());
        } else if meta.path.is_ident("required") {
            *required = true;
        } else if meta.path.is_ident("label_placement") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            *label_placement = Some(lit.value());
        } else if meta.path.is_ident("min_length") {
            let value = meta.value()?;
            let lit: syn::LitInt = value.parse()?;
            *min_length = Some(lit.base10_parse()?);
        } else if meta.path.is_ident("max_length") {
            let value = meta.value()?;
            let lit: syn::LitInt = value.parse()?;
            *max_length = Some(lit.base10_parse()?);
        }
        Ok(())
    })?;
    Ok(())
}

/// Convert field name to title case for label.
fn title_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Infer input type from Rust type.
fn infer_input_type(ty: &Type) -> String {
    let ty_str = quote!(#ty).to_string();
    let ty_lower = ty_str.to_lowercase();

    if ty_lower.contains("bool") {
        "checkbox".to_string()
    } else if ty_lower.contains("string") {
        "text".to_string()
    } else if ty_lower.contains("u32") || ty_lower.contains("i32") || ty_lower.contains("usize") {
        "number".to_string()
    } else {
        "text".to_string()
    }
}
