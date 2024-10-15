pub use argon2;

use argon2::password_hash::{Error as ArgonError, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

pub fn salt_osrng() -> SaltString {
    SaltString::generate(&mut OsRng)
}

/// Hashes a password with a provided salt and default `Argon2` parameters.
///
/// # Errors
///
/// Returns an error if hashing fails.
pub fn hash_default<T: AsRef<[u8]>>(
    password: T,
    salt: &SaltString,
) -> Result<PasswordHash<'_>, ArgonError> {
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_ref(), salt)
}

/// Hashes a password with a an `OsRng`-generated salt and default `Argon2` parameters.
///
/// # Errors
///
/// Returns an error if hashing fails.
pub fn hash_default_string<T: AsRef<[u8]>>(password: T) -> Result<String, ArgonError> {
    let salt = salt_osrng();
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_ref(), &salt)
        .map(|pwh| pwh.to_string())
}

/// Verifies a password against a password hash.
///
/// # Errors
///
/// Throws an error if the provided password hash cannot be parsed or if verification fails.
pub fn verify<T: AsRef<[u8]>>(password: T, password_hash: &str) -> Result<(), ArgonError> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Argon2::default().verify_password(password.as_ref(), &parsed_hash)
}
