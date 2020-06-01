use std::{
    ffi::{CStr, CString},
    fmt::{self, Debug},
    fs::{read_to_string, File},
    mem::{zeroed, ManuallyDrop},
    num::NonZeroU32,
    os::raw::{c_char, c_void},
    str::FromStr,
};

use nom::{
    character::complete::{char, digit0, digit1, one_of, space1},
    combinator::{map, map_res, opt, recognize},
    sequence::{pair, separated_pair},
    IResult,
};

use hltas::{read, types::*};

#[allow(non_camel_case_types, non_snake_case, dead_code)]
pub mod hltas_cpp;
use hltas_cpp::{
    hltas_input_get_frame, hltas_input_get_property, hltas_input_push_frame,
    hltas_input_set_error_message, hltas_input_set_property,
};

impl From<Button> for hltas_cpp::Button {
    #[inline]
    fn from(x: Button) -> Self {
        use Button::*;
        match x {
            Forward => Self::FORWARD,
            ForwardLeft => Self::FORWARD_LEFT,
            Left => Self::LEFT,
            BackLeft => Self::BACK_LEFT,
            Back => Self::BACK,
            BackRight => Self::BACK_RIGHT,
            Right => Self::RIGHT,
            ForwardRight => Self::FORWARD_RIGHT,
        }
    }
}

impl From<hltas_cpp::Button> for Button {
    #[inline]
    fn from(x: hltas_cpp::Button) -> Self {
        use hltas_cpp::Button::*;
        match x {
            FORWARD => Self::Forward,
            FORWARD_LEFT => Self::ForwardLeft,
            LEFT => Self::Left,
            BACK_LEFT => Self::BackLeft,
            BACK => Self::Back,
            BACK_RIGHT => Self::BackRight,
            RIGHT => Self::Right,
            FORWARD_RIGHT => Self::ForwardRight,
        }
    }
}

impl From<StrafeType> for hltas_cpp::StrafeType {
    #[inline]
    fn from(x: StrafeType) -> Self {
        use StrafeType::*;
        match x {
            MaxAccel => Self::MAXACCEL,
            MaxAngle => Self::MAXANGLE,
            MaxDeccel => Self::MAXDECCEL,
            ConstSpeed => Self::CONSTSPEED,
        }
    }
}

impl From<hltas_cpp::StrafeType> for StrafeType {
    #[inline]
    fn from(x: hltas_cpp::StrafeType) -> Self {
        use hltas_cpp::StrafeType::*;
        match x {
            MAXACCEL => Self::MaxAccel,
            MAXANGLE => Self::MaxAngle,
            MAXDECCEL => Self::MaxDeccel,
            CONSTSPEED => Self::ConstSpeed,
        }
    }
}

impl From<read::Context> for hltas_cpp::ErrorCode {
    #[inline]
    fn from(x: read::Context) -> Self {
        use hltas_cpp::ErrorCode::*;
        use read::Context::*;
        match x {
            ErrorReadingVersion => FAILVER,
            VersionTooHigh => NOTSUPPORTED,
            BothAutoJumpAndDuckTap => BOTHAJDT,
            NoLeaveGroundAction => NOLGAGSTACTION,
            TimesOnLeaveGroundAction => LGAGSTACTIONTIMES,
            NoSaveName => NOSAVENAME,
            NoSeed => NOSEED,
            NoYaw => NOYAW,
            NoButtons => NOBUTTONS,
            NoLGAGSTMinSpeed => NOLGAGSTMINSPEED,
            NoResetSeed => NORESETSEED,
            ErrorParsingLine => FAILFRAME,
            InvalidStrafingAlgorithm => INVALID_ALGORITHM,
            NoConstraints => MISSING_CONSTRAINTS,
            NoPlusMinusBeforeTolerance => NO_PM_IN_TOLERANCE,
            NoFromToParameters => MISSING_ALGORITHM_FROMTO_PARAMETERS,
            NoTo => NO_TO_IN_FROMTO_ALGORITHM,
        }
    }
}

