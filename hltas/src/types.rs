use std::num::NonZeroU32;

use nom;

use crate::read;

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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoActions {
    /// Yaw angle adjustment and strafing.
    pub yaw_adjustment: Option<YawAdjustment>,
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
pub enum YawAdjustment {
    /// Set the yaw angle to this value.
    Set(f32),
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
    ConstantSpeed,
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

/// Leave the ground automatically.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LeaveGroundAction {
    /// Speed at which to leave the ground.
    pub speed: LeaveGroundActionSpeed,
    /// Number of times to do the action. `0` means unlimited.
    pub times: u32,
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
    pub times: u32,
}

/// Duck-before-collision properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DuckBeforeCollision {
    /// Number of times to do the action. `0` means unlimited.
    pub times: u32,
    pub including_ceilings: bool,
}

/// Duck-before-ground properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DuckBeforeGround {
    /// Number of times to do the action. `0` means unlimited.
    pub times: u32,
}

/// Duck-when-jump properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DuckWhenJump {
    /// Number of times to do the action. `0` means unlimited.
    pub times: u32,
}

/// Manually specified movement keys.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    /// # extern crate hltas_rs;
    /// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::fs::read_to_string;
    /// use hltas_rs::HLTAS;
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
}
