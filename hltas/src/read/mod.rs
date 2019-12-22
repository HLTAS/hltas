use std::fmt::{self, Display, Formatter};

use nom::{
    self,
    bytes::complete::tag,
    character::complete::{line_ending, multispace0, space1},
    eof,
    error::ParseError,
    multi::{many0, many1},
    sequence::preceded,
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Error<'a> {
    pub input: &'a str,
    pub whole_input: &'a str,
    kind: ErrorKind,
}

type IResult<'a, T> = Result<(&'a str, T), nom::Err<Error<'a>>>;

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

        match self.kind {
            ErrorKind::ExpectedChar(c) => {
                if let Some(next_char) = self.input.chars().next() {
                    write!(f, "expected '{}', got '{}'", c, next_char)?;
                } else {
                    write!(f, "expected '{}', got EOF", c)?;
                }
            }
            ErrorKind::Other(nom_kind) => write!(f, "error applying {}", nom_kind.description())?,
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
        }
    }

    fn or(self, other: Self) -> Self {
        println!("{:?}", self);
        other
    }
}

fn version(i: &str) -> IResult<()> {
    let (i, _) = preceded(tag("version"), preceded(space1, tag("1")))(i)?;
    Ok((i, ()))
}

/// Parses a line ending character, followed by any additional whitespace.
fn whitespace(i: &str) -> IResult<()> {
    let (i, _) = preceded(line_ending, multispace0)(i)?;
    Ok((i, ()))
}

/// Parses an entire HLTAS script, ensuring nothing is left in the input.
pub fn hltas(i: &str) -> IResult<HLTAS> {
    let (i, _) = version(i)?;
    let (i, properties) = properties(i)?;
    let (i, _) = preceded(many1(line_ending), tag("frames"))(i)?;
    let (i, lines) = many0(preceded(whitespace, line))(i)?;

    let (i, _) = multispace0(i)?; // There can be arbitrary space in the end.
    let (i, _) = eof!(i,)?; // Error out if we didn't parse the whole input.

    Ok((i, HLTAS { properties, lines }))
}