impl From<VectorialStrafingConstraints> for hltas_cpp::AlgorithmParameters {
    #[inline]
    fn from(x: VectorialStrafingConstraints) -> Self {
        use hltas_cpp::{
            AlgorithmParameters__bindgen_ty_1, AlgorithmParameters__bindgen_ty_1__bindgen_ty_1,
            AlgorithmParameters__bindgen_ty_1__bindgen_ty_2,
            AlgorithmParameters__bindgen_ty_1__bindgen_ty_3,
            AlgorithmParameters__bindgen_ty_1__bindgen_ty_4, ConstraintsType,
        };

        use VectorialStrafingConstraints::*;
        match x {
            VelocityYaw { tolerance } => Self {
                Type: ConstraintsType::VELOCITY,
                Parameters: AlgorithmParameters__bindgen_ty_1 {
                    Velocity: AlgorithmParameters__bindgen_ty_1__bindgen_ty_1 {
                        Constraints: tolerance as f64,
                    },
                },
            },
            AvgVelocityYaw { tolerance } => Self {
                Type: ConstraintsType::VELOCITY_AVG,
                Parameters: AlgorithmParameters__bindgen_ty_1 {
                    VelocityAvg: AlgorithmParameters__bindgen_ty_1__bindgen_ty_2 {
                        Constraints: tolerance as f64,
                    },
                },
            },
            Yaw { yaw, tolerance } => Self {
                Type: ConstraintsType::YAW,
                Parameters: AlgorithmParameters__bindgen_ty_1 {
                    Yaw: AlgorithmParameters__bindgen_ty_1__bindgen_ty_3 {
                        Yaw: yaw as f64,
                        Constraints: tolerance as f64,
                    },
                },
            },
            YawRange { from, to } => Self {
                Type: ConstraintsType::YAW_RANGE,
                Parameters: AlgorithmParameters__bindgen_ty_1 {
                    YawRange: AlgorithmParameters__bindgen_ty_1__bindgen_ty_4 {
                        LowestYaw: from as f64,
                        HighestYaw: to as f64,
                    },
                },
            },
        }
    }
}

impl From<hltas_cpp::AlgorithmParameters> for VectorialStrafingConstraints {
    #[inline]
    fn from(x: hltas_cpp::AlgorithmParameters) -> Self {
        use hltas_cpp::ConstraintsType;
        unsafe {
            match x.Type {
                ConstraintsType::VELOCITY => Self::VelocityYaw {
                    tolerance: x.Parameters.Velocity.Constraints as f32,
                },
                ConstraintsType::VELOCITY_AVG => Self::AvgVelocityYaw {
                    tolerance: x.Parameters.VelocityAvg.Constraints as f32,
                },
                ConstraintsType::YAW => Self::Yaw {
                    yaw: x.Parameters.Yaw.Yaw as f32,
                    tolerance: x.Parameters.Yaw.Constraints as f32,
                },
                ConstraintsType::YAW_RANGE => Self::YawRange {
                    from: x.Parameters.YawRange.LowestYaw as f32,
                    to: x.Parameters.YawRange.HighestYaw as f32,
                },
            }
        }
    }
}

impl From<ChangeTarget> for hltas_cpp::ChangeTarget {
    #[inline]
    fn from(x: ChangeTarget) -> Self {
        use ChangeTarget::*;
        match x {
            Yaw => Self::YAW,
            Pitch => Self::PITCH,
            VectorialStrafingYaw => Self::TARGET_YAW,
        }
    }
}

impl From<hltas_cpp::ChangeTarget> for ChangeTarget {
    #[inline]
    fn from(x: hltas_cpp::ChangeTarget) -> Self {
        use hltas_cpp::ChangeTarget::*;
        match x {
            YAW => Self::Yaw,
            PITCH => Self::Pitch,
            TARGET_YAW => Self::VectorialStrafingYaw,
        }
    }
}

impl Default for hltas_cpp::StrafingAlgorithm {
    #[inline]
    fn default() -> Self {
        Self::YAW
    }
}

impl Default for hltas_cpp::AlgorithmParameters {
    #[inline]
    fn default() -> Self {
        Self {
            Type: hltas_cpp::ConstraintsType::YAW,
            Parameters: hltas_cpp::AlgorithmParameters__bindgen_ty_1 {
                Yaw: hltas_cpp::AlgorithmParameters__bindgen_ty_1__bindgen_ty_3 {
                    Yaw: 0.,
                    Constraints: 180.,
                },
            },
        }
    }
}

