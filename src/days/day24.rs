use std::collections::HashMap;

use nom::{
    branch::alt,
    character::complete::{alphanumeric1, newline, u8},
    combinator::eof,
    sequence::{separated_pair, tuple},
    IResult, Parser,
};
use nom_supreme::{
    error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated,
    tag::complete::tag, ParserExt,
};

use crate::util::prelude::*;

#[derive(Debug, Clone, Copy)]
enum Operation {
    And,
    Or,
    Xor,
}

impl Operation {
    fn apply(&self, inputs: [u8; 2]) -> u8 {
        match self {
            Operation::And => inputs[0] & inputs[1],
            Operation::Or => inputs[0] | inputs[1],
            Operation::Xor => inputs[0] ^ inputs[1],
        }
    }
}

#[derive(Debug)]
struct Computer<'a> {
    inputs: HashMap<&'a str, u8>,
    nodes: HashMap<&'a str, (Operation, [&'a str; 2])>,
}

impl<'a> Computer<'a> {
    pub fn from_input(input: &'a str) -> Result<Self, ErrorTree<&'a str>> {
        final_parser(Self::parse_computer)(input)
    }

    fn parse_computer(input: &'a str) -> IResult<&'a str, Self, ErrorTree<&'a str>> {
        tuple((
            collect_separated_terminated(
                separated_pair(alphanumeric1, tag(": "), u8),
                newline,
                newline.preceded_by(newline),
            ),
            collect_separated_terminated(
                separated_pair(
                    tuple((
                        alphanumeric1,
                        alt((
                            tag(" AND ").value(Operation::And),
                            tag(" OR ").value(Operation::Or),
                            tag(" XOR ").value(Operation::Xor),
                        )),
                        alphanumeric1,
                    )),
                    tag(" -> "),
                    alphanumeric1,
                )
                .map(|((left, op, right), output)| (output, (op, [left, right]))),
                newline,
                eof.opt_preceded_by(newline),
            ),
        ))
        .map(|(inputs, nodes)| Self { inputs, nodes })
        .parse(input)
    }

    fn results(self) -> HashMap<&'a str, u8> {
        let mut result = self.inputs;

        for node in self.nodes.keys() {
            if result.contains_key(node) {
                continue;
            }
            let mut stack = vec![node];

            while let Some(visited_node) = stack.last().copied() {
                let (op, [left, right]) =
                    self.nodes.get(visited_node).expect("only visit known keys");
                let Some(left_result) = result.get(left) else {
                    stack.push(left);
                    continue;
                };
                let Some(right_result) = result.get(right) else {
                    stack.push(right);
                    continue;
                };
                result.insert(visited_node, op.apply([*left_result, *right_result]));
                stack.pop();
            }
        }
        result
    }
}

pub fn level1(input: &str) -> i64 {
    let computer = Computer::from_input(input).expect("parse");
    let results = computer.results();
    results
        .into_iter()
        .filter(|(node, _)| node.starts_with('z'))
        .sorted()
        .rev()
        .map(|(_, bit)| bit)
        .fold(0, |acc, bit| 2 * acc + bit as i64)
}

pub fn level2(input: &str) -> i64 {
    0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day24.txt");
        assert_eq!(level1(test_input), 4)
    }

    #[test]
    fn level1_given_example_large() {
        let test_input = include_str!("./test_input/day24_large.txt");
        assert_eq!(level1(test_input), 2024)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day24.txt");
        assert_eq!(level2(test_input), 0)
    }
}
