use std::str::FromStr;

use nom::{
    character::complete::{alphanumeric1, char, digit1, line_ending, not_line_ending, space1},
    combinator::{map, map_res, opt, recognize},
    multi::many1,
    sequence::{pair, preceded, separated_pair},
};

use crate::{
    read::IResult,
    types::{Properties, Seeds},
};

pub(crate) fn property(i: &str) -> IResult<(&str, &str)> {
    separated_pair(alphanumeric1, space1, not_line_ending)(i)
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
            "demo" => properties.demo = Some(value),
            "save" => properties.save = Some(value),
            "frametime0ms" => properties.frametime_0ms = Some(value),
            "seed" => properties.seeds = Some(seeds(value)?.1),
            _ => continue,
        }
    }

    Ok((i, properties))
}