//! Types representing various parts of `.hltas` scripts.

use std::{io::Write, num::NonZeroU32};

use cookie_factory::GenError;
use nom;

use crate::{read, write};

/// A HLTAS script.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HLTAS<'a> {
    /// Properties before the frames section.
    pub properties: Properties<'a>,
    /// Contents of the frames section.
    pub lines: Vec<Line<'a>>,
}

/// Recognized HLTAS properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Properties<'a> {
    /// Name of the demo to record.
    pub demo: Option<&'a str>,
    /// Name of the save file to use for saving after the script has finished.
    pub save: Option<&'a str>,
    /// Frametime for 0 ms ducktaps.
    pub frametime_0ms: Option<&'a str>,
    /// RNG seeds.
    pub seeds: Option<Seeds>,
}

/// Shared and non-shared RNG seeds.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Seeds {
    /// The shared RNG seed, used by the weapon spread.
    pub shared: u32,
    /// The non-shared RNG seed, used by all other randomness.
    pub non_shared: i64,
}

/// A line in the frames section.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Line<'a> {
    /// A frame bulk.
    FrameBulk(FrameBulk<'a>),
    /// A save-load.
    Save(&'a str),
    /// A line that sets the shared seed to use next load.
    SharedSeed(u32),
    /// Sets or resets the strafing buttons.
    Buttons(Buttons),
    /// Minimum speed for the optimal leave-ground-action speed.
    LGAGSTMinSpeed(f32),
    /// An engine reset.
    Reset {
        /// The non-shared seed to use for this reset.
        non_shared_seed: i64,
    },
    /// A comment line.
    Comment(&'a str),
}

/// A buttons line.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Buttons {
    /// Reset the strafing buttons.
    Reset,
    /// Set the strafing buttons.
    Set {
        /// Button to use when strafing left in the air.
        air_left: Button,
        /// Button to use when strafing right in the air.
        air_right: Button,
        /// Button to use when strafing left on the ground.
        ground_left: Button,
        /// Button to use when strafing right on the ground.
        ground_right: Button,
    },
}

/// Buttons which can be used for strafing.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Button {
    /// `+forward`
    Forward,
    /// `+forward;+moveleft`
    ForwardLeft,
    /// `+moveleft`
    Left,
    /// `+back;+moveleft`
    BackLeft,
    /// `+back`
    Back,
    /// `+back;+moveright`
    BackRight,
    /// `+moveright`
    Right,
    /// `+forward;+moveright`
    ForwardRight,
}

/// Represents a number of similar frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameBulk<'a> {
    /// Automatic actions such as strafing, auto-jump, etc.
    pub auto_actions: AutoActions,
    /// Manually specified movement keys.
    pub movement_keys: MovementKeys,
    /// Manually specified action keys.
    pub action_keys: ActionKeys,
    /// Frame time of each of this frame bulk's frames.
    pub frame_time: &'a str,
    /// Pitch angle to set.
    pub pitch: Option<f32>,
    /// Number of frames in this frame bulk.
    pub frame_count: NonZeroU32,
    /// The console command to run every frame of this frame bulk.
    pub console_command: Option<&'a str>,
}

/// Automatic actions such as strafing, auto-jump, etc.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AutoActions {
    /// Yaw angle adjustment and strafing.
    pub movement: Option<AutoMovement>,
    /// Automatic jumping and ducktapping.
    pub leave_ground_action: Option<LeaveGroundAction>,
    /// Automatic jumpbug.
    pub jump_bug: Option<JumpBug>,
    /// Duck right before a non-ground collision would occur.
    pub duck_before_collision: Option<DuckBeforeCollision>,
    /// Duck before collision with ground would occur.
    pub duck_before_ground: Option<DuckBeforeGround>,
    /// Duck right before jumping, for example for the long jump module.
    pub duck_when_jump: Option<DuckWhenJump>,
}

/// Automatic yaw angle adjustment and strafing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoMovement {
    /// Set the yaw angle to this value.
    SetYaw(f32),
    /// Automatic strafing.
    Strafe(StrafeSettings),
}

