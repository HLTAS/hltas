//! Writing `.hltas` files.

use std::{fmt::Display, io::Write, num::NonZeroU32};

use cookie_factory::{
    combinator::string,
    gen_simple,
    multi::many_ref,
    sequence::{pair, tuple},
    GenError, SerializeFn, WriteContext,
};

use crate::types::*;

fn property<S: AsRef<str>, W: Write>(name: S, value: impl SerializeFn<W>) -> impl SerializeFn<W> {
    tuple((string(name), string(" "), value))
}

fn display<T: Display, W: Write>(data: T) -> impl SerializeFn<W> {
    move |mut out: WriteContext<W>| match write!(out, "{}", data) {
        Err(io) => Err(GenError::IoError(io)),
        Ok(()) => Ok(out),
    }
}

fn strafe_type<W: Write>(type_: StrafeType) -> impl SerializeFn<W> {
    use StrafeType::*;
    match type_ {
        MaxAccel => string("0"),
        MaxAngle => string("1"),
        MaxDeccel => string("2"),
        ConstSpeed => string("3"),
    }
}

fn strafe_dir<W: Write>(dir: StrafeDir) -> impl SerializeFn<W> {
    use StrafeDir::*;
    match dir {
        Left => string("0"),
        Right => string("1"),
        Best => string("2"),
        Yaw(_) => string("3"),
        Point { .. } => string("4"),
        Line { .. } => string("5"),
        LeftRight(_) => string("6"),
        RightLeft(_) => string("7"),
    }
}

fn strafe<W: Write>(settings: StrafeSettings) -> impl SerializeFn<W> {
    tuple((
        string("s"),
        strafe_type(settings.type_),
        strafe_dir(settings.dir),
    ))
}

/// Prints `StrafeSettings` into writer.
///
/// # Examples
///
/// ```
/// # extern crate hltas;
/// use hltas::types::{StrafeDir, StrafeSettings, StrafeType};
///
/// let settings = StrafeSettings {
///     type_: StrafeType::MaxAngle,
///     dir: StrafeDir::Left,
/// };
/// let mut buf = Vec::new();
/// hltas::write::gen_strafe(&mut buf, settings).unwrap();
/// assert_eq!(buf, b"s10");
/// ```
pub fn gen_strafe<W: Write>(w: W, value: StrafeSettings) -> Result<(), GenError> {
    let _ = gen_simple(strafe(value), w)?;
    Ok(())
}

fn gen_times<W: Write>(times: Times) -> impl SerializeFn<W> {
    move |out: WriteContext<W>| {
        if let Times::Limited(times) = times {
            display(times)(out)
        } else {
            Ok(out)
        }
    }
}

fn auto_actions<W: Write>(aa: &AutoActions) -> impl SerializeFn<W> + '_ {
    move |out: WriteContext<W>| {
        let out = match aa.movement {
            None | Some(AutoMovement::SetYaw(_)) => string("---")(out)?,
            Some(AutoMovement::Strafe(settings)) => strafe(settings)(out)?,
        };

        let out = match aa.leave_ground_action {
            None => string("---")(out)?,
            Some(LeaveGroundAction {
                speed,
                times,
                type_,
            }) => {
                use LeaveGroundActionType::*;
                let gen_type = match type_ {
                    Jump => string("j-"),
                    DuckTap { zero_ms: false } => string("-d"),
                    DuckTap { zero_ms: true } => string("-D"),
                };
                let gen_type_with_times = move |out: WriteContext<W>| match type_ {
                    Jump => tuple((string("j"), gen_times(times), string("-")))(out),
                    DuckTap { zero_ms: false } => pair(string("-d"), gen_times(times))(out),
                    DuckTap { zero_ms: true } => pair(string("-D"), gen_times(times))(out),
                };

                use LeaveGroundActionSpeed::*;
                match speed {
                    Any => pair(string("-"), gen_type_with_times)(out)?,
                    Optimal => tuple((string("l"), gen_times(times), gen_type))(out)?,
                    OptimalWithFullMaxspeed => {
                        tuple((string("L"), gen_times(times), gen_type))(out)?
                    }
                }
            }
        };

        let out = match aa.jump_bug {
            None => string("-")(out)?,
            Some(JumpBug { times }) => pair(string("b"), gen_times(times))(out)?,
        };

        let out = match aa.duck_before_collision {
            None => string("-")(out)?,
            Some(DuckBeforeCollision {
                times,
                including_ceilings,
            }) => pair(
                string(if including_ceilings { "C" } else { "c" }),
                gen_times(times),
            )(out)?,
        };

        let out = match aa.duck_before_ground {
            None => string("-")(out)?,
            Some(DuckBeforeGround { times }) => pair(string("g"), gen_times(times))(out)?,
        };

        let out = match aa.duck_when_jump {
            None => string("-")(out)?,
            Some(DuckWhenJump { times }) => pair(string("w"), gen_times(times))(out)?,
        };

        Ok(out)
    }
}

