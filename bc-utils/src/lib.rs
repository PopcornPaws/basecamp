#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]

pub use bc_utils_derive::SerdeAsString;
/// Helper derive macro for serializing and deserializing types that implement `Display` and
/// `FromStr`.
pub trait SerdeAsString: std::str::FromStr + std::fmt::Display {}

/// Fills an URL path (route) with input values by matching the respective input keys to patterns
/// in the URL string.
///
/// The input keys (variable names) must match the key in the route. Furthermore, the key's type must
/// implement `ToString`.
///
/// # Examples
///
/// ```
/// # use bc_utils::fill_route;
/// // no change, no input pattern present
/// assert_eq!(fill_route!(":{}", "/v1/id"), "/v1/id");
///
/// let my_id = 123;
/// // 'my_id' replaced with its stringified value
/// assert_eq!(fill_route!(":{}", "/v1/id/:my_id", my_id), "/v1/id/123");
///
/// let my_id = 123;
/// let your_id = 777;
/// let their_id = 999;
/// // all ids replaced with their stringified value
/// // note the different formatting option
/// assert_eq!(
///     fill_route!("<{}>", "/v1/id/<my_id>/range/<your_id>:<their_id>", my_id, your_id, their_id),
///     "/v1/id/123/range/777:999"
/// );
/// ```
#[macro_export]
macro_rules! fill_route {
    ($pattern:literal, $route:expr $(, $key:ident)*) => {
        {
            #[allow(unused_mut)]
            let mut result = $route.to_string(); // with no input keys, mut is unused
            $(
                let replacement = &$key.to_string();
                result = result.replace(&format!($pattern, stringify!($key)), replacement);
            )*
            result
        }
    };
}

/// Fills an URL with values matching `{}` patterns within the original string.
///
/// URLs of the form `/foo/{bar}/baz/{quux}` are used by `actix-web` and `axum` ^0.8 web frameworks
/// to match input routes.
///
/// # Examples
///
/// ```
/// # use bc_utils::route;
/// let simple = "/v1/id";
/// let complex = "/hello/{a}/bello/{b}/{c}:{d}/asd";
/// let a = 1234u16;
/// let b = "yello";
/// let c = 111u64;
/// let d = 222u64;
/// assert_eq!(route!(simple), "/v1/id");
/// assert_eq!(
///     route!(complex, a, b, c, d),
///     "/hello/1234/bello/yello/111:222/asd"
/// );
///
/// ```
#[macro_export]
macro_rules! route {
    ($route:expr $(, $key:ident)*) => {
        $crate::fill_route!("{{{}}}", $route $(, $key)*)
    };
}

#[cfg(test)]
mod test {
    mod serde_as_string_derive {
        use crate::SerdeAsString;
        use std::str::FromStr;

        #[derive(Clone, Copy, Debug, SerdeAsString)]
        struct Foo;

        impl std::fmt::Display for Foo {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "foo")
            }
        }

        impl FromStr for Foo {
            type Err = u8;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "foo" => Ok(Self),
                    _ => Err(0),
                }
            }
        }

        #[derive(Clone, Copy, Debug, SerdeAsString)]
        struct Bar;

        impl std::fmt::Display for Bar {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "bar")
            }
        }

        impl FromStr for Bar {
            type Err = u8;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "bar" => Ok(Self),
                    _ => Err(0),
                }
            }
        }

        #[test]
        fn works() {
            assert!(Foo::from_str("foo").is_ok());
            assert!(Foo::from_str("bar").is_err());
            assert_eq!(Foo.to_string(), "foo");

            assert!(serde_json::from_str::<Foo>("\"foo\"").is_ok());
            assert!(serde_json::from_str::<Foo>("\"bar\"").is_err());
            assert_eq!(serde_json::to_string(&Foo).unwrap(), "\"foo\"");

            assert!(Bar::from_str("bar").is_ok());
            assert!(Bar::from_str("foo").is_err());
            assert_eq!(Bar.to_string(), "bar");

            assert!(serde_json::from_str::<Bar>("\"bar\"").is_ok());
            assert!(serde_json::from_str::<Bar>("\"foo\"").is_err());
            assert_eq!(serde_json::to_string(&Bar).unwrap(), "\"bar\"");
        }
    }

    mod batch_import {
        use bc_batch::Batch;

        #[derive(Batch)]
        struct Foo {
            asd: u32,
        }

        #[test]
        fn works() {
            let batch = FooBatch::from(vec![]);
            assert!(batch.asd.is_empty());
        }
    }
}