impl Debug for hltas_cpp::AlgorithmParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("AlgorithmParameters");
        builder.field("Type", &self.Type);

        use hltas_cpp::ConstraintsType;
        let field: &dyn Debug = unsafe {
            match self.Type {
                ConstraintsType::VELOCITY => &self.Parameters.Velocity,
                ConstraintsType::VELOCITY_AVG => &self.Parameters.VelocityAvg,
                ConstraintsType::YAW => &self.Parameters.Yaw,
                ConstraintsType::YAW_RANGE => &self.Parameters.YawRange,
            }
        };
        builder.field("Parameters", field);

        builder.finish()
    }
}

impl Default for hltas_cpp::Button {
    #[inline]
    fn default() -> Self {
        Self::FORWARD
    }
}

impl Default for hltas_cpp::StrafeButtons {
    #[inline]
    fn default() -> Self {
        Self {
            AirLeft: hltas_cpp::Button::default(),
            AirRight: hltas_cpp::Button::default(),
            GroundLeft: hltas_cpp::Button::default(),
            GroundRight: hltas_cpp::Button::default(),
        }
    }
}

// Copied from hltas::read.
fn non_zero_u32(i: &str) -> IResult<&str, NonZeroU32> {
    map_res(
        recognize(pair(one_of("123456789"), digit0)),
        NonZeroU32::from_str,
    )(i)
}

// Three functions copied from hltas::read::properties.
fn shared_seed(i: &str) -> IResult<&str, u32> {
    map_res(digit1, u32::from_str)(i)
}

fn non_shared_seed(i: &str) -> IResult<&str, i64> {
    map_res(recognize(pair(opt(char('-')), digit1)), i64::from_str)(i)
}

fn seeds(i: &str) -> IResult<&str, Seeds> {
    map(
        separated_pair(shared_seed, space1, non_shared_seed),
        |(shared, non_shared)| Seeds { shared, non_shared },
    )(i)
}

/// Strings which a `hltas_frame` has pointers to.
#[derive(Default)]
pub struct AllocatedStrings {
    frame_time: Option<CString>,
    console_command: Option<CString>,
    save_name: Option<CString>,
}

