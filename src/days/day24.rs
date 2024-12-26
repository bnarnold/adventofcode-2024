use core::panic;
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

    fn swap(&mut self, src: &'a str, tgt: &'a str) {
        let Some(src_op) = self.nodes.remove(&src) else {
            return;
        };
        let Some(tgt_op) = self.nodes.remove(&tgt) else {
            return;
        };

        self.nodes.insert(src, tgt_op);
        self.nodes.insert(tgt, src_op);
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
    // check some properties, this is easier to solve ad hoc in the editor
    let mut computer = Computer::from_input(input).expect("parse");
    let swaps = [
        ["tpk", "wkb"],
        ["shj", "z07"],
        ["pfn", "z23"],
        ["kcd", "z27"],
    ];
    for swap in &swaps {
        computer.swap(swap[0], swap[1]);
    }
    let bit_count = computer.inputs.len() / 2;

    let xors = (0..bit_count)
        .map(|i| {
            let Some((label, _)) = computer.nodes.iter().find(|(_, (op, mut inputs))| {
                let x_input = format!("x{i:02}");
                let y_input = format!("y{i:02}");

                inputs.sort();
                matches!(op, Operation::Xor) && inputs == [x_input, y_input]
            }) else {
                panic!("No xor for bit {i:02}");
            };
            *label
        })
        .collect_vec();
    let ands = (0..bit_count)
        .map(|i| {
            let Some((label, _)) = computer.nodes.iter().find(|(_, (op, mut inputs))| {
                let x_input = format!("x{i:02}");
                let y_input = format!("y{i:02}");

                inputs.sort();
                matches!(op, Operation::And) && inputs == [x_input, y_input]
            }) else {
                panic!("No xor for bit {i:02}");
            };
            *label
        })
        .collect_vec();

    let carries = (1..bit_count)
        .map(|i| {
            let Some(label) = computer
                .nodes
                .iter()
                .flat_map(|(_, (op, [left, right]))| {
                    let xor = xors[i];

                    if matches!(op, Operation::Xor) {
                        if *left == xor {
                            Some(right)
                        } else if *right == xor {
                            Some(left)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .next()
            else {
                panic!("no xor carry operation for {i:02}");
            };
            *label
        })
        .collect_vec();

    for i in 0..bit_count {
        let (op, mut inputs) = computer
            .nodes
            .get(&format!("z{i:02}").as_str())
            .copied()
            .unwrap();
        assert!(
            matches!(op, Operation::Xor),
            "unexpected operation {op:?} for bit {i:02}",
        );
        inputs.sort();

        let known_inputs = if i == 0 {
            ["x00", "y00"]
        } else {
            let mut known_inputs = [xors[i], carries[i - 1]];
            known_inputs.sort();
            known_inputs
        };
        assert_eq!(
            inputs, known_inputs,
            "unexpected inputs {inputs:?} for bit {i:02} (expected {known_inputs:?})"
        );
    }

    let output = swaps.into_iter().flatten().sorted().join(",");
    println!("{output}");
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
}
