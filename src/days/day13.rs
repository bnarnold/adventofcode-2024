use nom::{
    character::complete::{i64, newline},
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
struct Machine {
    buttons: [[i64; 2]; 2],
    prize: [i64; 2],
}

impl Machine {
    fn winning_combination(&self) -> Option<[i64; 2]> {
        let determinant =
            self.buttons[0][0] * self.buttons[1][1] - self.buttons[0][1] * self.buttons[1][0];
        let multiplied_with_transpose = [
            self.buttons[1][1] * self.prize[0] - self.buttons[1][0] * self.prize[1],
            -self.buttons[0][1] * self.prize[0] + self.buttons[0][0] * self.prize[1],
        ];
        multiplied_with_transpose
            .iter()
            .all(|coord| coord % determinant == 0)
            .then_some(multiplied_with_transpose.map(|coord| coord / determinant))
    }

    fn correct_unit_conversion(self) -> Self {
        Self {
            prize: self.prize.map(|entry| entry + 10000000000000),
            ..self
        }
    }
}

fn parse_machine(input: &str) -> IResult<&str, Machine, ErrorTree<&str>> {
    tuple((
        separated_pair(
            i64.preceded_by(tag("X+")),
            tag(", "),
            i64.preceded_by(tag("Y+")),
        )
        .preceded_by(tag("Button A: ")),
        separated_pair(
            i64.preceded_by(tag("X+")),
            tag(", "),
            i64.preceded_by(tag("Y+")),
        )
        .preceded_by(tag("Button B: "))
        .preceded_by(newline),
        separated_pair(
            i64.preceded_by(tag("X=")),
            tag(", "),
            i64.preceded_by(tag("Y=")),
        )
        .preceded_by(tag("Prize: "))
        .preceded_by(newline),
    ))
    .map(|(button_a, button_b, prize)| Machine {
        buttons: [[button_a.0, button_a.1], [button_b.0, button_b.1]],
        prize: [prize.0, prize.1],
    })
    .parse(input)
}

fn parse_input(input: &str) -> Result<Vec<Machine>, ErrorTree<&str>> {
    final_parser(collect_separated_terminated(
        parse_machine,
        newline.precedes(newline),
        eof.opt_preceded_by(newline),
    ))(input)
}

pub fn level1(input: &str) -> i64 {
    let input = parse_input(input).expect("parse");
    input
        .into_iter()
        .flat_map(|machine| machine.winning_combination())
        .filter(|combination| combination[0] <= 100 && combination[1] <= 100)
        .map(|combination| 3 * combination[0] + combination[1])
        .sum()
}

pub fn level2(input: &str) -> i64 {
    let input = parse_input(input).expect("parse");
    input
        .into_iter()
        .flat_map(|machine| machine.correct_unit_conversion().winning_combination())
        .map(|combination| 3 * combination[0] + combination[1])
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day13.txt");
        assert_eq!(level1(test_input), 480)
    }
}