/// Converts a non-comment line to a `hltas_frame`.
///
/// `AllocatedStrings` contains strings to which the pointers in `hltas_frame` point.
///
/// # Safety
///
/// `AllocatedStrings` must not be dropped before `hltas_frame`.
///
/// # Panics
///
/// Panics if `line` is `Line::Comment`.
pub unsafe fn hltas_frame_from_non_comment_line(
    line: &Line,
) -> (hltas_cpp::hltas_frame, ManuallyDrop<AllocatedStrings>) {
    let mut frame: hltas_cpp::hltas_frame = zeroed();
    let mut strings = AllocatedStrings::default();

    match line {
        Line::Comment(_) => panic!("can't convert a comment line"),
        Line::FrameBulk(frame_bulk) => {
            match frame_bulk.auto_actions.movement {
                Some(AutoMovement::SetYaw(yaw)) => {
                    frame.YawPresent = true;
                    frame.Yaw = f64::from(yaw);
                }
                Some(AutoMovement::Strafe(StrafeSettings { type_, dir })) => {
                    frame.Strafe = true;
                    frame.Type = type_.into();
                    match dir {
                        StrafeDir::Left => {
                            frame.Dir = hltas_cpp::StrafeDir::LEFT;
                        }
                        StrafeDir::Right => {
                            frame.Dir = hltas_cpp::StrafeDir::RIGHT;
                        }
                        StrafeDir::Best => {
                            frame.Dir = hltas_cpp::StrafeDir::BEST;
                        }
                        StrafeDir::Yaw(yaw) => {
                            frame.Dir = hltas_cpp::StrafeDir::YAW;
                            frame.YawPresent = true;
                            frame.Yaw = f64::from(yaw);
                        }
                        StrafeDir::Point { x, y } => {
                            frame.Dir = hltas_cpp::StrafeDir::POINT;
                            frame.YawPresent = true;
                            frame.X = f64::from(x);
                            frame.Y = f64::from(y);
                        }
                        StrafeDir::Line { yaw } => {
                            frame.Dir = hltas_cpp::StrafeDir::LINE;
                            frame.YawPresent = true;
                            frame.Yaw = f64::from(yaw);
                        }
                    }
                }
                None => {}
            }

            if let Some(leave_ground_action) = frame_bulk.auto_actions.leave_ground_action {
                match leave_ground_action.speed {
                    LeaveGroundActionSpeed::Optimal => frame.Lgagst = true,
                    LeaveGroundActionSpeed::OptimalWithFullMaxspeed => {
                        frame.Lgagst = true;
                        frame.LgagstFullMaxspeed = true;
                    }
                    LeaveGroundActionSpeed::Any => {}
                }

                if frame.Lgagst {
                    frame.LgagstTimes = leave_ground_action.times.into();
                }

                match leave_ground_action.type_ {
                    LeaveGroundActionType::Jump => {
                        frame.Autojump = true;
                        if !frame.Lgagst {
                            frame.AutojumpTimes = leave_ground_action.times.into();
                        }
                    }
                    LeaveGroundActionType::DuckTap { zero_ms } => {
                        frame.Ducktap = true;
                        frame.Ducktap0ms = zero_ms;
                        if !frame.Lgagst {
                            frame.DucktapTimes = leave_ground_action.times.into();
                        }
                    }
                }
            }

            if let Some(JumpBug { times }) = frame_bulk.auto_actions.jump_bug {
                frame.Jumpbug = true;
                frame.JumpbugTimes = times.into();
            }

            if let Some(DuckBeforeCollision {
                including_ceilings,
                times,
            }) = frame_bulk.auto_actions.duck_before_collision
            {
                frame.Dbc = true;
                frame.DbcCeilings = including_ceilings;
                frame.DbcTimes = times.into();
            }

            if let Some(DuckBeforeGround { times }) = frame_bulk.auto_actions.duck_before_ground {
                frame.Dbg = true;
                frame.DbgTimes = times.into();
            }

            if let Some(DuckWhenJump { times }) = frame_bulk.auto_actions.duck_when_jump {
                frame.Dwj = true;
                frame.DwjTimes = times.into();
            }

            frame.Forward = frame_bulk.movement_keys.forward;
            frame.Left = frame_bulk.movement_keys.left;
            frame.Right = frame_bulk.movement_keys.right;
            frame.Back = frame_bulk.movement_keys.back;
            frame.Up = frame_bulk.movement_keys.up;
            frame.Down = frame_bulk.movement_keys.down;

            frame.Jump = frame_bulk.action_keys.jump;
            frame.Duck = frame_bulk.action_keys.duck;
            frame.Use = frame_bulk.action_keys.use_;
            frame.Attack1 = frame_bulk.action_keys.attack_1;
            frame.Attack2 = frame_bulk.action_keys.attack_2;
            frame.Reload = frame_bulk.action_keys.reload;

            let frame_time = CString::new(frame_bulk.frame_time).unwrap();
            frame.Frametime = frame_time.as_ptr();
            strings.frame_time = Some(frame_time);

            if let Some(pitch) = frame_bulk.pitch {
                frame.PitchPresent = true;
                frame.Pitch = f64::from(pitch);
            }

            frame.Repeats = frame_bulk.frame_count.get();

            if let Some(console_command) = frame_bulk.console_command {
                let console_command_cstring = CString::new(console_command).unwrap();
                frame.Commands = console_command_cstring.as_ptr();
                strings.console_command = Some(console_command_cstring);
            }
        }
        Line::Save(save_name) => {
            let save_name = CString::new(*save_name).unwrap();
            frame.SaveName = save_name.as_ptr();
            strings.save_name = Some(save_name);
        }
        Line::SharedSeed(seed) => {
            frame.SeedPresent = true;
            frame.Seed = *seed;
        }
        Line::Buttons(buttons) => match *buttons {
            Buttons::Reset => frame.BtnState = hltas_cpp::ButtonState::CLEAR,
            Buttons::Set {
                air_left,
                air_right,
                ground_left,
                ground_right,
            } => {
                frame.BtnState = hltas_cpp::ButtonState::SET;
                frame.Buttons.AirLeft = air_left.into();
                frame.Buttons.AirRight = air_right.into();
                frame.Buttons.GroundLeft = ground_left.into();
                frame.Buttons.GroundRight = ground_right.into();
            }
        },
        Line::LGAGSTMinSpeed(lgagst_min_speed) => {
            frame.LgagstMinSpeedPresent = true;
            frame.LgagstMinSpeed = *lgagst_min_speed;
        }
        Line::Reset { non_shared_seed } => {
            frame.ResetFrame = true;
            frame.ResetNonSharedRNGSeed = *non_shared_seed;
        }
        Line::VectorialStrafing(enabled) => {
            frame.StrafingAlgorithmPresent = true;
            frame.Algorithm = if *enabled {
                hltas_cpp::StrafingAlgorithm::VECTORIAL
            } else {
                hltas_cpp::StrafingAlgorithm::YAW
            };
        }
        Line::VectorialStrafingConstraints(constraints) => {
            frame.AlgorithmParametersPresent = true;
            frame.Parameters = (*constraints).into();
        }
        Line::Change(change) => {
            frame.ChangePresent = true;
            frame.Target = change.target.into();
            frame.ChangeFinalValue = change.final_value;
            frame.ChangeOver = change.over;
        }
    }

    (frame, ManuallyDrop::new(strings))
}