/// Automatic strafing settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrafeSettings {
    /// Strafing type.
    pub type_: StrafeType,
    /// Strafing direction.
    pub dir: StrafeDir,
}

/// Type of automatic strafing.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StrafeType {
    /// Gain as much speed as possible.
    MaxAccel,
    /// Turn as quickly as possible.
    MaxAngle,
    /// Lose as much speed as possible.
    MaxDeccel,
    /// Turn without changing the velocity magnitude.
    ConstSpeed,
}

/// Direction of automatic strafing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrafeDir {
    /// Turn left.
    Left,
    /// Turn right.
    Right,
    /// Let the strafing type decide. Most useful with maximum decceleration.
    Best,
    /// Strafe towards this yaw angle.
    Yaw(f32),
    /// Strafe towards this point.
    Point { x: f32, y: f32 },
    /// Strafe along a line.
    Line {
        /// The line goes from the player position in the direction of this yaw angle.
        yaw: f32,
    },
}

/// Number of times this action must be executed.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Times {
    /// Any number of times over the duration of this frame bulk.
    UnlimitedWithinFrameBulk,
    /// This exact number of times, possibly past this frame bulk.
    ///
    /// The action is overridden by the next frame bulk with the same action.
    Limited(NonZeroU32),
}

/// Leave the ground automatically.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LeaveGroundAction {
    /// Speed at which to leave the ground.
    pub speed: LeaveGroundActionSpeed,
    /// Number of times to do the action. `0` means unlimited.
    pub times: Times,
    /// How to leave the ground.
    pub type_: LeaveGroundActionType,
}

/// Speed at which to leave the ground.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LeaveGroundActionSpeed {
    /// Any speed.
    Any,
    /// Leave the ground when it's better to strafe in the air than on the ground.
    Optimal,
    /// Leave the ground when it's better to strafe in the air than on the ground; doesn't take
    /// into account maxspeed reduction due to e.g. ducking.
    OptimalWithFullMaxspeed,
}

/// How to leave the ground.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LeaveGroundActionType {
    /// By jumping.
    Jump,
    /// By ducktapping.
    DuckTap {
        /// Use 0 ms ducktapping.
        zero_ms: bool,
    },
}

/// Automatic jumpbug properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct JumpBug {
    /// Number of times to do the action. `0` means unlimited.
    pub times: Times,
}

/// Duck-before-collision properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DuckBeforeCollision {
    /// Number of times to do the action. `0` means unlimited.
    pub times: Times,
    pub including_ceilings: bool,
}

/// Duck-before-ground properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DuckBeforeGround {
    /// Number of times to do the action. `0` means unlimited.
    pub times: Times,
}

/// Duck-when-jump properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DuckWhenJump {
    /// Number of times to do the action. `0` means unlimited.
    pub times: Times,
}

/// Manually specified movement keys.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct MovementKeys {
    /// `+forward`
    pub forward: bool,
    /// `+moveleft`
    pub left: bool,
    /// `+moveright`
    pub right: bool,
    /// `+back`
    pub back: bool,
    /// `+moveup`
    pub up: bool,
    /// `+movedown`
    pub down: bool,
}

/// Manually specified action keys.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct ActionKeys {
    /// `+jump`
    pub jump: bool,
    /// `+duck`
    pub duck: bool,
    /// `+use`
    pub use_: bool,
    /// `+attack1`
    pub attack_1: bool,
    /// `+attack2`
    pub attack_2: bool,
    /// `+reload`
    pub reload: bool,
}

impl<'a> HLTAS<'a> {
    /// Parses a `.hltas` script.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate hltas;
    /// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::fs::read_to_string;
    /// use hltas::HLTAS;
    ///
    /// let contents = read_to_string("script.hltas")?;
    /// match HLTAS::from_str(&contents) {
    ///     Ok(hltas) => { /* ... */ }
    ///
    ///     // The errors are pretty-printed with context.
    ///     Err(error) => println!("{}", error),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::should_implement_trait)] // FromStr does not allow borrowing from the &str.
    pub fn from_str(input: &'a str) -> Result<Self, read::Error> {
        match read::hltas(input) {
            Ok((_, hltas)) => Ok(hltas),
            Err(nom::Err::Error(mut e)) | Err(nom::Err::Failure(mut e)) => {
                // Set the whole input to get correct line and column numbers in the error message.
                e.whole_input = input;
                Err(e)
            }
            Err(nom::Err::Incomplete(_)) => unreachable!(), // We don't use streaming parsers.
        }
    }

