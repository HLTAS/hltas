use std::{
    ffi::{CStr, CString},
    fs::{read_to_string, File},
    mem::zeroed,
    num::NonZeroU32,
    os::raw::{c_char, c_void},
};

use crate::{
    hltas_cpp::{
        self, hltas_input_get_frame, hltas_input_get_property, hltas_input_push_frame,
        hltas_input_set_error_message, hltas_input_set_property,
    },
    read::{self, properties::seeds},
    types::*,
    write,
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
            ConstantSpeed => Self::CONSTSPEED,
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
            CONSTSPEED => Self::ConstantSpeed,
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
        }
    }
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

                    let mut comments = String::new();
                    for line in hltas.lines {
                        match line {
                            Line::FrameBulk(frame_bulk) => {
                                let mut frame: hltas_cpp::hltas_frame = zeroed();
                                let comments_cstring = CString::new(comments).unwrap();
                                frame.Comments = comments_cstring.as_ptr();
                                comments = String::new();

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

                                if let Some(leave_ground_action) =
                                    frame_bulk.auto_actions.leave_ground_action
                                {
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
                                                frame.AutojumpTimes =
                                                    leave_ground_action.times.into();
                                            }
                                        }
                                        LeaveGroundActionType::DuckTap { zero_ms } => {
                                            frame.Ducktap = true;
                                            frame.Ducktap0ms = zero_ms;
                                            if !frame.Lgagst {
                                                frame.DucktapTimes =
                                                    leave_ground_action.times.into();
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

                                if let Some(DuckBeforeGround { times }) =
                                    frame_bulk.auto_actions.duck_before_ground
                                {
                                    frame.Dbg = true;
                                    frame.DbgTimes = times.into();
                                }

                                if let Some(DuckWhenJump { times }) =
                                    frame_bulk.auto_actions.duck_when_jump
                                {
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

                                let frametime = CString::new(frame_bulk.frame_time).unwrap();
                                frame.Frametime = frametime.as_ptr();

                                if let Some(pitch) = frame_bulk.pitch {
                                    frame.PitchPresent = true;
                                    frame.Pitch = f64::from(pitch);
                                }

                                frame.Repeats = frame_bulk.frame_count.get();

                                // So it doesn't go out of scope and de-allocate too early.
                                let console_command_cstring;
                                if let Some(console_command) = frame_bulk.console_command {
                                    console_command_cstring =
                                        CString::new(console_command).unwrap();
                                    frame.Commands = console_command_cstring.as_ptr();
                                }

                                hltas_input_push_frame(input, &frame);
                            }
                            Line::Save(save_name) => {
                                let mut frame: hltas_cpp::hltas_frame = zeroed();
                                let comments_cstring = CString::new(comments).unwrap();
                                frame.Comments = comments_cstring.as_ptr();
                                comments = String::new();

                                let save_name = CString::new(save_name).unwrap();
                                frame.SaveName = save_name.as_ptr();
                                hltas_input_push_frame(input, &frame);
                            }
                            Line::SharedSeed(seed) => {
                                let mut frame: hltas_cpp::hltas_frame = zeroed();
                                let comments_cstring = CString::new(comments).unwrap();
                                frame.Comments = comments_cstring.as_ptr();
                                comments = String::new();

                                frame.SeedPresent = true;
                                frame.Seed = seed;
                                hltas_input_push_frame(input, &frame);
                            }
                            Line::Buttons(buttons) => {
                                let mut frame: hltas_cpp::hltas_frame = zeroed();
                                let comments_cstring = CString::new(comments).unwrap();
                                frame.Comments = comments_cstring.as_ptr();
                                comments = String::new();

                                match buttons {
                                    Buttons::Reset => {
                                        frame.BtnState = hltas_cpp::ButtonState::CLEAR
                                    }
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
                                }
                                hltas_input_push_frame(input, &frame);
                            }
                            Line::LGAGSTMinSpeed(lgagst_min_speed) => {
                                let mut frame: hltas_cpp::hltas_frame = zeroed();
                                let comments_cstring = CString::new(comments).unwrap();
                                frame.Comments = comments_cstring.as_ptr();
                                comments = String::new();

                                frame.LgagstMinSpeedPresent = true;
                                frame.LgagstMinSpeed = lgagst_min_speed;
                                hltas_input_push_frame(input, &frame);
                            }
                            Line::Reset { non_shared_seed } => {
                                let mut frame: hltas_cpp::hltas_frame = zeroed();
                                let comments_cstring = CString::new(comments).unwrap();
                                frame.Comments = comments_cstring.as_ptr();
                                comments = String::new();

                                frame.ResetFrame = true;
                                frame.ResetNonSharedRNGSeed = non_shared_seed;
                                hltas_input_push_frame(input, &frame);
                            }
                            Line::Comment(comment) => {
                                comments.push_str(comment);
                                comments.push('\n');
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

            let mut hltas = HLTAS {
                properties: Properties {
                    demo,
                    save,
                    seeds,
                    frametime_0ms,
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

            if write::hltas(file, &hltas).is_err() {
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