fn key<W: Write>(symbol: &str, enabled: bool) -> impl SerializeFn<W> + '_ {
    move |out: WriteContext<W>| string(if enabled { symbol } else { "-" })(out)
}

fn movement_keys<W: Write>(mk: MovementKeys) -> impl SerializeFn<W> {
    tuple((
        key("f", mk.forward),
        key("l", mk.left),
        key("r", mk.right),
        key("b", mk.back),
        key("u", mk.up),
        key("d", mk.down),
    ))
}

fn action_keys<W: Write>(ak: ActionKeys) -> impl SerializeFn<W> {
    tuple((
        key("j", ak.jump),
        key("d", ak.duck),
        key("u", ak.use_),
        key("1", ak.attack_1),
        key("2", ak.attack_2),
        key("r", ak.reload),
    ))
}

fn yaw_field<W: Write>(movement: &Option<AutoMovement>) -> impl SerializeFn<W> + '_ {
    move |out: WriteContext<W>| match movement {
        None => string("-")(out),
        Some(AutoMovement::SetYaw(yaw)) => display(yaw)(out),
        Some(AutoMovement::Strafe(StrafeSettings { dir, .. })) => match dir {
            StrafeDir::Yaw(yaw) => display(yaw)(out),
            StrafeDir::Point { x, y } => tuple((display(x), string(" "), display(y)))(out),
            StrafeDir::Line { yaw } => display(yaw)(out),
            StrafeDir::LeftRight(count) | StrafeDir::RightLeft(count) => display(count)(out),
            _ => string("-")(out),
        },
    }
}

fn line_frame_bulk<W: Write>(frame_bulk: &FrameBulk) -> impl SerializeFn<W> + '_ {
    move |out: WriteContext<W>| {
        let out = auto_actions(&frame_bulk.auto_actions)(out)?;
        let out = string("|")(out)?;
        let out = movement_keys(frame_bulk.movement_keys)(out)?;
        let out = string("|")(out)?;
        let out = action_keys(frame_bulk.action_keys)(out)?;
        let out = string("|")(out)?;
        let out = string(&frame_bulk.frame_time)(out)?;
        let out = string("|")(out)?;
        let out = yaw_field(&frame_bulk.auto_actions.movement)(out)?;

        let out = string("|")(out)?;
        let out = if let Some(pitch) = frame_bulk.pitch {
            display(pitch)(out)?
        } else {
            string("-")(out)?
        };

        let out = string("|")(out)?;
        let out = display(frame_bulk.frame_count)(out)?;

        let out = if let Some(console_command) = frame_bulk.console_command.as_deref() {
            pair(string("|"), string(console_command))(out)?
        } else {
            out
        };

        Ok(out)
    }
}

/// Prints `FrameBulk` into writer.
///
/// # Examples
///
/// ```
/// # extern crate hltas;
/// use hltas::types::FrameBulk;
///
/// let mut frame_bulk = FrameBulk::with_frame_time("0.001".to_owned());
/// frame_bulk.movement_keys.left = true;
/// frame_bulk.action_keys.jump = true;
///
/// let mut buf = Vec::new();
/// hltas::write::gen_frame_bulk(&mut buf, &frame_bulk).unwrap();
/// assert_eq!(buf, b"----------|-l----|j-----|0.001|-|-|1");
/// ```
pub fn gen_frame_bulk<W: Write>(w: W, value: &FrameBulk) -> Result<(), GenError> {
    let _ = gen_simple(line_frame_bulk(value), w)?;
    Ok(())
}

