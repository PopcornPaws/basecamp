use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, FieldsNamed, Type, TypePath};

#[proc_macro_derive(QueryBuilder)]
pub fn derive_query_builder(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    /*
     */

    // For demonstration, we'll just return an empty TokenStream
    TokenStream::new()
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

fn extract_names_and_types(fields: FieldsNamed) -> Vec<(String, String)> {
    // Collect field names and their inner types
    let mut field_names_and_types = Vec::new();
    for field in fields.named {
        let Some(field_name) = field.ident else {
            unreachable!()
        };

        // Check if the field type is Option<T>
        if let Type::Path(TypePath { path, .. }) = &field.ty {
            if let Some(segment) = path.segments.first() {
                if segment.ident == "Option" {
                    // Extract the inner type T from Option<T>
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            field_types.push((field_name.clone(), inner_type.clone()));
                            continue;
                        }
                    }
                }
            }
        }

        // If the field is not Option<T>, emit an error
        return syn::Error::new_spanned(
            &field.ty,
            format!("Field '{}' is not of type Option<T>", field_name),
        )
        .to_compile_error()
        .into();
    }
    vec![]
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::{parse_quote, ItemStruct};

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
    fn test_extract_names_and_types() {
        let input: DeriveInput = parse_quote! {
            pub struct Foo {
                bar: Option<u8>,
                baz: Option<String>,
            }
        };

        let fields = extract_named_fields(input).unwrap();
        assert_eq!(extract_names_and_types(fields), vec![]);
    }
}