/// Reads the HLTAS from `filename` and writes it into `input`.
///
/// This is meant to be used internally from the C++ HLTAS library.
///
/// # Safety
///
/// `input` must be a valid `HLTAS::Input`, `filename` must be a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn hltas_rs_read(
    input: *mut c_void,
    filename: *const c_char,
) -> hltas_cpp::ErrorDescription {
    if let Ok(filename) = CStr::from_ptr(filename).to_str() {
        if let Ok(contents) = read_to_string(filename) {
            match HLTAS::from_str(&contents) {
                Ok(hltas) => {
                    if let Some(demo) = hltas.properties.demo {
                        let demo = CString::new(demo).unwrap();
                        hltas_input_set_property(
                            input,
                            b"demo\0" as *const u8 as *const c_char,
                            demo.as_ptr(),
                        );
                    }
                    if let Some(save) = hltas.properties.save {
                        let save = CString::new(save).unwrap();
                        hltas_input_set_property(
                            input,
                            b"save\0" as *const u8 as *const c_char,
                            save.as_ptr(),
                        );
                    }
                    if let Some(frametime_0ms) = hltas.properties.frametime_0ms {
                        let frametime_0ms = CString::new(frametime_0ms).unwrap();
                        hltas_input_set_property(
                            input,
                            b"frametime0ms\0" as *const u8 as *const c_char,
                            frametime_0ms.as_ptr(),
                        );
                    }
                    if let Some(seeds) = hltas.properties.seeds {
                        let seeds = format!("{} {}", seeds.shared, seeds.non_shared);
                        let seeds = CString::new(seeds).unwrap();
                        hltas_input_set_property(
                            input,
                            b"seed\0" as *const u8 as *const c_char,
                            seeds.as_ptr(),
                        );
                    }
                    if let Some(hlstrafe_version) = hltas.properties.hlstrafe_version {
                        let hlstrafe_version = CString::new(hlstrafe_version.to_string()).unwrap();
                        hltas_input_set_property(
                            input,
                            b"hlstrafe_version\0" as *const u8 as *const c_char,
                            hlstrafe_version.as_ptr(),
                        );
                    }

                    let mut comments = String::new();
                    for line in hltas.lines {
                        match line {
                            Line::Comment(comment) => {
                                comments.push_str(comment);
                                comments.push('\n');
                            }
                            line => {
                                let (mut frame, mut strings) =
                                    hltas_frame_from_non_comment_line(&line);

                                let comments_cstring = CString::new(comments).unwrap();
                                frame.Comments = comments_cstring.as_ptr();
                                comments = String::new();

                                hltas_input_push_frame(input, &frame);
                                ManuallyDrop::drop(&mut strings);
                            }
                        }
                    }
                    hltas_cpp::ErrorDescription {
                        Code: hltas_cpp::ErrorCode::OK,
                        LineNumber: 0,
                    }
                }
                Err(error) => {
                    let code = error
                        .context
                        .map(hltas_cpp::ErrorCode::from)
                        .unwrap_or(hltas_cpp::ErrorCode::FAILLINE);

                    let message = format!("{}", error);
                    if let Ok(message) = CString::new(message) {
                        hltas_input_set_error_message(input, message.as_ptr());
                    }

                    hltas_cpp::ErrorDescription {
                        Code: code,
                        LineNumber: error.line() as u32,
                    }
                }
            }
        } else {
            hltas_cpp::ErrorDescription {
                Code: hltas_cpp::ErrorCode::FAILOPEN,
                LineNumber: 0,
            }
        }
    } else {
        hltas_cpp::ErrorDescription {
            Code: hltas_cpp::ErrorCode::FAILOPEN,
            LineNumber: 0,
        }
    }
}