fn button<W: Write>(button: Button) -> impl SerializeFn<W> {
    use Button::*;
    match button {
        Forward => string("0"),
        ForwardLeft => string("1"),
        Left => string("2"),
        BackLeft => string("3"),
        Back => string("4"),
        BackRight => string("5"),
        Right => string("6"),
        ForwardRight => string("7"),
    }
}

fn line_buttons<W: Write>(buttons: Buttons) -> impl SerializeFn<W> {
    move |out: WriteContext<W>| match buttons {
        Buttons::Reset => string("buttons")(out),
        Buttons::Set {
            air_left,
            air_right,
            ground_left,
            ground_right,
        } => {
            let sp_button = |&b| pair(string(" "), button(b));
            pair(
                string("buttons"),
                many_ref(&[air_left, air_right, ground_left, ground_right], sp_button),
            )(out)
        }
    }
}

fn gen_tolerance<W: Write>(tolerance: f32) -> impl SerializeFn<W> {
    move |out: WriteContext<W>| {
        if tolerance != 0. {
            pair(string(" +-"), display(tolerance))(out)
        } else {
            Ok(out)
        }
    }
}

fn gen_entity_index<W: Write>(entity: Option<NonZeroU32>) -> impl SerializeFn<W> {
    move |out: WriteContext<W>| match entity {
        Some(number) => pair(string(" entity "), display(number.get()))(out),
        None => Ok(out),
    }
}

fn line_vectorial_strafing_constraints<W: Write>(
    constraints: VectorialStrafingConstraints,
) -> impl SerializeFn<W> {
    move |out: WriteContext<W>| match constraints {
        VectorialStrafingConstraints::VelocityYaw { tolerance } => property(
            "target_yaw",
            pair(string("velocity"), gen_tolerance(tolerance)),
        )(out),
        VectorialStrafingConstraints::AvgVelocityYaw { tolerance } => property(
            "target_yaw",
            pair(string("velocity_avg"), gen_tolerance(tolerance)),
        )(out),
        VectorialStrafingConstraints::VelocityYawLocking { tolerance } => property(
            "target_yaw",
            pair(string("velocity_lock"), gen_tolerance(tolerance)),
        )(out),
        VectorialStrafingConstraints::Yaw { yaw, tolerance } => {
            property("target_yaw", pair(display(yaw), gen_tolerance(tolerance)))(out)
        }
        VectorialStrafingConstraints::YawRange { from, to } => property(
            "target_yaw",
            tuple((string("from "), display(from), string(" to "), display(to))),
        )(out),
        VectorialStrafingConstraints::LookAt { entity, x, y, z } => property(
            "target_yaw",
            tuple((
                string("look_at"),
                gen_entity_index(entity),
                string(" "),
                display(x),
                string(" "),
                display(y),
                string(" "),
                display(z),
            )),
        )(out),
    }
}

fn line_change<W: Write>(change: Change) -> impl SerializeFn<W> {
    move |out: WriteContext<W>| {
        let out = string("change ")(out)?;

        let out = string(match change.target {
            ChangeTarget::Yaw => "yaw",
            ChangeTarget::Pitch => "pitch",
            ChangeTarget::VectorialStrafingYaw => "target_yaw",
            ChangeTarget::VectorialStrafingYawOffset => "target_yaw_offset",
        })(out)?;

        let out = string(" to ")(out)?;
        let out = display(change.final_value)(out)?;
        let out = string(" over ")(out)?;
        let out = display(change.over)(out)?;
        let out = string(" s")(out)?;

        Ok(out)
    }
}

fn line<W: Write>(line: &Line) -> impl SerializeFn<W> + '_ {
    move |out: WriteContext<W>| match line {
        Line::FrameBulk(frame_bulk) => line_frame_bulk(frame_bulk)(out),
        Line::Save(save) => property("save", string(save))(out),
        Line::SharedSeed(seed) => property("seed", display(seed))(out),
        Line::Buttons(buttons) => line_buttons(*buttons)(out),
        Line::LGAGSTMinSpeed(lgagst_min_speed) => {
            property("lgagstminspeed", display(lgagst_min_speed))(out)
        }
        Line::Reset { non_shared_seed } => property("reset", display(non_shared_seed))(out),
        Line::Comment(comment) => tuple((string("//"), string(comment)))(out),
        Line::VectorialStrafing(enabled) => property(
            "strafing",
            string(if *enabled { "vectorial" } else { "yaw" }),
        )(out),
        Line::VectorialStrafingConstraints(constraints) => {
            line_vectorial_strafing_constraints(*constraints)(out)
        }
        Line::Change(change) => line_change(*change)(out),
        Line::TargetYawOverride(yaws) => tuple((
            string("target_yaw_override"),
            many_ref(yaws, |yaw| pair(string(" "), display(yaw))),
        ))(out),
    }
}

