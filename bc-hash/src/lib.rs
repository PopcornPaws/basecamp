#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]

#[cfg(feature = "hmac")]
pub mod hmac;
#[cfg(feature = "password")]
pub mod password;

macro_rules! hash {
    ($name:ident, $fun:ident, $len:expr) => {
        pub fn $fun<T: AsRef<[u8]>>(input: T) -> [u8; $len] {
            use sha3::Digest;
            sha3::$name::new()
                .chain_update(input.as_ref())
                .finalize()
                .into()
        }
    };
}

hash!(Sha3_256, sha3_256, 32);
hash!(Sha3_512, sha3_512, 64);
hash!(Keccak256, keccak256, 32);
hash!(Keccak512, keccak512, 64);
