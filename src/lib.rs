//! A crate for reading and writing Half-Life TAS scripts (`.hltas`).
//!
//! # Examples
//!
//! ```
//! # extern crate hltas;
//! # fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! use hltas::{HLTAS, types::{JumpBug, Line, Times}};
//!
//! let contents = "\
//! version 1
//! demo test
//! frames
//! ------b---|------|------|0.001|-|-|5";
//!
//! match HLTAS::from_str(&contents) {
//!     Ok(hltas) => {
//!         assert_eq!(hltas.properties.demo, Some("test"));
//!
//!         if let Line::FrameBulk(frame_bulk) = hltas.lines[0] {
//!             assert_eq!(
//!                 frame_bulk.auto_actions.jump_bug,
//!                 Some(JumpBug { times: Times::UnlimitedWithinFrameBulk })
//!             );
//!             assert_eq!(frame_bulk.frame_time, "0.001");
//!             assert_eq!(frame_bulk.frame_count.get(), 5);
//!         } else {
//!             unreachable!()
//!         }
//!     }
//!
//!     // The errors are pretty-printed with context.
//!     Err(error) => println!("{}", error),
//! }
//! # Ok(())
//! # }
//! ```

pub mod types;
pub use types::HLTAS;

pub mod read;
mod write;

pub mod capi;
#[allow(non_camel_case_types, non_snake_case, dead_code)]
mod hltas_cpp;
