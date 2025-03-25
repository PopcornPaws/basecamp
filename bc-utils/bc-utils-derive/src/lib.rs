#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(SerdeAsString)]
pub fn derive_serde_as_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let serialize_impl = quote! {
        impl serde::Serialize for #name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>{
                serializer.serialize_str(&self.to_string())
            }
        }
    };

    let deserialize_impl = quote! {
        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>{
                let s = <String as serde::Deserialize>::deserialize(deserializer)?;
                s.parse().map_err(serde::de::Error::custom)
            }
        }
    };

    let expanded = quote! {
        #serialize_impl
        #deserialize_impl
    };

    proc_macro::TokenStream::from(expanded)
}