/// Writes the HLTAS from `input` to `filename`.
///
/// This is meant to be used internally from the C++ HLTAS library.
///
/// # Safety
///
/// `input` must be a valid `HLTAS::Input`, `filename` must be a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn hltas_rs_write(
    input: *const c_void,
    filename: *const c_char,
) -> hltas_cpp::ErrorDescription {
    if let Ok(filename) = CStr::from_ptr(filename).to_str() {
        if let Ok(file) = File::create(filename) {
            let demo = hltas_input_get_property(input, b"demo\0" as *const u8 as *const c_char);
            let demo = if demo.is_null() {
                None
            } else if let Ok(demo) = CStr::from_ptr(demo).to_str() {
                Some(demo)
            } else {
                return hltas_cpp::ErrorDescription {
                    Code: hltas_cpp::ErrorCode::FAILWRITE,
                    LineNumber: 0,
                };
            };

            let save = hltas_input_get_property(input, b"save\0" as *const u8 as *const c_char);
            let save = if save.is_null() {
                None
            } else if let Ok(save) = CStr::from_ptr(save).to_str() {
                Some(save)
            } else {
                return hltas_cpp::ErrorDescription {
                    Code: hltas_cpp::ErrorCode::FAILWRITE,
                    LineNumber: 0,
                };
            };

            let seed = hltas_input_get_property(input, b"seed\0" as *const u8 as *const c_char);
            let seeds = if seed.is_null() {
                None
            } else if let Ok(seed) = CStr::from_ptr(seed).to_str() {
                if let Ok((_, seeds)) = seeds(seed) {
                    Some(seeds)
                } else {
                    return hltas_cpp::ErrorDescription {
                        Code: hltas_cpp::ErrorCode::FAILWRITE,
                        LineNumber: 0,
                    };
                }
            } else {
                return hltas_cpp::ErrorDescription {
                    Code: hltas_cpp::ErrorCode::FAILWRITE,
                    LineNumber: 0,
                };
            };

            let frametime_0ms =
                hltas_input_get_property(input, b"frametime0ms\0" as *const u8 as *const c_char);
            let frametime_0ms = if frametime_0ms.is_null() {
                None
            } else if let Ok(frametime_0ms) = CStr::from_ptr(frametime_0ms).to_str() {
                Some(frametime_0ms)
            } else {
                return hltas_cpp::ErrorDescription {
                    Code: hltas_cpp::ErrorCode::FAILWRITE,
                    LineNumber: 0,
                };
            };

            let hlstrafe_version = hltas_input_get_property(
                input,
                b"hlstrafe_version\0" as *const u8 as *const c_char,
            );
            let hlstrafe_version = if hlstrafe_version.is_null() {
                None
            } else if let Ok(hlstrafe_version) = CStr::from_ptr(hlstrafe_version).to_str() {
                if let Ok((_, hlstrafe_version)) = non_zero_u32(hlstrafe_version) {
                    Some(hlstrafe_version)
                } else {
                    return hltas_cpp::ErrorDescription {
                        Code: hltas_cpp::ErrorCode::FAILWRITE,
                        LineNumber: 0,
                    };
                }
            } else {
                return hltas_cpp::ErrorDescription {
                    Code: hltas_cpp::ErrorCode::FAILWRITE,
                    LineNumber: 0,
                };
            };

            let mut hltas = HLTAS {
                properties: Properties {
                    demo,
                    save,
                    seeds,
                    frametime_0ms,
                    hlstrafe_version,
                },
                lines: Vec::new(),
            };

            let mut index = 0;
            loop {
                let mut frame = zeroed();
                if hltas_input_get_frame(input, index, &mut frame) != 0 {
                    break;
                }
                index += 1;

                if !frame.Comments.is_null() {
                    let comments = if let Ok(comments) = CStr::from_ptr(frame.Comments).to_str() {
                        comments
                    } else {
                        return hltas_cpp::ErrorDescription {
                            Code: hltas_cpp::ErrorCode::FAILWRITE,
                            LineNumber: 0,
                        };
                    };

                    for line in comments.lines() {
                        hltas.lines.push(Line::Comment(line));
                    }
                }

                if !frame.SaveName.is_null() {
                    let save = if let Ok(save) = CStr::from_ptr(frame.SaveName).to_str() {
                        save
                    } else {
                        return hltas_cpp::ErrorDescription {
                            Code: hltas_cpp::ErrorCode::FAILWRITE,
                            LineNumber: 0,
                        };
                    };

                    hltas.lines.push(Line::Save(save));
                    continue;
                }

                if frame.SeedPresent {
                    hltas.lines.push(Line::SharedSeed(frame.Seed));
                    continue;
                }

                if frame.BtnState != hltas_cpp::ButtonState::NOTHING {
                    let line = if frame.BtnState == hltas_cpp::ButtonState::SET {
                        Line::Buttons(Buttons::Set {
                            air_left: frame.Buttons.AirLeft.into(),
                            air_right: frame.Buttons.AirRight.into(),
                            ground_left: frame.Buttons.GroundLeft.into(),
                            ground_right: frame.Buttons.GroundRight.into(),
                        })
                    } else {
                        Line::Buttons(Buttons::Reset)
                    };

                    hltas.lines.push(line);
                    continue;
                }

                if frame.LgagstMinSpeedPresent {
                    hltas.lines.push(Line::LGAGSTMinSpeed(frame.LgagstMinSpeed));
                    continue;
                }

                if frame.ResetFrame {
                    hltas.lines.push(Line::Reset {
                        non_shared_seed: frame.ResetNonSharedRNGSeed,
                    });
                    continue;
                }

                if frame.StrafingAlgorithmPresent {
                    hltas.lines.push(Line::VectorialStrafing(
                        frame.Algorithm == hltas_cpp::StrafingAlgorithm::VECTORIAL,
                    ));
                    continue;
                }

                if frame.AlgorithmParametersPresent {
                    hltas
                        .lines
                        .push(Line::VectorialStrafingConstraints(frame.Parameters.into()));

                    continue;
                }

                if frame.ChangePresent {
                    hltas.lines.push(Line::Change(Change {
                        target: frame.Target.into(),
                        final_value: frame.ChangeFinalValue,
                        over: frame.ChangeOver,
                    }));

                    continue;
                }

                let movement = if frame.Strafe {
                    use hltas_cpp::StrafeDir::*;
                    Some(AutoMovement::Strafe(StrafeSettings {
                        type_: frame.Type.into(),
                        dir: match frame.Dir {
                            LEFT => StrafeDir::Left,
                            RIGHT => StrafeDir::Right,
                            BEST => StrafeDir::Best,
                            YAW => StrafeDir::Yaw(frame.Yaw as f32),
                            POINT => StrafeDir::Point {
                                x: frame.X as f32,
                                y: frame.Y as f32,
                            },
                            LINE => StrafeDir::Line {
                                yaw: frame.Yaw as f32,
                            },
                        },
                    }))
                } else if frame.YawPresent {
                    Some(AutoMovement::SetYaw(frame.Yaw as f32))
                } else {
                    None
                };

                let leave_ground_action = if frame.Lgagst {
                    let speed = if frame.LgagstFullMaxspeed {
                        LeaveGroundActionSpeed::OptimalWithFullMaxspeed
                    } else {
                        LeaveGroundActionSpeed::Optimal
                    };

                    if frame.Autojump {
                        Some(LeaveGroundAction {
                            speed,
                            times: frame.LgagstTimes.into(),
                            type_: LeaveGroundActionType::Jump,
                        })
                    } else {
                        Some(LeaveGroundAction {
                            speed,
                            times: frame.LgagstTimes.into(),
                            type_: LeaveGroundActionType::DuckTap {
                                zero_ms: frame.Ducktap0ms,
                            },
                        })
                    }
                } else if frame.Autojump {
                    Some(LeaveGroundAction {
                        speed: LeaveGroundActionSpeed::Any,
                        times: frame.AutojumpTimes.into(),
                        type_: LeaveGroundActionType::Jump,
                    })
                } else if frame.Ducktap {
                    Some(LeaveGroundAction {
                        speed: LeaveGroundActionSpeed::Any,
                        times: frame.DucktapTimes.into(),
                        type_: LeaveGroundActionType::DuckTap {
                            zero_ms: frame.Ducktap0ms,
                        },
                    })
                } else {
                    None
                };

                let jump_bug = if frame.Jumpbug {
                    Some(JumpBug {
                        times: frame.JumpbugTimes.into(),
                    })
                } else {
                    None
                };

                let duck_before_collision = if frame.Dbc {
                    Some(DuckBeforeCollision {
                        times: frame.DbcTimes.into(),
                        including_ceilings: frame.DbcCeilings,
                    })
                } else {
                    None
                };

                let duck_before_ground = if frame.Dbg {
                    Some(DuckBeforeGround {
                        times: frame.DbgTimes.into(),
                    })
                } else {
                    None
                };

                let duck_when_jump = if frame.Dwj {
                    Some(DuckWhenJump {
                        times: frame.DwjTimes.into(),
                    })
                } else {
                    None
                };

                let forward = frame.Forward;
                let left = frame.Left;
                let right = frame.Right;
                let back = frame.Back;
                let up = frame.Up;
                let down = frame.Down;

                let jump = frame.Jump;
                let duck = frame.Duck;
                let use_ = frame.Use;
                let attack_1 = frame.Attack1;
                let attack_2 = frame.Attack2;
                let reload = frame.Reload;

                let frame_time = if let Ok(frametime) = CStr::from_ptr(frame.Frametime).to_str() {
                    frametime
                } else {
                    return hltas_cpp::ErrorDescription {
                        Code: hltas_cpp::ErrorCode::FAILWRITE,
                        LineNumber: 0,
                    };
                };

                let pitch = if frame.PitchPresent {
                    Some(frame.Pitch as f32)
                } else {
                    None
                };

                let frame_count = if let Some(frame_count) = NonZeroU32::new(frame.Repeats) {
                    frame_count
                } else {
                    return hltas_cpp::ErrorDescription {
                        Code: hltas_cpp::ErrorCode::FAILWRITE,
                        LineNumber: 0,
                    };
                };

                let console_command = if frame.Commands.is_null() {
                    None
                } else if let Ok(commands) = CStr::from_ptr(frame.Commands).to_str() {
                    Some(commands)
                } else {
                    return hltas_cpp::ErrorDescription {
                        Code: hltas_cpp::ErrorCode::FAILWRITE,
                        LineNumber: 0,
                    };
                };

                let frame_bulk = FrameBulk {
                    auto_actions: AutoActions {
                        movement,
                        leave_ground_action,
                        jump_bug,
                        duck_before_collision,
                        duck_before_ground,
                        duck_when_jump,
                    },
                    movement_keys: MovementKeys {
                        forward,
                        left,
                        right,
                        back,
                        up,
                        down,
                    },
                    action_keys: ActionKeys {
                        jump,
                        duck,
                        use_,
                        attack_1,
                        attack_2,
                        reload,
                    },
                    frame_time,
                    pitch,
                    frame_count,
                    console_command,
                };

                hltas.lines.push(Line::FrameBulk(frame_bulk));
            }

            if hltas.to_writer(file).is_err() {
                hltas_cpp::ErrorDescription {
                    Code: hltas_cpp::ErrorCode::FAILWRITE,
                    LineNumber: 0,
                }
            } else {
                hltas_cpp::ErrorDescription {
                    Code: hltas_cpp::ErrorCode::OK,
                    LineNumber: 0,
                }
            }
        } else {
            hltas_cpp::ErrorDescription {
                Code: hltas_cpp::ErrorCode::FAILOPEN,
                LineNumber: 0,
            }
        }
    } else {
        hltas_cpp::ErrorDescription {
            Code: hltas_cpp::ErrorCode::FAILOPEN,
            LineNumber: 0,
        }
    }
}
