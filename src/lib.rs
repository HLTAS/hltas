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
//!         assert_eq!(hltas.properties.demo.as_deref(), Some("test"));
//!
//!         if let Line::FrameBulk(frame_bulk) = &hltas.lines[0] {
//!             assert_eq!(
//!                 frame_bulk.auto_actions.jump_bug,
//!                 Some(JumpBug { times: Times::UnlimitedWithinFrameBulk })
//!             );
//!             assert_eq!(&frame_bulk.frame_time, "0.001");
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
//!
//! # Features
//!
//! - `serde1`: implements [serde]'s [`Serialize`] and [`Deserialize`] traits for all types.
//!
//! - `proptest1`: implements [proptest]'s [`Arbitrary`] trait for all types. Only "valid" contents
//!   are generated, as in, writing to string and parsing back will work and give you the same
//!   result.
//!
//! [serde]: https://crates.io/crates/serde
//! [`Serialize`]: https://docs.serde.rs/serde/trait.Serialize.html
//! [`Deserialize`]: https://docs.serde.rs/serde/trait.Deserialize.html
//! [proptest]: https://crates.io/crates/proptest
//! [`Arbitrary`]: https://docs.rs/proptest/1.5.0/proptest/arbitrary/trait.Arbitrary.html

#![doc(html_root_url = "https://docs.rs/hltas/0.8.0")]
#![deny(unsafe_code)]

pub mod types;
pub use types::HLTAS;

pub mod read;
pub mod write;
