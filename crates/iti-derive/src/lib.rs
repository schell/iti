//! Procedural macros for iti form components.
//!
//! This crate provides the `#[derive(Form)]` macro for automatically generating
//! form UI components from Rust structs.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod generate;
mod parse;

/// Derive macro for automatically generating form components from struct definitions.
///
/// # Example
///
/// ```ignore
/// #[derive(Form)]
/// struct LoginForm {
///     #[form(label = "Email", input_type = "email", required)]
///     email: String,
///
///     #[form(label = "Password", input_type = "password", required)]
///     password: String,
/// }
/// ```
#[proc_macro_derive(Form, attributes(form))]
pub fn derive_form(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Parse the struct and its attributes
    match parse::parse_form_struct(&input) {
        Ok(form_meta) => {
            // Generate the form component code
            match generate::generate_form_component(&form_meta) {
                Ok(expanded) => expanded.into(),
                Err(e) => e.to_compile_error().into(),
            }
        }
        Err(e) => e.to_compile_error().into(),
    }
}
