//! Types representing various parts of `.hltas` scripts.

use std::{io::Write, num::NonZeroU32};

use cookie_factory::GenError;
#[cfg(feature = "proptest1")]
use proptest::prelude::*;
#[cfg(feature = "proptest1")]
use proptest_derive::Arbitrary;
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

use crate::{read, write};

/// A HLTAS script.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct HLTAS {
    /// Properties before the frames section.
    pub properties: Properties,
    /// Contents of the frames section.
    pub lines: Vec<Line>,
}

/// Recognized HLTAS properties.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct Properties {
    /// Name of the demo to record.
    #[cfg_attr(
        feature = "proptest1",
        proptest(strategy = "prop::option::of(arbitrary_property_value())")
    )]
    pub demo: Option<String>,
    /// Name of the save file to use for saving after the script has finished.
    #[cfg_attr(
        feature = "proptest1",
        proptest(strategy = "prop::option::of(arbitrary_property_value())")
    )]
    pub save: Option<String>,
    /// Frametime for 0 ms ducktaps.
    #[cfg_attr(
        feature = "proptest1",
        proptest(strategy = "prop::option::of(arbitrary_frame_time())")
    )]
    pub frametime_0ms: Option<String>,
    /// RNG seeds.
    pub seeds: Option<Seeds>,
    /// Version of the HLStrafe prediction this TAS was made for.
    ///
    /// This controls some inner workings of HLStrafe and is used to update the prediction code
    /// without causing old scripts to desync.
    #[cfg_attr(
        feature = "proptest1",
        proptest(strategy = "any::<u32>().prop_map(NonZeroU32::new)")
    )]
    pub hlstrafe_version: Option<NonZeroU32>,
    /// The command that loads the map or save before running the TAS.
    ///
    /// For example, if you need to run the TAS as `map bkz_goldbhop;bxt_tas_loadscript tas.hltas`,
    /// you can set this property to `map bkz_goldbhop`. Then you will be able to run the TAS by
    /// simply executing `bxt_tas_loadscript tas.hltas`, and the load command will be run
    /// automatically.
    #[cfg_attr(
        feature = "proptest1",
        proptest(strategy = "prop::option::of(arbitrary_property_value())")
    )]
    pub load_command: Option<String>,
}

/// Shared and non-shared RNG seeds.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct Seeds {
    /// The shared RNG seed, used by the weapon spread.
    pub shared: u32,
    /// The non-shared RNG seed, used by all other randomness.
    pub non_shared: i64,
}

/// A line in the frames section.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub enum Line {
    /// A frame bulk.
    FrameBulk(FrameBulk),
    /// A save-load.
    Save(#[cfg_attr(feature = "proptest1", proptest(regex = "\\S+"))] String),
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
    Comment(String),
    /// Enables or disables vectorial strafing.
    VectorialStrafing(bool),
    /// Sets the constraints for vectorial strafing.
    VectorialStrafingConstraints(VectorialStrafingConstraints),
    /// Starts smoothly changing a value.
    Change(Change),
    /// Overrides yaw and target yaw for the subsequent frames.
    TargetYawOverride(
        #[cfg_attr(
            feature = "proptest1",
            proptest(strategy = "prop::collection::vec(any::<f32>(), 1..100)")
        )]
        Vec<f32>,
    ),
}

/// A buttons line.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct FrameBulk {
    /// Automatic actions such as strafing, auto-jump, etc.
    pub auto_actions: AutoActions,
    /// Manually specified movement keys.
    pub movement_keys: MovementKeys,
    /// Manually specified action keys.
    pub action_keys: ActionKeys,
    /// Frame time of each of this frame bulk's frames.
    #[cfg_attr(feature = "proptest1", proptest(strategy = "arbitrary_frame_time()"))]
    pub frame_time: String,
    /// Pitch angle to set.
    pub pitch: Option<f32>,
    /// Number of frames in this frame bulk.
    #[cfg_attr(feature = "proptest1", proptest(strategy = "arbitrary_non_zero_u32()"))]
    pub frame_count: NonZeroU32,
    /// The console command to run every frame of this frame bulk.
    pub console_command: Option<String>,
}

/// Automatic actions such as strafing, auto-jump, etc.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub enum AutoMovement {
    /// Set the yaw angle to this value.
    SetYaw(f32),
    /// Automatic strafing.
    Strafe(StrafeSettings),
}

/// Automatic strafing settings.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct StrafeSettings {
    /// Strafing type.
    pub type_: StrafeType,
    /// Strafing direction.
    pub dir: StrafeDir,
}