    /// Outputs the script in the `.hltas` format.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate hltas;
    /// use std::fs::File;
    /// use hltas::HLTAS;
    ///
    /// fn save_script(hltas: &HLTAS) -> Result<(), Box<dyn std::error::Error>> {
    ///     let file = File::create("script.hltas")?;
    ///     hltas.to_writer(file)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn to_writer<W: Write>(&self, writer: W) -> Result<(), GenError> {
        write::hltas(writer, self)
    }
}

impl<'a> FrameBulk<'a> {
    /// Returns a `FrameBulk` with the given frame time and frame count of 1 and otherwise empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate hltas;
    /// # fn foo() {
    /// use hltas::types::FrameBulk;
    ///
    /// let frame_bulk = FrameBulk::with_frame_time("0.001");
    /// assert_eq!(frame_bulk.frame_time, "0.001");
    /// assert_eq!(frame_bulk.frame_count.get(), 1);
    ///
    /// // The rest is empty.
    /// assert_eq!(frame_bulk.movement_keys.forward, false);
    /// # }
    /// ```
    #[inline]
    pub fn with_frame_time(frame_time: &'a str) -> Self {
        Self {
            auto_actions: Default::default(),
            movement_keys: Default::default(),
            action_keys: Default::default(),
            frame_time,
            pitch: None,
            frame_count: NonZeroU32::new(1).unwrap(),
            console_command: None,
        }
    }
}

impl From<u32> for Times {
    #[inline]
    fn from(x: u32) -> Self {
        if x == 0 {
            Times::UnlimitedWithinFrameBulk
        } else {
            Times::Limited(NonZeroU32::new(x).unwrap())
        }
    }
}

