#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, GenericArgument, Ident, Lit, Meta, NestedMeta,
    PathArguments, PathSegment, Type, TypePath, Visibility,
};

const MACRO: &str = "Record";

struct RecordField<'a> {
    field: &'a Field,
    should_flatten: bool,
}

impl RecordField<'_> {
    fn name(&self) -> &Ident {
        self.field.ident.as_ref().unwrap()
    }

    fn ty(&self) -> &Type {
        &self.field.ty
    }

    fn visibility(&self) -> &Visibility {
        &self.field.vis
    }

    fn type_path(&self) -> Option<&TypePath> {
        if let Type::Path(type_path) = self.ty() {
            Some(type_path)
        } else {
            None
        }
    }

    fn inner_vec_type(&self) -> Option<&Ident> {
        self.type_path().and_then(inner_vec_type)
    }
}

impl<'a> From<&'a Field> for RecordField<'a> {
    fn from(field: &'a Field) -> Self {
        Self {
            field,
            should_flatten: should_flatten(&field.attrs),
        }
    }
}

fn should_flatten(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .find(|attr| attr.path.is_ident("record"))
        .is_some_and(|attr| has_attribute(attr, "flatten"))
}

fn has_attribute(attribute: &Attribute, ident: &str) -> bool {
    if let Meta::List(meta_list) = attribute.parse_meta().unwrap() {
        for nested_meta in meta_list.nested {
            if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
                return path.is_ident(ident);
            }
        }
    }
    false
}

fn first_path_segment(type_path: &TypePath) -> Option<&PathSegment> {
    type_path.path.segments.first()
}

fn inner_vec_type(type_path: &TypePath) -> Option<&Ident> {
    first_path_segment(type_path)
        .filter(|segment| segment.ident == "Vec")
        .map(|segment| &segment.arguments)
        .and_then(|arg| {
            if let PathArguments::AngleBracketed(data) = arg {
                data.args.first()
            } else {
                None
            }
        })
        .and_then(|arg| {
            if let GenericArgument::Type(Type::Path(path)) = arg {
                Some(path)
            } else {
                None
            }
        })
        .and_then(first_path_segment)
        .map(|segment| &segment.ident)
}

fn batch_ident(name: &Ident) -> Ident {
    Ident::new(&format!("Batch{name}"), name.span());
}

#[derive(Default)]
struct BatchBuilder {
    batch_fields: Vec<TokenStream2>,
    push_fields: Vec<TokenStream2>,
    flattened_names: Vec<Ident>,
    non_flattened_names: Vec<Ident>,
    non_flattened_types: Vec<Type>,
}

impl BatchBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, field: &RecordField) {
        if let Some(inner_type) = field.inner_vec_type() {
            if field.should_flatten {
                let field_type = batch_ident(inner_type);
                self.push_flattened(field.visibility(), field.name(), &field_type);
            } else {
                self.push_non_flattened(field.visibility(), field.name(), field.ty());
            }
        } else {
            self.push_non_flattened(field.visibility(), field.name(), field.ty());
        }
    }

    fn push_flattened(&mut self, vis: &Visibility, field_name: &Ident, field_type: &Ident) {
        self.batch_fields.push(quote! {
            #vis #field_name: #field_type
        });
        self.push_fields.push(quote! {
            self.#field_name.extend(record.#field_name);
        });
        self.flattened_names.push(field_name.clone());
    }

    fn push_non_flattened(&mut self, vis: &Visibility, field_name: &Ident, field_type: &Type) {
        self.batch_fields.push(quote! {
            #vis #field_name: Vec<#field_type>
        });
        self.push_fields.push(quote! {
            self.#field_name.push(record.#field_name);
        });
        self.non_flattened_names.push(field_name.clone());
        self.non_flattened_types.push(field_type.clone());
    }

    fn join_non_flattened_names(&self) -> String {
        self.non_flattened_names
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(",")
    }

    fn expand(self) -> TokenStream {
        let BatchBuilder {
            batch_fields,
            push_fields,
            flattened_names,
            non_flattened_names,
            non_flattened_types,
        } = self;
        todo!();
    }
}

