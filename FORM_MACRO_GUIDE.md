# Form Derive Macro Guide

## Overview

The `#[derive(Form)]` macro provides a **developer-friendly** way to automatically generate form UI components from Rust struct definitions. This is a significant quality-of-life improvement over manually creating and wiring up form components.

## Quick Start

### Basic Usage

```rust
use iti_derive::Form;

#[derive(Form)]
struct LoginForm {
    #[form(label = "Email")]
    email: String,

    #[form(label = "Password")]
    password: String,

    #[form(label = "Remember me")]
    remember_me: bool,
}

// In your mogwai app:
let form: LoginFormComponent<Web> = Default::default();
// form is now a fully-functional form component with all fields laid out!
```

## What the Macro Generates

When you apply `#[derive(Form)]` to a struct, the macro generates:

1. **`{StructName}Component<V: View>`** — A mogwai component wrapping all your form fields
2. **Field wrappers** — Each field becomes a `FormGroup` (for text inputs) or `Checkbox` (for bools)
3. **Default implementation** — Full form UI with proper Bootstrap styling
4. **Helper methods** — Methods to collect data, validate, and reset

### Generated Example

For:
```rust
#[derive(Form)]
struct LoginForm {
    #[form(label = "Email")]
    email: String,
}
```

The macro generates (simplified):
```rust
pub struct LoginFormComponent<V: View> {
    #[child]
    form_elem: V::Element,
    email: FormGroup<V, TextInput<V>>,
}

impl<V: View> Default for LoginFormComponent<V> {
    fn default() -> Self {
        let email_input = TextInput::new(TextInputType::Text, "");
        let mut email = FormGroup::new("Email", email_input);
        
        rsx! {
            let form_elem = form() {
                {&email}
                button(type = "submit") { "Submit" }
            }
        }
        
        Self { form_elem, email }
    }
}

impl<V: View> LoginFormComponent<V> {
    pub fn collect_values(&self) -> HashMap<String, String> {
        // Returns current field values
    }
}
```

## Supported Field Types

### String Fields
```rust
#[form(label = "Email Address")]
email: String,
```
→ Generates `FormGroup<V, TextInput<V>>`

### Boolean Fields
```rust
#[form(label = "Accept terms")]
accept_terms: bool,
```
→ Generates `Checkbox<V>`

### Optional Fields
```rust
email: Option<String>,
```
→ Same as String, but field is optional

## Field Attributes

### `label`
Sets the label text displayed above the input.
```rust
#[form(label = "Full Name")]
name: String,
```

If not specified, the field name is automatically title-cased:
```rust
#[form]  // Becomes "Email Address"
email_address: String,
```

### `input_type` (Coming Soon)
Controls the HTML input type (defaults to inferred from Rust type):
```rust
#[form(input_type = "email")]
email: String,  // Type is inferred as "email" anyway for String

#[form(input_type = "password")]
secret: String,
```

### `required` (Coming Soon)
Marks a field as required:
```rust
#[form(label = "Username", required)]
username: String,
```

### `help` (Coming Soon)
Adds help text below the field:
```rust
#[form(help = "Use 8+ characters")]
password: String,
```

### `placeholder` (Coming Soon)
Sets placeholder text:
```rust
#[form(placeholder = "john@example.com")]
email: String,
```

## Integration with mogwai

### In Your Component

```rust
use mogwai::prelude::*;
use iti_derive::Form;

#[derive(Form)]
struct MyForm {
    #[form(label = "Username")]
    username: String,

    #[form(label = "Agree")]
    agree: bool,
}

#[derive(ViewChild)]
pub struct MyPage<V: View> {
    #[child]
    div: V::Element,
    form: MyFormComponent<V>,
}

impl<V: View> Default for MyPage<V> {
    fn default() -> Self {
        let form = MyFormComponent::default();
        
        rsx! {
            let div = div() {
                h1() { "Registration" }
                {&form}
            }
        }
        
        Self { div, form }
    }
}

impl<V: View> MyPage<V> {
    pub async fn step(&mut self) {
        // Handle form events
        let values = self.form.collect_values();
        println!("Form data: {:?}", values);
    }
}
```