impl From<Times> for u32 {
    #[inline]
    fn from(x: Times) -> Self {
        if let Times::Limited(t) = x {
            t.get()
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        fs::{read_dir, read_to_string},
        str::from_utf8,
    };

    #[test]
    fn parse() {
        for entry in read_dir("test-data/parse")
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| name.ends_with(".hltas"))
                    .unwrap_or(false)
            })
        {
            let contents = read_to_string(entry.path()).unwrap();
            assert!(HLTAS::from_str(&contents).is_ok());
        }
    }

    #[test]
    fn parse_write_parse() {
        for entry in read_dir("test-data/parse")
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| name.ends_with(".hltas"))
                    .unwrap_or(false)
            })
        {
            let contents = read_to_string(entry.path()).unwrap();
            let hltas = HLTAS::from_str(&contents).unwrap();

            let mut output = Vec::new();
            hltas.to_writer(&mut output).unwrap();

            let hltas_2 = HLTAS::from_str(from_utf8(&output).unwrap()).unwrap();
            assert_eq!(hltas, hltas_2);
        }
    }

    fn bhop_gt() -> HLTAS<'static> {
        HLTAS {
            properties: Properties {
                demo: Some("bhop"),
                frametime_0ms: Some("0.0000001"),
                save: None,
                seeds: None,
            },
            lines: vec![
                Line::FrameBulk(FrameBulk {
                    console_command: Some("sensitivity 0;bxt_timer_reset;bxt_taslog"),
                    ..FrameBulk::with_frame_time("0.001")
                }),
                Line::FrameBulk(FrameBulk {
                    frame_count: NonZeroU32::new(5).unwrap(),
                    ..FrameBulk::with_frame_time("0.001")
                }),
                Line::FrameBulk(FrameBulk {
                    auto_actions: AutoActions {
                        movement: Some(AutoMovement::Strafe(StrafeSettings {
                            type_: StrafeType::MaxAccel,
                            dir: StrafeDir::Yaw(170.),
                        })),
                        ..AutoActions::default()
                    },
                    frame_count: NonZeroU32::new(400).unwrap(),
                    pitch: Some(0.),
                    ..FrameBulk::with_frame_time("0.001")
                }),
                Line::FrameBulk(FrameBulk {
                    frame_count: NonZeroU32::new(2951).unwrap(),
                    ..FrameBulk::with_frame_time("0.001")
                }),
                Line::FrameBulk(FrameBulk {
                    auto_actions: AutoActions {
                        movement: Some(AutoMovement::Strafe(StrafeSettings {
                            type_: StrafeType::MaxAccel,
                            dir: StrafeDir::Yaw(90.),
                        })),
                        ..AutoActions::default()
                    },
                    frame_count: NonZeroU32::new(1).unwrap(),
                    console_command: Some("bxt_timer_start"),
                    ..FrameBulk::with_frame_time("0.001")
                }),
                Line::Comment(" More frames because some of them get converted to 0ms"),
                Line::FrameBulk(FrameBulk {
                    auto_actions: AutoActions {
                        movement: Some(AutoMovement::Strafe(StrafeSettings {
                            type_: StrafeType::MaxAccel,
                            dir: StrafeDir::Yaw(90.),
                        })),
                        leave_ground_action: Some(LeaveGroundAction {
                            speed: LeaveGroundActionSpeed::Optimal,
                            times: Times::UnlimitedWithinFrameBulk,
                            type_: LeaveGroundActionType::DuckTap { zero_ms: true },
                        }),
                        ..AutoActions::default()
                    },
                    frame_count: NonZeroU32::new(5315).unwrap(),
                    ..FrameBulk::with_frame_time("0.001")
                }),
                Line::FrameBulk(FrameBulk {
                    console_command: Some(
                        "stop;bxt_timer_stop;pause;sensitivity 1;_bxt_taslog 0;bxt_taslog;\
                         //condebug",
                    ),
                    ..FrameBulk::with_frame_time("0.001")
                }),
            ],
        }
    }

    #[test]
    fn validate() {
        let contents = read_to_string("test-data/parse/bhop.hltas").unwrap();
        let hltas = HLTAS::from_str(&contents).unwrap();

        let gt = bhop_gt();
        assert_eq!(hltas, gt);
    }

    #[test]
    fn parse_write_parse_validate() {
        let contents = read_to_string("test-data/parse/bhop.hltas").unwrap();
        let hltas = HLTAS::from_str(&contents).unwrap();

        let mut output = Vec::new();
        hltas.to_writer(&mut output).unwrap();

        let hltas = HLTAS::from_str(from_utf8(&output).unwrap()).unwrap();

        let gt = bhop_gt();
        assert_eq!(hltas, gt);
    }

    macro_rules! test_error {
        ($test_name:ident, $filename:literal, $context:ident) => {
            #[test]
            fn $test_name() {
                let contents =
                    read_to_string(concat!("test-data/error/", $filename, ".hltas")).unwrap();
                let err = HLTAS::from_str(&contents).unwrap_err();
                assert_eq!(err.context, Some(read::Context::$context));
            }
        };
    }

    test_error! { error_no_version, "no-version", ErrorReadingVersion }
    test_error! { error_too_high_version, "too-high-version", VersionTooHigh }
    test_error! { error_no_save_name, "no-save-name", NoSaveName }
    test_error! { error_no_seed, "no-seed", NoSeed }
    test_error! { error_no_buttons, "no-buttons", NoButtons }
    test_error! { error_no_lgagst_min_speed, "no-lgagst-min-speed", NoLGAGSTMinSpeed }
    test_error! { error_no_reset_seed, "no-reset-seed", NoResetSeed }
    test_error! { error_both_autojump_ducktap, "both-j-d", BothAutoJumpAndDuckTap }
    test_error! { error_no_yaw, "no-yaw", NoYaw }
    test_error! { error_no_lgagst_action, "no-lgagst-action", NoLeaveGroundAction }
    test_error! { error_lgagst_action_times, "lgagst-action-times", TimesOnLeaveGroundAction  }
}
