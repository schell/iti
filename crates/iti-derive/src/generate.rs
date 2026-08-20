//! Code generation for form components.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Result as SynResult;

use crate::parse::FormMeta;

/// Generate the form component code from struct metadata.
pub fn generate_form_component(meta: &FormMeta) -> SynResult<TokenStream> {
    let struct_name = &meta.struct_name;
    let struct_ident = &meta.struct_ident;
    let component_name = format!("{}Component", struct_name);
    let component_ident = syn::Ident::new(&component_name, proc_macro2::Span::call_site());

    // Generate field declarations for the component struct
    let field_decls = generate_field_declarations(meta);

    // Generate field initialization code
    let field_inits_code = generate_field_initialization_code(meta);

    // Generate form rendering
    let form_render = generate_form_render(meta);

    // Generate the field names for the Self construction
    let field_names = meta
        .fields
        .iter()
        .map(|field| syn::Ident::new(&field.name, proc_macro2::Span::call_site()));

    // Generate the StepMut impl body
    let step_mut_body = generate_step_mut_body(meta);

    // Generate the try_value method body
    let try_value_body = generate_try_value_body(meta);

    let expanded = quote! {
        /// Auto-generated form component from #[derive(Form)].
        #[derive(mogwai::prelude::ViewChild)]
        pub struct #component_ident<V: mogwai::prelude::View> {
            #[child]
            form_elem: V::Element,
            #field_decls
        }

        impl<V: mogwai::prelude::View> Default for #component_ident<V> {
            fn default() -> Self {
                use mogwai::prelude::*;

                #field_inits_code

                rsx! {
                    #form_render
                }

                Self {
                    form_elem,
                    #(#field_names),*
                }
            }
        }

        impl ::iti::form_traits::Form for #struct_ident {
            type Component<V: mogwai::prelude::View> = #component_ident<V>;
        }

        impl<V: mogwai::prelude::View> ::iti::form_traits::FormComponent<V>
            for #component_ident<V>
        {
            type Data = #struct_ident;

            fn try_value(&self) -> ::std::result::Result<Self::Data, ::std::vec::Vec<::iti::form_traits::FormError>> {
                #try_value_body
            }
        }

        impl<V: mogwai::prelude::View> mogwai::step::StepMut for #component_ident<V> {
            type Output = ::iti::form_traits::FormEvent;

            fn step_mut(&mut self) -> impl std::future::Future<Output = ::iti::form_traits::FormEvent> {
                async move {
                    #step_mut_body
                }
            }
        }
    };

    Ok(expanded)
}

/// Generate struct field declarations for the component.
fn generate_field_declarations(meta: &FormMeta) -> TokenStream {
    let fields = meta.fields.iter().map(|field| {
        let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());

        match field.input_type.as_deref() {
            Some("checkbox") => {
                quote! {
                    #field_name: ::iti::components::Checkbox<V>,
                }
            }
            Some("textarea") => {
                quote! {
                    #field_name: ::iti::components::FormGroup<V, ::iti::components::Textarea<V>>,
                }
            }
            _ => {
                quote! {
                    #field_name: ::iti::components::FormGroup<V, ::iti::components::TextInput<V>>,
                }
            }
        }
    });

    quote! {
        #(#fields)*
    }
}

/// Map an input_type string to the corresponding TextInputType variant.
fn input_type_variant(input_type: &str) -> TokenStream {
    match input_type {
        "email" => quote! { ::iti::components::text_input::TextInputType::Email },
        "password" => quote! { ::iti::components::text_input::TextInputType::Password },
        "tel" => quote! { ::iti::components::text_input::TextInputType::Tel },
        "url" => quote! { ::iti::components::text_input::TextInputType::Url },
        "search" => quote! { ::iti::components::text_input::TextInputType::Search },
        _ => quote! { ::iti::components::text_input::TextInputType::Text },
    }
}

