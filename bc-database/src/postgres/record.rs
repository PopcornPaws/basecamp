pub use bc_record_derive::Record;

// TODO handle one-to-many relationships with flattened data properly
pub trait Record: Sized {
    type Batch: From<Vec<Self>>;
}

#[cfg(test)]
mod test {
    use super::Record;

    #[derive(Clone, Debug, Record)]
    #[record(table = test)]
    struct TestRecord {
        id: i16,
        foo: String,
        bar: i64,
        baz: Vec<u8>,
        #[record(flatten)]
        quux: Vec<InnerRecord>,
    }

    #[derive(Clone, Debug, Record)]
    #[record(table = inner_test)]
    struct InnerRecord {
        foo: String,
        bar: Vec<u8>,
        baz: bool,
    }

    fn dummy_records() -> [TestRecord; 3] {
        [
            TestRecord {
                id: 0,
                foo: "stinky".to_string(),
                bar: -34,
                baz: vec![1, 2, 3],
                quux: vec![
                    InnerRecord {
                        foo: "hello".to_string(),
                        bar: vec![1, 2, 3, 4, 5],
                        baz: true,
                    },
                    InnerRecord {
                        foo: "bello".to_string(),
                        bar: vec![10, 20],
                        baz: false,
                    },
                ],
            },
            TestRecord {
                id: 1,
                foo: "spongy".to_string(),
                bar: 1234,
                baz: vec![4],
                quux: vec![InnerRecord {
                    foo: "yello".to_string(),
                    bar: vec![100, 200, 250],
                    baz: true,
                }],
            },
            TestRecord {
                id: 2,
                foo: "stingy".to_string(),
                bar: 0,
                baz: vec![],
                quux: vec![],
            },
        ]
    }

    #[test]
    fn push_to_batch() {
        let [r_0, r_1, r_2] = dummy_records();
        let mut batch = BatchTestRecord::new();
        batch.push(r_0);
        batch.push(r_1);
        batch.push(r_2);

        assert_eq!(batch.id, &[0, 1, 2]);
        assert_eq!(batch.foo, &["stinky", "spongy", "stingy"]);
        assert_eq!(batch.bar, &[-34, 1234, 0]);
        assert_eq!(batch.baz, &[vec![1, 2, 3], vec![4], vec![]]);
        assert_eq!(batch.quux.foo, &["hello", "bello", "yello"]);
        assert_eq!(
            batch.quux.bar,
            &[vec![1, 2, 3, 4, 5], vec![10, 20], vec![100, 200, 250]]
        );
        assert_eq!(batch.quux.baz, &[true, false, true]);

        assert_eq!(
            BatchTestRecord::raw_insert_query(),
            "INSERT INTO test (id,foo,bar,baz) SELECT * FROM UNNEST($1::INT2[],$2::TEXT[],$3::INT8[],$4::BYTEA[])"
        );
        assert_eq!(
            BatchInnerRecord::raw_insert_query(),
            "INSERT INTO inner_test (foo,bar,baz) SELECT * FROM UNNEST($1::TEXT[],$2::BYTEA[],$3::BOOL[])"
        );
    }

    #[test]
    fn batch_from_single_vec() {
        let batch = BatchTestRecord::from(dummy_records().to_vec());
        assert_eq!(batch.id, &[0, 1, 2]);
        assert_eq!(batch.foo, &["stinky", "spongy", "stingy"]);
        assert_eq!(batch.bar, &[-34, 1234, 0]);
        assert_eq!(batch.baz, &[vec![1, 2, 3], vec![4], vec![]]);
        assert_eq!(
            batch.quux.bar,
            &[vec![1, 2, 3, 4, 5], vec![10, 20], vec![100, 200, 250]]
        );
        assert_eq!(batch.quux.baz, &[true, false, true]);
    }

    #[tokio::test]
    async fn insert_batch() {
        use crate::postgres::Config;
        use sqlx::Row;

        let mut config = Config::from_env();
        config.options = config.options.with_database("batch_insert");
        let pool = config.connect_with_migration().await.unwrap();

        let batch = BatchTestRecord::from(dummy_records().to_vec());
        batch.insert(&pool).await.unwrap();

        let count = sqlx::query("SELECT COUNT (*) FROM test")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.get::<i64, &str>("count"), 3);
        let count = sqlx::query("SELECT COUNT (*) FROM inner_test")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.get::<i64, &str>("count"), 3);

        sqlx::query("TRUNCATE TABLE test, inner_test")
            .execute(&pool)
            .await
            .unwrap();
    }
}
