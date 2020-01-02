//! Reading `.hltas` files.

use std::{
    fmt::{self, Display, Formatter},
    num::NonZeroU32,
    str::FromStr,
};

use nom::{
    self,
    bytes::complete::tag,
    character::complete::{digit0, line_ending, multispace0, one_of, space1},
    combinator::{all_consuming, map_res, recognize, verify},
    error::ParseError,
    multi::{many1, many_till},
    sequence::{pair, preceded},
    Offset,
};

use crate::types::HLTAS;

mod line;
use line::line;

pub(crate) mod properties;
use properties::properties;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ErrorKind {
    ExpectedChar(char),
    Other(nom::error::ErrorKind),
}

/// Enumeration of possible semantic errors.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Context {
    /// Failed to read the version.
    ErrorReadingVersion,
    /// The version is too high.
    VersionTooHigh,
    /// Both autojump and ducktap are enabled at once.
    BothAutoJumpAndDuckTap,
    /// LGAGST is enabled without autojump or ducktap.
    NoLeaveGroundAction,
    /// Times is specified on the LGAGST action.
    TimesOnLeaveGroundAction,
    /// Save name is not specified.
    NoSaveName,
    /// Seed is not specified.
    NoSeed,
    /// Yaw is required but not specified.
    NoYaw,
    /// Buttons are not specified.
    NoButtons,
    /// The LGAGST min speed valueis not specified.
    NoLGAGSTMinSpeed,
    /// The reset seed is not specified.
    NoResetSeed,
    /// Failed to parse a frames entry.
    ErrorParsingLine,
    /// Invalid strafing algorithm.
    InvalidStrafingAlgorithm,
    /// Vectorial strafing constraints are not specified.
    NoConstraints,
    /// Vectorial strafing constraint tolerance is not specified.
    NoTolerance,
    /// The +- in the vectorial strafing constraints is missing.
    NoPlusMinusBeforeTolerance,
    /// The parameters in the yaw range vectorial strafing constraints are not specified.
    NoFromToParameters,
    /// The yaw range vectorial strafing constraint is missing the "to" word.
    NoTo,
}

/// `.hltas` parsing error.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Error<'a> {
    /// Remaining input at the point of failure.
    pub input: &'a str,
    pub(crate) whole_input: &'a str,
    kind: ErrorKind,
    /// Semantic meaning of the parsing error.
    pub context: Option<Context>,
}

type IResult<'a, T> = Result<(&'a str, T), nom::Err<Error<'a>>>;

impl Error<'_> {
    fn add_context(&mut self, context: Context) {
        if self.context.is_some() {
            return;
        }

        self.context = Some(context);
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Context::*;
        match self {
            ErrorReadingVersion => write!(f, "failed to read version"),
            VersionTooHigh => write!(f, "this version is not supported"),
            BothAutoJumpAndDuckTap => write!(
                f,
                "both autojump and ducktap are specified at the same time"
            ),
            NoLeaveGroundAction => write!(
                f,
                "no LGAGST action specified (either autojump or ducktap is required)"
            ),
            TimesOnLeaveGroundAction => write!(
                f,
                "times on autojump or ducktap with LGAGST enabled (put times on LGAGST instead)"
            ),
            NoSaveName => write!(f, "missing save name"),
            NoSeed => write!(f, "missing seed value"),
            NoButtons => write!(f, "missing button values"),
            NoLGAGSTMinSpeed => write!(f, "missing lgagstminspeed value"),
            NoResetSeed => write!(f, "missing reset seed"),
            NoYaw => write!(f, "missing yaw value"),
            ErrorParsingLine => write!(f, "failed to parse the line"),
            InvalidStrafingAlgorithm => write!(
                f,
                "invalid strafing algorithm (only \"yaw\" and \"vectorial\" allowed)"
            ),
            NoConstraints => write!(f, "missing constraints"),
            NoTolerance => write!(f, "missing tolerance (e.g. \"+-0.1\")"),
            NoPlusMinusBeforeTolerance => write!(f, "missing +- before tolerance"),
            NoFromToParameters => write!(f, "missing from/to parameters"),
            NoTo => write!(f, "missing \"to\" in the from/to constraint"),
        }
    }
}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut line = 0;
        let mut column = 0;
        let mut offset = self.whole_input.offset(self.input);
        let mut just_error_line = None;

        for (j, l) in self.whole_input.lines().enumerate() {
            if offset <= l.len() {
                line = j;
                column = offset;
                just_error_line = Some(l);
                break;
            } else {
                offset = offset - l.len() - 1;
            }
        }

        if let Some(context) = self.context {
            context.fmt(f)?;
        } else {
            match self.kind {
                ErrorKind::ExpectedChar(c) => {
                    if let Some(next_char) = self.input.chars().next() {
                        write!(f, "expected '{}', got '{}'", c, next_char)?;
                    } else {
                        write!(f, "expected '{}', got EOF", c)?;
                    }
                }
                ErrorKind::Other(nom_kind) => {
                    write!(f, "error applying {}", nom_kind.description())?
                }
            }
        }

        // Can happen if whole_input is some unrelated &str.
        if just_error_line.is_none() {
            return Ok(());
        }
        let just_error_line = just_error_line.unwrap();

        let line_number = format!("{} | ", line);

        write!(f, "\n{}{}\n", line_number, just_error_line)?;
        write!(f, "{:1$}^", ' ', line_number.len() + column)?;

        if let ErrorKind::ExpectedChar(c) = self.kind {
            write!(f, " expected '{}'", c)?;
        }

        Ok(())
    }
}

