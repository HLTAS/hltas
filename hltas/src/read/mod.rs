use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, multispace0, space1},
    eof,
    multi::{many0, many1},
    sequence::preceded,
    IResult,
};

use crate::types::HLTAS;

mod line;
use line::line;

pub(crate) mod properties;
use properties::properties;

fn version(i: &str) -> IResult<&str, ()> {
    let (i, _) = preceded(tag("version"), preceded(space1, tag("1")))(i)?;
    Ok((i, ()))
}

/// Parses a line ending character, followed by any additional whitespace.
fn whitespace(i: &str) -> IResult<&str, ()> {
    let (i, _) = preceded(line_ending, multispace0)(i)?;
    Ok((i, ()))
}

/// Parses an entire HLTAS script, ensuring nothing is left in the input.
pub fn hltas(i: &str) -> IResult<&str, HLTAS> {
    let (i, _) = version(i)?;
    let (i, properties) = properties(i)?;
    let (i, _) = preceded(many1(line_ending), tag("frames"))(i)?;
    let (i, lines) = many0(preceded(whitespace, line))(i)?;

    let (i, _) = multispace0(i)?; // There can be arbitrary space in the end.
    let (i, _) = eof!(i,)?; // Error out if we didn't parse the whole input.

    Ok((i, HLTAS { properties, lines }))
}
