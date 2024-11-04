#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
extern crate proc_macro;

mod builder;
mod field;
mod utils;

use builder::Builder;
use field::Field;
use utils::extract_attribute_value;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Fields};

const MACRO: &str = "Record";

/// # Panics
///
/// Panics if
/// - derived for non-structs (enum, union)
/// - derived for struct with unnamed fields
/// - `table` is missing from the struct attributes
#[proc_macro_derive(Record, attributes(record))]
pub fn impl_record(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let struct_type = input.ident;
    //let batch_name = batch_ident(name);

    let fields = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            fields_named
        } else {
            panic!("{MACRO} supports named fields only")
        }
    } else {
        panic!("{MACRO} supports structs only")
    };

    // Extract the struct-level attribute (table name)
    let table_name = extract_attribute_value(&input.attrs, "record", "table").unwrap_or_default();
    assert!(!table_name.is_empty(), "missing table name\nadd '#[table = \"my_example_table_name\"]' as an attribute to the struct");

    let builder =
        fields
            .named
            .iter()
            .map(Field::from)
            .fold(Builder::new(), |mut builder, field| {
                builder.push(&field);
                builder
            });

    builder.expand(&struct_type, &table_name).into()
}
