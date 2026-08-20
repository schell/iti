//! Example: Registration form using the Form derive macro.
//!
//! Demonstrates the #[derive(Form)] macro with multiple field types and customizations.

use iti_derive::Form;

/// A registration form with various field types.
#[allow(dead_code)]
#[derive(Form)]
pub struct RegistrationForm {
    #[form(label = "Full Name")]
    full_name: String,

    #[form(label = "Email Address")]
    email: String,

    #[form(label = "Password")]
    password: String,

    #[form(label = "Confirm Password")]
    confirm_password: String,

    #[form(label = "I agree to the terms and conditions")]
    agree_to_terms: bool,

    #[form(label = "Subscribe to our newsletter")]
    subscribe: bool,
}

/// Another form showcasing optional fields.
#[allow(dead_code)]
#[derive(Form)]
pub struct ContactForm {
    #[form(label = "Name")]
    name: String,

    #[form(label = "Email")]
    email: String,

    #[form(label = "Message")]
    message: String,

    #[form(label = "Phone Number (optional)")]
    phone: String,
}

fn main() {
    println!("Form derivation examples:");
    println!("1. RegistrationForm - multi-field form with checkboxes");
    println!("2. ContactForm - simple contact form");
    println!("\nIn a real mogwai application, you would:");
    println!("- Create these forms with Form::new() or Default::default()");
    println!("- Drive the event loop with form.step().await");
    println!("- Collect form data with form.collect_values()");
}
