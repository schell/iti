//! Example: Login form using the Form derive macro.

use iti_derive::Form;

/// A simple login form with email and password fields.
#[allow(dead_code)]
#[derive(Form)]
pub struct LoginForm {
    #[form(label = "Email Address")]
    email: String,

    #[form(label = "Password")]
    password: String,

    #[form(label = "Remember me")]
    remember_me: bool,
}

fn main() {
    println!("This is a compile-time test of the Form derive macro.");
    println!("In a real app, you would use this in a mogwai application.");

    // The macro generates:
    // - LoginFormComponent<V> struct with form fields
    // - Default implementation
    // - Methods for data collection
}
