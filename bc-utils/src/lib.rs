#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]

pub use bc_utils_derive::SerdeAsString;
/// Helper derive macro for serializing and deserializing types that implement `Display` and
/// `FromStr`.
pub trait SerdeAsString: std::str::FromStr + std::fmt::Display {}

#[macro_export]
macro_rules! url {
    ($pattern:literal, $path:expr $(, $key:ident)*) => {
        {
            #[allow(unused_mut)]
            let mut result = $path.to_string(); // with no input keys, mut is unused
            $(
                let replacement = &$key.to_string();
                result = result.replace(&format!($pattern, stringify!($key)), replacement);
            )*
            result
        }
    };
}

// NOTE this is actix-web specific
#[macro_export]
macro_rules! actix_url {
    ($path:expr $(, $key:ident)*) => {
        $crate::url!("{{{}}}", $path $(, $key)*)
    };
}

// NOTE this is axum specific
#[macro_export]
macro_rules! axum_url {
    ($path:expr $(, $key:ident)*) => {
        $crate::url!(":{}", $path $(, $key)*)
    };
}

#[cfg(test)]
mod test {
    mod url {
        #[test]
        fn actix() {
            let simple = "/v1/id";
            let complex = "/hello/{a}/bello/{b}/{c}:{d}/asd";
            let a = 1234u16;
            let b = "yello";
            let c = 111u64;
            let d = 222u64;
            assert_eq!(actix_url!(simple), "/v1/id");
            assert_eq!(
                actix_url!(complex, a, b, c, d),
                "/hello/1234/bello/yello/111:222/asd"
            );
        }

        #[test]
        fn axum() {
            let simple = "/v1/id";
            let complex = "/hello/:a/bello/:b/:c::d/asd";
            let a = 1234u16;
            let b = "yello";
            let c = 111u64;
            let d = 222u64;
            assert_eq!(axum_url!(simple), "/v1/id");
            assert_eq!(
                axum_url!(complex, a, b, c, d),
                "/hello/1234/bello/yello/111:222/asd"
            );
        }
    }

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
}
