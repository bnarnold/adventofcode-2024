use nom::{
    branch::alt,
    character::complete::{anychar, u32},
    multi::many0,
    sequence::{delimited, separated_pair},
    IResult, Parser,
};
use nom_supreme::{error::ErrorTree, final_parser::final_parser, tag::complete::tag, ParserExt};

use crate::util::prelude::*;

pub fn level1(input: &str) -> u32 {
    let instructions = parse_line(input).expect("parse");
    instructions
        .into_iter()
        .map(|instruction| match instruction {
            Instruction::Mul(x, y) => x * y,
            _ => 0,
        })
        .sum()
}

pub fn level2(input: &str) -> u32 {
    let instructions = parse_line(input).expect("parse");
    let mut enabled = true;
    instructions
        .into_iter()
        .map(|instruction| match instruction {
            Instruction::Mul(x, y) if enabled => x * y,
            Instruction::Do => {
                enabled = true;
                0
            }
            Instruction::Dont => {
                enabled = false;
                0
            }
            _ => 0,
        })
        .sum()
}

#[derive(Debug, Copy, Clone)]
enum Instruction {
    Mul(u32, u32),
    Do,
    Dont,
    Corrupted,
}

fn parse_line(input: &str) -> Result<Vec<Instruction>, ErrorTree<&str>> {
    final_parser(many0(alt((
        parse_mul.map(|(x, y)| Instruction::Mul(x, y)),
        tag("do()").map(|_| Instruction::Do),
        tag("don't()").map(|_| Instruction::Dont),
        anychar.map(|_| Instruction::Corrupted),
    ))))(input)
}

fn parse_mul(input: &str) -> IResult<&str, (u32, u32), ErrorTree<&str>> {
    delimited(tag("("), separated_pair(u32, tag(","), u32), tag(")"))
        .preceded_by(tag("mul"))
        .context("mul")
        .parse(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day3.txt");
        assert_eq!(level1(test_input), 161)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day3_extended.txt");
        assert_eq!(level2(test_input), 48)
    }
}
