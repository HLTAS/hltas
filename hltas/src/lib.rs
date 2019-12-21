//! A crate for reading and writing Half-Life TAS scripts (`.hltas`).

pub mod read;
pub mod write;

pub mod types;
pub use types::HLTAS;

#[cfg(not(test))]
pub mod capi;
#[allow(non_camel_case_types, non_snake_case, dead_code)]
mod hltas_cpp;
