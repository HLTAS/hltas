{{crate}}
=====

[![crates.io](https://img.shields.io/crates/v/hltas.svg)](https://crates.io/crates/hltas)
[![Documentation](https://docs.rs/hltas/badge.svg)](https://docs.rs/hltas)

{{readme}}

## C++ Wrapper

Also included is a C++ wrapper, exporting the same C++ interface as the previous C++ version of HLTAS.

### Using the C++ wrapper from CMake
- You will need Rust: either from your distribution's packages, or from [rustup](https://rustup.rs/).
- From your project's `CMakeLists.txt`, call `add_subdirectory("path/to/hltas")`.
- Link to the `hltas-cpp` target: `target_link_libraries(your-target hltas-cpp)`.

License: {{license}}
