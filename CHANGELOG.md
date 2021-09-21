# Changelog

## [Unreleased]
### Changed
- Changed all `&'a str` types into `Cow<'a, str>` thus making it possible to construct a new `HLTAS` programmatically with owned strings.

## [0.4.0] - 21 Jun 2020
### Added
- `VectorialStrafingConstraints::VelocityYawLocking`.

### Changed
- Made tolerance optional in `target_yaw` lines. When tolerance is absent (e.g. `target_yaw velocity_avg`) it is assumed to be zero.

## [0.3.0] - 30 Jan 2020
### Added
- `Line::Change`.

## [0.2.0] - 3 Jan 2020
### Added
- `Properties::hlstrafe_version`.
- `Line::VectorialStrafing` and `Line::VectorialStrafingConstraints`.

[Unreleased]: https://github.com/HLTAS/hltas/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/HLTAS/hltas/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/HLTAS/hltas/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/HLTAS/hltas/compare/v0.1.0...v0.2.0
