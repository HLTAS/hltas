use std::{borrow::Cow, str::FromStr};

use nom::{
    bytes::complete::take_while1,
    character::complete::{char, digit1, line_ending, not_line_ending, space1},
    combinator::{map, map_res, opt, recognize},
    multi::many1,
    sequence::{pair, preceded, separated_pair},
    AsChar,
};

use crate::{
    read::{non_zero_u32, IResult},
    types::{Properties, Seeds},
};

pub(crate) fn property(i: &str) -> IResult<(&str, &str)> {
    separated_pair(
        take_while1(|c: char| c.is_alphanum() || c == '_'),
        space1,
        not_line_ending,
    )(i)
}

pub(crate) fn shared_seed(i: &str) -> IResult<u32> {
    map_res(digit1, u32::from_str)(i)
}

pub(crate) fn non_shared_seed(i: &str) -> IResult<i64> {
    map_res(recognize(pair(opt(char('-')), digit1)), i64::from_str)(i)
}

pub(crate) fn seeds(i: &str) -> IResult<Seeds> {
    map(
        separated_pair(shared_seed, space1, non_shared_seed),
        |(shared, non_shared)| Seeds { shared, non_shared },
    )(i)
}

fn nl_property(i: &str) -> IResult<(&str, &str)> {
    preceded(many1(line_ending), property)(i)
}

pub(crate) fn properties(mut i: &str) -> IResult<Properties> {
    let mut properties = Properties::default();

    while let Ok((input, (name, value))) = nl_property(i) {
        i = input;

        match name {
            "demo" => properties.demo = Some(Cow::Borrowed(value)),
            "save" => properties.save = Some(Cow::Borrowed(value)),
            "frametime0ms" => properties.frametime_0ms = Some(Cow::Borrowed(value)),
            "seed" => properties.seeds = Some(seeds(value)?.1),
            "hlstrafe_version" => properties.hlstrafe_version = Some(non_zero_u32(value)?.1),
            "load_command" => properties.load_command = Some(Cow::Borrowed(value)),
            _ => continue,
        }
    }

    Ok((i, properties))
}