/// Prints [`Line`] into writer.
///
/// # Examples
///
/// ```
/// # extern crate hltas;
/// use hltas::types::Line;
///
/// let line = Line::Reset { non_shared_seed: 1234 };
/// let mut buf = Vec::new();
/// hltas::write::gen_line(&mut buf, &line).unwrap();
/// assert_eq!(buf, b"reset 1234");
/// ```
pub fn gen_line<W: Write>(w: W, value: &Line) -> Result<(), GenError> {
    let _ = gen_simple(line(value), w)?;
    Ok(())
}

fn line_nl<W: Write>(l: &Line) -> impl SerializeFn<W> + '_ {
    move |out: WriteContext<W>| pair(line(l), string("\n"))(out)
}

/// Prints newline-terminated [`Line`]s into writer.
///
/// # Examples
///
/// ```
/// # extern crate hltas;
/// use hltas::types::Line;
///
/// let lines = [
///     Line::Comment(" Hello World".to_string()),
///     Line::Reset { non_shared_seed: 1234 },
/// ];
/// let mut buf = Vec::new();
/// hltas::write::gen_lines(&mut buf, &lines).unwrap();
/// assert_eq!(buf, b"// Hello World\nreset 1234\n");
/// ```
pub fn gen_lines<'a, W, It, I>(w: W, lines: I) -> Result<(), GenError>
where
    W: Write,
    It: Iterator<Item = &'a Line> + Clone,
    I: IntoIterator<Item = &'a Line, IntoIter = It>,
{
    let _ = gen_simple(many_ref(lines, line_nl), w)?;
    Ok(())
}

/// Prints `HLTAS` into writer.
///
/// # Examples
///
/// ```
/// # extern crate hltas;
/// use hltas::HLTAS;
///
/// let contents = "version 1
/// demo test
/// frames
/// ------b---|------|------|0.001|-|-|5
/// ";
///
/// let hltas = HLTAS::from_str(contents).unwrap();
/// let mut buf = Vec::new();
/// hltas::write::gen_hltas(&mut buf, &hltas).unwrap();
/// assert_eq!(buf, contents.as_bytes());
/// ```
pub fn gen_hltas<W: Write>(w: W, hltas: &HLTAS) -> Result<(), GenError> {
    let mut w = gen_simple(string("version 1\n"), w)?;

    if let Some(demo) = hltas.properties.demo.as_deref() {
        w = gen_simple(pair(property("demo", string(demo)), string("\n")), w)?;
    }
    if let Some(save) = hltas.properties.save.as_deref() {
        w = gen_simple(pair(property("save", string(save)), string("\n")), w)?;
    }
    if let Some(Seeds { shared, non_shared }) = hltas.properties.seeds {
        let seeds = tuple((display(shared), string(" "), display(non_shared)));
        w = gen_simple(pair(property("seed", seeds), string("\n")), w)?;
    }
    if let Some(frametime_0ms) = hltas.properties.frametime_0ms.as_deref() {
        w = gen_simple(
            pair(
                property("frametime0ms", string(frametime_0ms)),
                string("\n"),
            ),
            w,
        )?;
    }
    if let Some(hlstrafe_version) = hltas.properties.hlstrafe_version {
        w = gen_simple(
            pair(
                property("hlstrafe_version", display(hlstrafe_version)),
                string("\n"),
            ),
            w,
        )?;
    }
    if let Some(load_command) = hltas.properties.load_command.as_deref() {
        w = gen_simple(
            pair(property("load_command", string(load_command)), string("\n")),
            w,
        )?;
    }

    let w = gen_simple(string("frames\n"), w)?;

    gen_lines(w, &hltas.lines)?;

    Ok(())
}
