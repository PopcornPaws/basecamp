use crate::field::Field;
use crate::utils::batch_ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type, Visibility};

#[derive(Default)]
pub struct Builder {
    pub batch_fields: Vec<TokenStream>,
    pub push_fields: Vec<TokenStream>,
    pub flattened_names: Vec<Ident>,
    pub non_flattened_names: Vec<Ident>,
    pub non_flattened_types: Vec<Type>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, field: &Field) {
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

    pub fn push_flattened(&mut self, vis: &Visibility, field_name: &Ident, field_type: &Ident) {
        self.batch_fields.push(quote! {
            #vis #field_name: #field_type
        });
        self.push_fields.push(quote! {
            self.#field_name.extend(record.#field_name);
        });
        self.flattened_names.push(field_name.clone());
    }

    pub fn push_non_flattened(&mut self, vis: &Visibility, field_name: &Ident, field_type: &Type) {
        self.batch_fields.push(quote! {
            #vis #field_name: Vec<#field_type>
        });
        self.push_fields.push(quote! {
            self.#field_name.push(record.#field_name);
        });
        self.non_flattened_names.push(field_name.clone());
        self.non_flattened_types.push(field_type.clone());
    }

    pub fn join_non_flattened_names(&self) -> String {
        self.non_flattened_names
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(",")
    }

    pub fn expand(self, struct_type: &Ident, table_name: &str) -> TokenStream {
        let non_flattened_names_string = self.join_non_flattened_names();

        let Self {
            batch_fields,
            push_fields,
            flattened_names,
            non_flattened_names,
            non_flattened_types,
        } = self;

        let batch_type = batch_ident(struct_type);

        quote! {
            impl Record for #struct_type {
                type Batch = #batch_type;
            }

            #[derive(Clone, Debug, Default)]
            pub struct #batch_type {
                #(#batch_fields,)*
            }

            impl #batch_type {
                pub fn new() -> Self {
                    Self::default()
                }

                pub fn push(&mut self, record: #struct_type) {
                    #(#push_fields)*
                }

                pub fn extend(&mut self, records: Vec<#struct_type>) {
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

            impl From<Vec<#struct_type>> for #batch_type {
                fn from(records: Vec<#struct_type>) -> Self {
                    records.into_iter().fold(Self::default(), |mut multi, record| {
                        multi.push(record);
                        multi
                    })
                }
            }
        }
    }
}