/// Generate field initialization code that will be used in both Default impl and Self construction.
fn generate_field_initialization_code(meta: &FormMeta) -> TokenStream {
    meta.fields.iter().map(|field| {
        let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
        let label = field.label.as_ref().cloned().unwrap_or_default();
        let required = field.required;
        let placeholder = field.placeholder.as_ref();
        let help = field.help.as_ref();
        let min_length = field.min_length;
        let max_length = field.max_length;

        // Label placement code
        let placement_code = match field.label_placement.as_deref() {
            Some("above") => Some(quote! {
                #field_name.set_label_placement(::iti::components::form_group::LabelPlacement::Above);
            }),
            Some("below") => Some(quote! {
                #field_name.set_label_placement(::iti::components::form_group::LabelPlacement::Below);
            }),
            Some("inline") => Some(quote! {
                #field_name.set_label_placement(::iti::components::form_group::LabelPlacement::Inline);
            }),
            Some("floating") => Some(quote! {
                #field_name.set_label_placement(::iti::components::form_group::LabelPlacement::Floating);
            }),
            _ => None,
        };

        match field.input_type.as_deref() {
            Some("checkbox") => {
                quote! {
                    let mut #field_name = ::iti::components::Checkbox::new(#label, false);
                }
            }
            Some("textarea") => {
                let placeholder_code = placeholder.map(|p| quote! {
                    #field_name.child_mut().set_placeholder(#p);
                });
                let min_code = min_length.map(|n| quote! {
                    #field_name.child_mut().set_min_length(#n);
                });
                let max_code = max_length.map(|n| quote! {
                    #field_name.child_mut().set_max_length(#n);
                });
                let help_code = help.map(|h| quote! {
                    #field_name.set_help_text(#h);
                });
                let rows_code = quote! {
                    #field_name.child_mut().set_rows(3);
                };
                quote! {
                    let __textarea = ::iti::components::Textarea::new("");
                    let mut #field_name = ::iti::components::FormGroup::new(#label, __textarea);
                    #rows_code
                    #placeholder_code
                    #min_code
                    #max_code
                    #help_code
                    if #required {
                        #field_name.child_mut().set_required(true);
                        #field_name.set_required_indicator(true);
                    }
                    #placement_code
                }
            }
            _ => {
                let input_type = input_type_variant(field.input_type.as_deref().unwrap_or("text"));
                let placeholder_code = placeholder.map(|p| quote! {
                    #field_name.child_mut().set_placeholder(#p);
                });
                let min_code = min_length.map(|n| quote! {
                    #field_name.child_mut().set_min_length(#n);
                });
                let max_code = max_length.map(|n| quote! {
                    #field_name.child_mut().set_max_length(#n);
                });
                let help_code = help.map(|h| quote! {
                    #field_name.set_help_text(#h);
                });
                let input_var = format!("__{}_input", field.name);
                let input_ident = syn::Ident::new(&input_var, proc_macro2::Span::call_site());
                quote! {
                    let #input_ident = ::iti::components::TextInput::new(#input_type, "");
                    let mut #field_name = ::iti::components::FormGroup::new(#label, #input_ident);
                    #placeholder_code
                    #min_code
                    #max_code
                    #help_code
                    if #required {
                        #field_name.child_mut().set_required(true);
                        #field_name.set_required_indicator(true);
                    }
                    #placement_code
                }
            }
        }
    }).collect()
}

