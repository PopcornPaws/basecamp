pub use bc_batch_derive::Batch;

#[cfg(test)]
mod test {
    use super::*;
    use uuid::Uuid;

    #[derive(Batch)]
    struct Foo {
        foo: Uuid,
        bar: String,
        baz: Option<u32>,
        asd_jkl: Vec<bool>,
    }

    #[test]
    fn batch_derive_macro_works() {
        let foos = vec![
            Foo {
                foo: Uuid::nil(),
                bar: "first".to_string(),
                baz: None,
                asd_jkl: vec![true, false, true],
            },
            Foo {
                foo: Uuid::nil(),
                bar: "second".to_string(),
                baz: Some(123),
                asd_jkl: vec![false, true, false, true],
            },
            Foo {
                foo: Uuid::nil(),
                bar: "third".to_string(),
                baz: Some(0),
                asd_jkl: vec![true, true, false, false],
            },
        ];

        let batch = FooBatch::from(foos);
        assert_eq!(batch.foo, vec![Uuid::nil(); 3]);
        assert_eq!(batch.bar, ["first", "second", "third"]);
        assert_eq!(batch.baz, [None, Some(123), Some(0)]);
        assert_eq!(
            batch.asd_jkl,
            [
                vec![true, false, true],
                vec![false, true, false, true],
                vec![true, true, false, false]
            ]
        );
    }
}
