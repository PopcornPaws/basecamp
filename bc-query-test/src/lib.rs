#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]

#[cfg(test)]
mod test {
    use bc_query::QueryBuilder;

    #[derive(Debug, PartialEq, Eq)]
    struct TestWrapper(String);

    impl std::fmt::Display for TestWrapper {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Default, QueryBuilder)]
    struct TestQuery {
        foo: Option<usize>,
        bar: Option<String>,
        baz: Option<TestWrapper>,
    }

    #[test]
    fn builder_impl() {
        let test_query = TestQuery::new()
            .with_foo(212)
            .with_bar("hello".to_string())
            .with_baz(TestWrapper("wrapped".to_string()));

        assert_eq!(test_query.foo, Some(212));
        assert_eq!(test_query.bar.as_deref(), Some("hello"));
        assert_eq!(test_query.baz, Some(TestWrapper("wrapped".to_string())));
    }

    #[test]
    fn append_to_impl() {
        let test_query = TestQuery::new()
            .with_foo(212)
            .with_bar("hello".to_string())
            .with_baz(TestWrapper("wrapped".to_string()));

        let mut base = "/base".to_string();
        test_query.append_to(&mut base);
        assert_eq!(base, "/base?foo=212&bar=hello&baz=wrapped");

        let mut base = "/base".to_string();
        TestQuery::new().append_to(&mut base);
        assert_eq!(base, "/base");

        let mut base = "/base".to_string();
        TestQuery::new()
            .with_bar("hello".to_string())
            .append_to(&mut base);
        assert_eq!(base, "/base?bar=hello");

        let mut base = "/base".to_string();
        TestQuery::new()
            .with_foo(123)
            .with_baz(TestWrapper("hello".to_string()))
            .append_to(&mut base);
        assert_eq!(base, "/base?foo=123&baz=hello");
    }
}