impl std::error::Error for Error<'_> {}

impl<'a> ParseError<&'a str> for Error<'a> {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        Self {
            input,
            whole_input: input,
            kind: ErrorKind::Other(kind),
            context: None,
        }
    }

    fn append(_input: &'a str, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: &'a str, c: char) -> Self {
        Self {
            input,
            whole_input: input,
            kind: ErrorKind::ExpectedChar(c),
            context: None,
        }
    }
}

impl Error<'_> {
    /// Returns the line number on which the error has occurred.
    pub fn line(&self) -> usize {
        let mut line = 0;
        let mut offset = self.whole_input.offset(self.input);

        for (j, l) in self.whole_input.lines().enumerate() {
            if offset <= l.len() {
                line = j;
                break;
            } else {
                offset = offset - l.len() - 1;
            }
        }

        line
    }
}

/// Adds context to the potential parser error.
///
/// If the error already has context stored, does nothing.
fn context<'a, T>(
    context: Context,
    f: impl Fn(&'a str) -> IResult<T>,
) -> impl Fn(&'a str) -> IResult<T> {
    move |i: &str| {
        f(i).map_err(move |error| match error {
            nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
            nom::Err::Error(mut e) => {
                e.add_context(context);
                nom::Err::Error(e)
            }
            nom::Err::Failure(mut e) => {
                e.add_context(context);
                nom::Err::Failure(e)
            }
        })
    }
}

fn non_zero_u32(i: &str) -> IResult<NonZeroU32> {
    map_res(
        recognize(pair(one_of("123456789"), digit0)),
        NonZeroU32::from_str,
    )(i)
}

fn version(i: &str) -> IResult<()> {
    // This is a little involved to report the correct HLTAS error.
    // When we can't parse the version as a number at all, we should report ErrorReadingVersion.
    // When we can parse it as a number and it's above 1, we should report VersionTooHigh.
    let version_number = context(
        Context::VersionTooHigh,
        verify(
            context(Context::ErrorReadingVersion, one_of("123456789")),
            |&c| c == '1',
        ),
    );
    let (i, _) = preceded(tag("version"), preceded(space1, version_number))(i)?;
    Ok((i, ()))
}

/// Parses a line ending character, followed by any additional whitespace.
fn whitespace(i: &str) -> IResult<()> {
    let (i, _) = preceded(line_ending, multispace0)(i)?;
    Ok((i, ()))
}

/// Parses an entire HLTAS script, ensuring nothing is left in the input.
pub(crate) fn hltas(i: &str) -> IResult<HLTAS> {
    let (i, _) = context(Context::ErrorReadingVersion, version)(i)?;
    let (i, properties) = properties(i)?;
    let (i, _) = preceded(many1(line_ending), tag("frames"))(i)?;
    let (i, (lines, _)) = context(
        Context::ErrorParsingLine,
        many_till(preceded(whitespace, line), all_consuming(multispace0)),
    )(i)?;

    Ok((i, HLTAS { properties, lines }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_0() {
        let input = "version 0";
        let err = version(input).unwrap_err();
        if let nom::Err::Error(err) = err {
            assert_eq!(err.context, Some(Context::ErrorReadingVersion));
        } else {
            unreachable!()
        }
    }

    #[test]
    fn version_too_high() {
        let input = "version 9";
        let err = version(input).unwrap_err();
        if let nom::Err::Error(err) = err {
            assert_eq!(err.context, Some(Context::VersionTooHigh));
        } else {
            unreachable!()
        }
    }
}
