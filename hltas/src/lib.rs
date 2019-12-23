//! A crate for reading and writing Half-Life TAS scripts (`.hltas`).

pub mod types;
pub use types::HLTAS;

mod read;
mod write;

#[cfg(not(test))]
pub mod capi;
#[allow(non_camel_case_types, non_snake_case, dead_code)]
mod hltas_cpp;
