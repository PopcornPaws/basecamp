#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::pedantic)]
#![warn(unused_crate_dependencies)]

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(test)]
mod test {
    use tokio as _;
}