/// Generate rsx! rendering code for the form.
///
/// Renders a bare <div> containing only the field components.
/// No submit button — the caller adds buttons alongside the component.
fn generate_form_render(meta: &FormMeta) -> TokenStream {
    let field_renders = meta.fields.iter().map(|field| {
        let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
        quote! { {&#field_name} }
    });

    quote! {
        let form_elem = div() {
            #(#field_renders)*
        }
    }
}

/// Generate the body of the StepMut impl.
///
/// Races all field event futures, maps each to a `FormEvent`, calls
/// `update_validation()` on FormGroup fields, and returns the `FormEvent`.
fn generate_step_mut_body(meta: &FormMeta) -> TokenStream {
    if meta.fields.is_empty() {
        return quote! {
            std::future::pending::<::iti::form_traits::FormEvent>().await
        };
    }

    // For each field, generate a future that returns just the field index.
    // After the race resolves, construct the FormEvent by querying the
    // winning field's current state — avoids borrow conflicts and type
    // mismatches between different field event types.
    let futures: Vec<TokenStream> = meta
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
            let idx = i as u8;

            match field.input_type.as_deref() {
                Some("checkbox") => {
                    quote! {
                        self.#field_name.step_mut().map(|_| #idx)
                    }
                }
                _ => {
                    quote! {
                        self.#field_name.child_mut().step_mut().map(|_| #idx)
                    }
                }
            }
        })
        .collect();

    // Generate the post-race FormEvent construction for each field.
    let event_constructs: Vec<TokenStream> = meta
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
            let name_str = &field.name;
            let idx = i as u8;

            match field.input_type.as_deref() {
                Some("checkbox") => {
                    quote! {
                        #idx => (::iti::form_traits::FormEvent::FieldChanged {
                            field: #name_str.to_string(),
                            value: ::iti::form_traits::FormValue::Bool(
                                self.#field_name.is_checked()
                            ),
                            valid: true,
                        }),
                    }
                }
                Some("textarea") => {
                    quote! {
                        #idx => (::iti::form_traits::FormEvent::FieldChanged {
                            field: #name_str.to_string(),
                            value: ::iti::form_traits::FormValue::String(
                                self.#field_name.child().value()
                            ),
                            valid: self.#field_name.child().is_valid(),
                        }),
                    }
                }
                _ => {
                    // TextInput: we can't distinguish Input vs Blur without the
                    // raw event, so always emit FieldChanged.
                    quote! {
                        #idx => (::iti::form_traits::FormEvent::FieldChanged {
                            field: #name_str.to_string(),
                            value: ::iti::form_traits::FormValue::String(
                                self.#field_name.child().value()
                            ),
                            valid: self.#field_name.child().is_valid(),
                        }),
                    }
                }
            }
        })
        .collect();

    // Generate validation update calls for FormGroup fields.
    let validation_updates: Vec<TokenStream> = meta
        .fields
        .iter()
        .map(|field| {
            let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());

            match field.input_type.as_deref() {
                Some("checkbox") => quote! {},
                _ => quote! {
                    self.#field_name.update_validation();
                },
            }
        })
        .collect();

    // Nest the .or() calls
    let mut or_chain = futures[0].clone();
    for f in &futures[1..] {
        or_chain = quote! { (#or_chain).or(#f) };
    }

    quote! {
        use futures_lite::FutureExt;
        use mogwai::future::MogwaiFutureExt;
        use ::iti::components::Validatable;

        let idx = #or_chain.await;
        #(#validation_updates)*

        match idx {
            #(#event_constructs)*
            _ => ::std::unreachable!("unknown field index"),
        }
    }
}

/// Generate the body of the `try_value` method that constructs the original
/// struct from current field values, collecting all validation errors into
/// a `Vec<FormError>`.
fn generate_try_value_body(meta: &FormMeta) -> TokenStream {
    let struct_ident = &meta.struct_ident;

    // Generate validation checks for required FormGroup fields.
    let checks: Vec<TokenStream> = meta
        .fields
        .iter()
        .map(|field| {
            let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
            let name_str = &field.name;

            match field.input_type.as_deref() {
                Some("checkbox") => quote! {},
                _ => {
                    if field.required {
                        quote! {
                            if self.#field_name.child().value().is_empty() {
                                errors.push(::iti::form_traits::FormError::RequiredFieldEmpty {
                                    field: #name_str.to_string(),
                                });
                            } else if !self.#field_name.child().is_valid() {
                                errors.push(::iti::form_traits::FormError::ValidationFailed {
                                    field: #name_str.to_string(),
                                    message: self.#field_name.child().validation_message()
                                        .unwrap_or_default(),
                                });
                            }
                        }
                    } else {
                        quote! {}
                    }
                }
            }
        })
        .collect();

    // Generate the struct construction expressions for each field.
    let field_constructions: Vec<TokenStream> = meta
        .fields
        .iter()
        .map(|field| {
            let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());

            match field.input_type.as_deref() {
                Some("checkbox") => quote! {
                    #field_name: self.#field_name.is_checked()
                },
                _ => quote! {
                    #field_name: self.#field_name.child().value()
                },
            }
        })
        .collect();

    quote! {
        use ::iti::components::Validatable;

        let mut errors: ::std::vec::Vec<::iti::form_traits::FormError> = ::std::vec::Vec::new();

        #(#checks)*

        if !errors.is_empty() {
            return ::std::result::Result::Err(errors);
        }

        ::std::result::Result::Ok(#struct_ident {
            #(#field_constructions),*
        })
    }
}