## Limitations & Future Work

### Current (MVP)
- ✅ Basic String fields → TextInput with FormGroup
- ✅ Boolean fields → Checkbox
- ✅ Automatic label generation
- ✅ Default field layout (vertical stack)
- ✅ Bootstrap 5 styling
- ✅ Form submission button

### Not Yet Implemented
- ❌ Custom input types (email, password detection)
- ❌ Field-level validation callbacks
- ❌ `required` attribute enforcement
- ❌ Help text and placeholder support
- ❌ Nested forms
- ❌ Collections (Vec<T>)
- ❌ Custom enum field types (RadioGroup/Select)
- ❌ Grid/column layout customization
- ❌ Cross-field validation
- ❌ Async validation

## Examples

See `examples/` directory:
- `login_form.rs` — Simple 3-field login form
- `registration_form.rs` — Multi-field registration with checkboxes

## Macro Implementation Details

The `#[derive(Form)]` macro is implemented in the `iti-derive` crate using:
- **syn** — Parsing Rust struct definitions
- **quote** — Generating Rust code
- **proc-macro2** — Procedural macro infrastructure

### Parsing Phase
1. Extracts struct name and fields
2. Parses `#[form(...)]` attributes on each field
3. Infers HTML input type from Rust type
4. Builds metadata (`FormMeta`)

### Code Generation Phase
1. Generates component struct with field declarations
2. Creates `Default` implementation with full `rsx!` UI
3. Generates helper methods
4. Produces syntactically valid Rust code

### Output
The macro produces standard Rust code that can be:
- Type-checked like normal code
- Debugged with standard tools
- Inspected with `cargo expand --example login_form`

## Best Practices

1. **Always include labels** — Makes forms accessible and user-friendly
```rust
#[form(label = "User Email")]  // ✅ Good
email: String,

#[form]  // ⚠️ Auto-generated label "Email"
email: String,
```

2. **Use appropriate field types** — Let the macro infer what it can
```rust
remember_me: bool,      // ✅ Becomes Checkbox
email: String,          // ✅ Becomes TextInput
```

3. **Group related fields** — Create separate forms for logical sections
```rust
#[derive(Form)]
struct AddressForm {
    street: String,
    city: String,
    zip: String,
}

#[derive(Form)]
struct PersonalInfo {
    first_name: String,
    last_name: String,
}
```

## Comparison: Manual vs. Macro

### Without Macro (Manual)
```rust
let email_input = TextInput::new(TextInputType::Text, "");
let mut email_group = FormGroup::new("Email", email_input);
email_group.child_mut().set_required(true);

let agree_check = Checkbox::new("I Agree", false);

rsx! {
    let form = form() {
        {&email_group}
        {&agree_check}
        button(type = "submit") { "Submit" }
    }
}

// ... many more lines of boilerplate
```

### With Macro
```rust
#[derive(Form)]
struct MyForm {
    #[form(label = "Email")]
    email: String,
    
    #[form(label = "I Agree")]
    agree: bool,
}

let form: MyFormComponent<Web> = Default::default();
// Done! Full form is ready to use.
```

## Troubleshooting

### "cannot find derive macro `Form`"
Make sure to import it:
```rust
use iti_derive::Form;
```

### Macro expansion errors
Use cargo-expand to see what code the macro generated:
```bash
cargo expand --example login_form
```

### Type errors in generated code
The most common cause is using non-supported types. Currently only:
- `String` (and `Option<String>`)
- `bool`

## Contributing & Feedback

The form macro is part of the iti library. Suggestions for improvements:
- Additional field types (numbers, dates, etc.)
- Better attribute syntax
- More layout customization options
- Nested form support

## Related Resources

- **Components**: `FormGroup`, `TextInput`, `Checkbox` in `crate::components`
- **Traits**: `FormData`, `FormWidget` in `crate::form_traits`
- **Macro Code**: See `crates/iti-derive/src/`