/// Type of automatic strafing.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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
    /// Alternate turning left and right for this number of frames each.
    LeftRight(
        #[cfg_attr(feature = "proptest1", proptest(strategy = "arbitrary_non_zero_u32()"))]
        NonZeroU32,
    ),
    /// Alternate turning right and left for this number of frames each.
    RightLeft(
        #[cfg_attr(feature = "proptest1", proptest(strategy = "arbitrary_non_zero_u32()"))]
        NonZeroU32,
    ),
}

/// Number of times this action must be executed.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub enum Times {
    /// Any number of times over the duration of this frame bulk.
    UnlimitedWithinFrameBulk,
    /// This exact number of times, possibly past this frame bulk.
    ///
    /// The action is overridden by the next frame bulk with the same action.
    Limited(
        #[cfg_attr(feature = "proptest1", proptest(strategy = "arbitrary_non_zero_u32()"))]
        NonZeroU32,
    ),
}

/// Leave the ground automatically.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct JumpBug {
    /// Number of times to do the action. `0` means unlimited.
    pub times: Times,
}

/// Duck-before-collision properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct DuckBeforeCollision {
    /// Number of times to do the action. `0` means unlimited.
    pub times: Times,
    pub including_ceilings: bool,
}

/// Duck-before-ground properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct DuckBeforeGround {
    /// Number of times to do the action. `0` means unlimited.
    pub times: Times,
}

/// Duck-when-jump properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct DuckWhenJump {
    /// Number of times to do the action. `0` means unlimited.
    pub times: Times,
}

/// Manually specified movement keys.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
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

/// Constraints for the vectorial strafing algorithm.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub enum VectorialStrafingConstraints {
    /// Constrains the player yaw relative the velocity yaw.
    VelocityYaw {
        /// The player's yaw should remain within velocity yaw ± tolerance degrees.
        tolerance: f32,
    },
    /// Constrains the player yaw relative the yaw of velocity averaged over last two frames.
    AvgVelocityYaw {
        /// The player's yaw should remain within average velocity yaw ± tolerance degrees.
        tolerance: f32,
    },
    /// Constrains the player yaw to the velocity yaw, locking to the target strafing yaw.
    ///
    /// When the velocity yaw rotates past the target strafing yaw (usually the frame bulk yaw),
    /// the constraint locks the player yaw to the target strafing yaw. When the target strafing
    /// yaw changes, the yaw is unlocked and follows the velocity yaw until the next time it
    /// reaches the target strafing yaw, and so on.
    VelocityYawLocking {
        /// The player's yaw should remain within velocity yaw or target strafing yaw ± tolerance
        /// degrees.
        tolerance: f32,
    },
    /// Constrains the player yaw relative to the given yaw.
    Yaw {
        /// The target yaw in degrees.
        yaw: f32,
        /// The player's yaw should remain within yaw ± tolerance degrees.
        tolerance: f32,
    },
    /// Constrains the player yaw to the given range.
    ///
    /// The range is in degrees, mod 360, inclusive from both sides. The order matters: from 10 to
    /// 350 results in a wide angle range, and from 350 to 10 results in a narrow angle range
    /// opposite to the first one.
    YawRange {
        /// The lowest yaw angle of the range in degrees.
        from: f32,
        /// The highest yaw angle of the range in degrees.
        to: f32,
    },
    /// Constrains the player yaw to look at the given point.
    LookAt {
        /// Option to trace an entity's origin from entity index.
        entity: u32,
        /// Specified origin or offset from entity origin.
        x: f32,
        y: f32,
        z: f32,
    },
}

/// Description of the value to change.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub struct Change {
    /// The value to change.
    pub target: ChangeTarget,
    /// The final value after the change.
    pub final_value: f32,
    /// Duration, in seconds, over which to change the value.
    pub over: f32,
}

/// Values that can be affected by `Change`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "proptest1", derive(Arbitrary))]
pub enum ChangeTarget {
    /// The player's yaw angle.
    Yaw,
    /// The player's pitch angle.
    Pitch,
    /// The target yaw angle in the vectorial strafing constraints.
    VectorialStrafingYaw,
    /// The target yaw angle offset in the vectorial strafing constraints.
    VectorialStrafingYawOffset,
}

/// Generates arbitrary [`NonZeroU32`]s.
#[cfg(feature = "proptest1")]
fn arbitrary_non_zero_u32() -> impl Strategy<Value = NonZeroU32> {
    (1..=u32::MAX).prop_map(|x| NonZeroU32::new(x).unwrap())
}