/// # Panics
///
/// Panics if 
/// - derived for non-structs (enum, union)
/// - derived for struct with unnamed fields
/// - `table` is missing from the struct attributes
#[proc_macro_derive(Record, attributes(record, table))]
pub fn impl_record(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let batch_name = batch_ident(name);

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
    let mut table_name = String::new();
    for attr in input.attrs {
        if attr.path.is_ident("table") {
            if let Ok(Meta::NameValue(meta_name_value)) = attr.parse_meta() {
                if let Lit::Str(lit_str) = meta_name_value.lit {
                    table_name = lit_str.value();
                }
            }
        }
    }

    assert!(!table_name.is_empty(), "missing table name\nadd '#[table = \"my_example_table_name\"]' as an attribute to the struct");

    /*
    let mut multi_fields = Vec::new();
    let mut push_fields = Vec::new();
    let mut flattened_names = Vec::new();
    let mut non_flattened_names = Vec::new();
    let mut non_flattened_types = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let mut should_flatten = false;

        // iterate over field attributes to check whether it should be flattened
        for attr in &field.attrs {
            if attr.path.is_ident("record") {
                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    for nested_meta in meta_list.nested {
                        if let NestedMeta::Meta(Meta::Path(path)) = nested_meta {
                            if path.is_ident("flatten") {
                                should_flatten = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        // if we encounter a field of type Vec<T> check whether it should be flattened
        if let Type::Path(type_path) = field_type {
            if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(ref data) =
                    type_path.path.segments[0].arguments
                {
                    if let Some(syn::GenericArgument::Type(Type::Path(inner_type_path))) =
                        data.args.first()
                    {
                        let inner_type = &inner_type_path.path.segments[0].ident;

                        if should_flatten {
                            let multi_inner_type =
                                Ident::new(&format!("Multi{}", inner_type), inner_type.span());
                            multi_fields.push(quote! {
                                #field_name: #multi_inner_type
                            });
                            push_fields.push(quote! {
                                self.#field_name.extend(record.#field_name);
                            });
                            flattened_names.push(quote! {
                                #field_name
                            });
                        } else {
                            multi_fields.push(quote! {
                                #field_name: Vec<Vec<#inner_type>>
                            });
                            push_fields.push(quote! {
                                self.#field_name.push(record.#field_name);
                            });
                            non_flattened_names.push(quote! {
                                #field_name
                            });
                            non_flattened_types.push(field_type);
                        }
                        continue;
                    }
                }
            }
        }

        multi_fields.push(quote! {
            pub #field_name: Vec<#field_type>
        });
        push_fields.push(quote! {
            self.#field_name.push(record.#field_name);
        });
        non_flattened_names.push(quote! {
            #field_name
        });
        non_flattened_types.push(field_type);
    }

    let non_flattened_names_string = non_flattened_names
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<String>>()
        .join(",");
    */

    let expanded = quote! {
        /*
        impl Record for #single_name {
            type Multi = #multi_name;
        }

        #[derive(Clone, Debug, Default)]
        pub struct #multi_name {
            #(#multi_fields,)*
        }

        impl #multi_name {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn push(&mut self, record: #single_name) {
                #(#push_fields)*
            }

            pub fn extend(&mut self, records: Vec<#single_name>) {
                records.into_iter().for_each(|record| self.push(record));
            }

            pub fn raw_insert_query() -> &'static str {
                use sqlx::TypeInfo;
                let values = vec![
                    #(<#non_flattened_types as sqlx::Type<sqlx::Postgres>>::type_info().name().to_string(),)*
                ];
                let values_string = values
                    .into_iter()
                    .enumerate()
                    .map(|(i, val)| format!("${}::{}[]", i + 1, val))
                    .collect::<Vec<String>>()
                    .join(",");
                Box::leak(Box::new(format!("INSERT INTO {} ({}) SELECT * FROM UNNEST({})",#table_name, #non_flattened_names_string, values_string)))
            }

            pub fn insert_query<'a>(&'a self) -> sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments> {
                sqlx::query(Self::raw_insert_query())
                    #(.bind(&self.#non_flattened_names))*
            }

            pub async fn insert_tx(self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<(), sqlx::Error> {
                self.insert_query().execute(tx.as_mut()).await?;
                #(self.#flattened_names.insert_query().execute(tx.as_mut()).await?;)*
                Ok(())
            }

            pub async fn insert(self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
                let mut tx = pool.begin().await?;
                self.insert_tx(&mut tx).await?;
                tx.commit().await
            }
        }

        impl From<Vec<#single_name>> for #multi_name {
            fn from(records: Vec<#single_name>) -> Self {
                records.into_iter().fold(Self::default(), |mut multi, record| {
                    multi.push(record);
                    multi
                })
            }
        }

    */
    };
    expanded.into()
}
