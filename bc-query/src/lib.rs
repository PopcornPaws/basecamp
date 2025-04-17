#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![warn(unused_crate_dependencies)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Error, Fields, FieldsNamed, GenericArgument, Ident, PathArguments,
    PathSegment, Type, TypePath, parse_macro_input,
};

#[proc_macro_derive(QueryBuilder)]
pub fn derive_query_builder(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident.clone();
    let named_fields = match extract_named_fields(input) {
        Ok(extracted) => extracted,
        Err(error) => return error.to_compile_error().into(),
    };
    let names_and_types = match extract_names_and_types(named_fields) {
        Ok(extracted) => extracted,
        Err(error) => return error.to_compile_error().into(),
    };

    let with_method_impls = names_and_types
        .iter()
        .map(|(name, typ)| {
            let method_name = format_ident!("with_{}", name);
            quote! {
                pub fn #method_name(mut self, #name: #typ) -> Self {
                    self.#name = Some(#name);
                    self
                }
            }
        })
        .collect::<Vec<TokenStream2>>();

    let push_to_query_impls = names_and_types
        .iter()
        .map(|(name, _)| {
            let name_str = name.to_string();
            quote! {
                if let Some(ref #name) = self.#name {
                    query_str_vec.push(format!("{}={}", #name_str, #name));
                }
            }
        })
        .collect::<Vec<TokenStream2>>();

    quote! {
        impl #struct_name {
            pub fn new() -> Self {
                Self::default()
            }

            #(#with_method_impls)*

            pub fn append_to(&self, base: &mut String) {
                let mut query_str_vec = Vec::new();
                #(#push_to_query_impls)*;
                let joined: String = query_str_vec.join("&");
                if !joined.is_empty() {
                    use std::fmt::Write;
                    write!(base, "?{}", joined).unwrap_or(())
                }
            }
        }
    }
    .into()
}

fn extract_named_fields(input: DeriveInput) -> Result<FieldsNamed, Error> {
    // Proceed only if the input is a struct
    if let Data::Struct(data_struct) = input.data {
        if let Fields::Named(fields_named) = data_struct.fields {
            Ok(fields_named)
        } else {
            Err(Error::new_spanned(
                &input.ident,
                "QueryBuilder can only be derived for structs with named fields",
            ))
        }
    } else {
        Err(Error::new_spanned(
            &input.ident,
            "QueryBuilder can only be derived for structs",
        ))
    }
}

fn extract_names_and_types(fields: FieldsNamed) -> Result<Vec<(Ident, Ident)>, Error> {
    // Collect field names and their inner types
    let mut field_names_and_types = Vec::new();
    for field in fields.named {
        let Some(field_name) = field.ident else {
            unreachable!()
        };

        let not_option_error = Err(Error::new_spanned(
            &field.ty,
            format!("Field '{}' is not of type Option<T>", field_name),
        ));

        // Check if the field type is Option<T>
        let segment = extract_path_segment(&field.ty)?;
        if segment.ident != "Option" {
            return not_option_error;
        }

        // Extract the inner type T from Option<T>
        let PathArguments::AngleBracketed(args) = &segment.arguments else {
            return not_option_error;
        };

        let Some(GenericArgument::Type(inner_type)) = args.args.first() else {
            return not_option_error;
        };

        let segment = extract_path_segment(inner_type)?;

        field_names_and_types.push((field_name, segment.ident.clone()));
    }

    Ok(field_names_and_types)
}

fn extract_path_segment(typ: &Type) -> Result<&PathSegment, Error> {
    let invalid_path_error = Err(Error::new_spanned(typ, "expected path segment type"));
    let Type::Path(TypePath { path, .. }) = typ else {
        return invalid_path_error;
    };

    let Some(segment) = path.segments.first() else {
        return invalid_path_error;
    };
    Ok(segment)
}

#[cfg(test)]
mod test {
    use super::*;
    use proc_macro2::Span;
    use syn::parse_quote;

    #[test]
    fn cannot_derive_for_enum() {
        let input: DeriveInput = parse_quote! {
            pub enum Foo {
                Bar,
                Baz,
            }
        };

        assert!(extract_named_fields(input).is_err());
    }

    #[test]
    fn cannot_derive_for_struct_with_unnamed_fields() {
        let input: DeriveInput = parse_quote! {
            pub struct Foo(u8, bool);
        };

        assert!(extract_named_fields(input).is_err());
    }

    #[test]
    fn names_and_types_extraction() {
        let input: DeriveInput = parse_quote! {
            pub struct Foo {
                bar: Option<u8>,
                baz: Option<String>,
            }
        };

        let fields = extract_named_fields(input).unwrap();
        let names_and_types = extract_names_and_types(fields).unwrap();

        let (bar_name, bar_type) = &names_and_types[0];
        let (baz_name, baz_type) = &names_and_types[1];
        assert_eq!(bar_name, &Ident::new("bar", Span::call_site()));
        assert_eq!(bar_type, &Ident::new("u8", Span::call_site()));
        assert_eq!(baz_name, &Ident::new("baz", Span::call_site()));
        assert_eq!(baz_type, &Ident::new("String", Span::call_site()));
    }
}