/// Generates arbitrary valid frame times.
#[cfg(feature = "proptest1")]
fn arbitrary_frame_time() -> impl Strategy<Value = String> {
    any::<f64>().prop_map(|x| x.to_string())
}

/// Generates arbitrary valid property values (starting or ending with non-whitespace).
#[cfg(feature = "proptest1")]
fn arbitrary_property_value() -> impl Strategy<Value = String> {
    any_with::<String>("\\S|\\S\\PC*\\S".into())
}

impl HLTAS {
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
    pub fn from_str(input: &str) -> Result<Self, read::Error> {
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

impl FrameBulk {
    /// Returns a `FrameBulk` with the given frame time and frame count of 1 and otherwise empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate hltas;
    /// # fn foo() {
    /// use hltas::types::FrameBulk;
    ///
    /// let frame_bulk = FrameBulk::with_frame_time("0.001".to_owned());
    /// assert_eq!(&frame_bulk.frame_time, "0.001");
    /// assert_eq!(frame_bulk.frame_count.get(), 1);
    ///
    /// // The rest is empty.
    /// assert_eq!(frame_bulk.movement_keys.forward, false);
    /// # }
    /// ```
    #[inline]
    pub fn with_frame_time(frame_time: String) -> Self {
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

    fn bhop_gt() -> HLTAS {
        HLTAS {
            properties: Properties {
                demo: Some("bhop".to_owned()),
                frametime_0ms: Some("0.0000001".to_owned()),
                save: None,
                seeds: None,
                hlstrafe_version: Some(NonZeroU32::new(1).unwrap()),
                load_command: None,
            },
            lines: vec![
                Line::FrameBulk(FrameBulk {
                    console_command: Some("sensitivity 0;bxt_timer_reset;bxt_taslog".to_owned()),
                    ..FrameBulk::with_frame_time("0.001".to_owned())
                }),
                Line::FrameBulk(FrameBulk {
                    frame_count: NonZeroU32::new(5).unwrap(),
                    ..FrameBulk::with_frame_time("0.001".to_owned())
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
                    ..FrameBulk::with_frame_time("0.001".to_owned())
                }),
                Line::FrameBulk(FrameBulk {
                    frame_count: NonZeroU32::new(2951).unwrap(),
                    ..FrameBulk::with_frame_time("0.001".to_owned())
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
                    console_command: Some("bxt_timer_start".to_owned()),
                    ..FrameBulk::with_frame_time("0.001".to_owned())
                }),
                Line::Comment(" More frames because some of them get converted to 0ms".to_owned()),
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
                    ..FrameBulk::with_frame_time("0.001".to_owned())
                }),
                Line::FrameBulk(FrameBulk {
                    console_command: Some(
                        "stop;bxt_timer_stop;pause;sensitivity 1;_bxt_taslog 0;bxt_taslog;\
                         //condebug"
                            .to_owned(),
                    ),
                    ..FrameBulk::with_frame_time("0.001".to_owned())
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

    #[test]
    fn write_to_too_small_buffer() {
        let contents = read_to_string("test-data/parse/bhop.hltas").unwrap();
        let hltas = HLTAS::from_str(&contents).unwrap();

        let mut buf = [0; 4];
        assert!(matches!(
            hltas.to_writer(&mut buf[..]),
            Err(GenError::BufferTooSmall(_))
        ));
    }

    #[test]
    fn write_to_big_enough_buffer() {
        let contents = read_to_string("test-data/parse/bhop.hltas").unwrap();
        let hltas = HLTAS::from_str(&contents).unwrap();

        let mut buf = [0; 1024];
        let mut buf = &mut buf[..]; // Get the slice to have its len updated by Write.
        hltas.to_writer(&mut buf).unwrap();
        assert!(buf.len() < 1024);
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
    test_error! { error_lgagst_action_times, "lgagst-action-times", TimesOnLeaveGroundAction }
    test_error! {
        error_no_plus_minus_before_tolerance,
        "no-plus-minus-before-tolerance",
        NoPlusMinusBeforeTolerance
    }

    #[cfg(feature = "proptest1")]
    proptest! {
        #[test]
        fn write_parse(hltas: HLTAS) {
            let mut buffer = Vec::new();
            hltas.to_writer(&mut buffer).unwrap();
            let hltas_2 = HLTAS::from_str(from_utf8(&buffer).unwrap()).unwrap();
            prop_assert_eq!(hltas, hltas_2);
        }
    }
}
