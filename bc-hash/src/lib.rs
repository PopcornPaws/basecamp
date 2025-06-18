#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]

#[cfg(feature = "hmac")]
pub mod hmac;
#[cfg(feature = "password")]
pub mod password;

#[cfg(any(feature = "sha2", feature = "sha3"))]
macro_rules! hash {
    ($package:ident, $name:ident, $fun:ident, $len:expr) => {
        pub fn $fun<T: AsRef<[u8]>>(input: T) -> [u8; $len] {
            use $package::Digest;
            $package::$name::new()
                .chain_update(input.as_ref())
                .finalize()
                .into()
        }
    };
}

#[cfg(feature = "sha2")]
hash!(sha2, Sha256, sha2_256, 32);
#[cfg(feature = "sha2")]
hash!(sha2, Sha512, sha2_512, 64);

#[cfg(feature = "sha3")]
hash!(sha3, Sha3_256, sha3_256, 32);
#[cfg(feature = "sha3")]
hash!(sha3, Sha3_512, sha3_512, 64);
#[cfg(feature = "sha3")]
hash!(sha3, Keccak256, keccak256, 32);
#[cfg(feature = "sha3")]
hash!(sha3, Keccak512, keccak512, 64);
