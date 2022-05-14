hltas
=====

[![crates.io](https://img.shields.io/crates/v/hltas.svg)](https://crates.io/crates/hltas)
[![Documentation](https://docs.rs/hltas/badge.svg)](https://docs.rs/hltas)

[CHANGELOG](https://github.com/HLTAS/hltas/blob/master/CHANGELOG.md)

A crate for reading and writing Half-Life TAS scripts (`.hltas`).

## Examples

```rust
use hltas::{HLTAS, types::{JumpBug, Line, Times}};

let contents = "\
version 1
demo test
frames
------b---|------|------|0.001|-|-|5";

match HLTAS::from_str(&contents) {
    Ok(hltas) => {
        assert_eq!(hltas.properties.demo.as_deref(), Some("test"));

        if let Line::FrameBulk(frame_bulk) = &hltas.lines[0] {
            assert_eq!(
                frame_bulk.auto_actions.jump_bug,
                Some(JumpBug { times: Times::UnlimitedWithinFrameBulk })
            );
            assert_eq!(&frame_bulk.frame_time, "0.001");
            assert_eq!(frame_bulk.frame_count.get(), 5);
        } else {
            unreachable!()
        }
    }

    // The errors are pretty-printed with context.
    Err(error) => println!("{}", error),
}
```

## Features

- `serde1`: implements [serde]'s [`Serialize`] and [`Deserialize`] traits for all types.

- `proptest1`: implements [proptest]'s [`Arbitrary`] trait for all types. Only "valid" contents
  are generated, as in, writing to string and parsing back will work and give you the same
  result.

[serde]: https://crates.io/crates/serde
[`Serialize`]: https://docs.serde.rs/serde/trait.Serialize.html
[`Deserialize`]: https://docs.serde.rs/serde/trait.Deserialize.html
[proptest]: https://crates.io/crates/proptest
[`Arbitrary`]: https://docs.rs/proptest/1.0.0/proptest/arbitrary/trait.Arbitrary.html

## C++ Wrapper

Also included is a C++ wrapper, exporting the same C++ interface as the previous C++ version of HLTAS.

### Using the C++ wrapper from CMake
- You will need Rust: either from your distribution's packages, or from [rustup](https://rustup.rs/).
- From your project's `CMakeLists.txt`, call `add_subdirectory("path/to/hltas")`.
- Link to the `hltas-cpp` target: `target_link_libraries(your-target hltas-cpp)`.

License: MIT/Apache-2.0
