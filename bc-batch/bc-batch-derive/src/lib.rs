#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// # Panics
///
/// May panic if field name idents cannot be taken by reference.
#[proc_macro_derive(Batch)]
pub fn derive_batch(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let batch_name = format_ident!("{}Batch", struct_name);

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return syn::Error::new_spanned(
                    struct_name,
                    "Batch can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(struct_name, "Batch can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let expanded = quote! {
        pub struct #batch_name {
            #(pub #field_names: Vec<#field_types>,)*
        }

        impl From<Vec<#struct_name>> for #batch_name {
            fn from(items: Vec<#struct_name>) -> Self {
                let mut batch = Self {
                    #(#field_names: Vec::new(),)*
                };

                for item in items {
                    #(batch.#field_names.push(item.#field_names);)*
                }

                batch
            }
        }

        impl std::iter::FromIterator<#struct_name> for #batch_name {
            fn from_iter<I: IntoIterator<Item = #struct_name>>(iter: I) -> Self {
                let mut batch = Self {
                    #(#field_names: Vec::new(),)*
                };

                for item in iter {
                    #(batch.#field_names.push(item.#field_names);)*
                }

                batch
            }
        }
    };

    expanded.into()
}
