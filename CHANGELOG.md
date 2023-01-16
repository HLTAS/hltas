# Changelog

## [Unreleased]

## [0.7.0] - 16 Jan 2023
### Added
- `ChangeTarget::VectorialStrafingYawOffset`.
- `VectorialStrafingConstraints::LookAt`.

## [0.6.0] - 31 Aug 2022
### Added
- `StrafeDir::LeftRight`, `StrafeDir::RightLeft`.
- `serde`'s `Serialize` and `Deserialize` for all types under the `serde1` feature.
- `proptest`'s `Arbitrary` for all types under the `proptest1` feature.
- cpp: `Frame::HasYaw()`, `Frame::HasXY()`.

## [0.5.0] - 31 Dec 2021
### Added
- `Properties::load_command`.
- `Line::TargetYawOverride`.
- cpp: `Input::ToString()`, `Input::FromString()`.

### Changed
- Changed all reference components of the `HLTAS` type into owned types thus making it easy to construct a `HLTAS` programmatically and to store `HLTAS` instances.

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

[Unreleased]: https://github.com/HLTAS/hltas/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/HLTAS/hltas/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/HLTAS/hltas/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/HLTAS/hltas/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/HLTAS/hltas/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/HLTAS/hltas/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/HLTAS/hltas/compare/v0.1.0...v0.2.0
