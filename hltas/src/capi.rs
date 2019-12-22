use std::{
    ffi::{CStr, CString},
    fs::read_to_string,
    mem::zeroed,
    os::raw::{c_char, c_void},
};

use crate::{
    hltas_cpp::{self, hltas_input_push_frame, hltas_input_set_property},
    read::hltas,
    types::*,
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
            match hltas(&contents) {
                Ok((_, hltas)) => {
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

                                match frame_bulk.auto_actions.yaw_adjustment {
                                    Some(YawAdjustment::Set(yaw)) => {
                                        frame.YawPresent = true;
                                        frame.Yaw = f64::from(yaw);
                                    }
                                    Some(YawAdjustment::Strafe(StrafeSettings { type_, dir })) => {
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
                                        frame.LgagstTimes = leave_ground_action.times;
                                    }

                                    match leave_ground_action.type_ {
                                        LeaveGroundActionType::Jump => {
                                            frame.Autojump = true;
                                            if !frame.Lgagst {
                                                frame.AutojumpTimes = leave_ground_action.times;
                                            }
                                        }
                                        LeaveGroundActionType::DuckTap { zero_ms } => {
                                            frame.Ducktap = true;
                                            frame.Ducktap0ms = zero_ms;
                                            if !frame.Lgagst {
                                                frame.DucktapTimes = leave_ground_action.times;
                                            }
                                        }
                                    }
                                }

                                if let Some(JumpBug { times }) = frame_bulk.auto_actions.jump_bug {
                                    frame.Jumpbug = true;
                                    frame.JumpbugTimes = times;
                                }

                                if let Some(DuckBeforeCollision {
                                    including_ceilings,
                                    times,
                                }) = frame_bulk.auto_actions.duck_before_collision
                                {
                                    frame.Dbc = true;
                                    frame.DbcCeilings = including_ceilings;
                                    frame.DbcTimes = times;
                                }

                                if let Some(DuckBeforeGround { times }) =
                                    frame_bulk.auto_actions.duck_before_ground
                                {
                                    frame.Dbg = true;
                                    frame.DbgTimes = times;
                                }

                                if let Some(DuckWhenJump { times }) =
                                    frame_bulk.auto_actions.duck_when_jump
                                {
                                    frame.Dwj = true;
                                    frame.DwjTimes = times;
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
                Err(_) => hltas_cpp::ErrorDescription {
                    Code: hltas_cpp::ErrorCode::FAILLINE,
                    LineNumber: 0,
                },
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
