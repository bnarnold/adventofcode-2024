use std::collections::HashSet;

use nom::{
    bytes::complete::tag,
    character::complete::{newline, u64},
    combinator::eof,
    sequence::separated_pair,
};
use nom_supreme::{
    error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated,
};

use crate::util::prelude::*;

pub fn level1(input: &str) -> u64 {
    let parsed: Result<Vec<(_, Vec<_>)>, ErrorTree<&str>> =
        final_parser::<_, _, ErrorTree<&str>, _>(collect_separated_terminated(
            separated_pair(
                u64,
                tag(": "),
                collect_separated_terminated(u64, tag(" "), newline),
            ),
            tag(""),
            eof,
        ))(input);
    let parsed = parsed.expect("parse error");
    parsed
        .into_iter()
        .filter(|(target, numbers)| can_build(*target, numbers))
        .map(|(target, _)| target)
        .sum()
}

fn can_build(target: u64, numbers: &[u64]) -> bool {
    let Some((&last, numbers)) = numbers.split_first() else {
        return false;
    };
    let mut candidates: HashSet<_> = [(last)].into_iter().collect();
    for number in numbers.iter().copied() {
        candidates = candidates
            .into_iter()
            .flat_map(|x| [(x + number), (x * number)])
            .filter(|x| *x <= target)
            .collect();
    }
    candidates.into_iter().any(|x| x == target)
}

fn concatenate(x: u64, y: u64) -> Option<u64> {
    let mut power_of_10 = 1;
    while power_of_10 <= y {
        power_of_10 = power_of_10.checked_mul(10)?;
    }
    x.checked_mul(power_of_10)?.checked_add(y)
}
fn can_build_with_concatenate(target: u64, numbers: &[u64]) -> bool {
    let Some((&last, numbers)) = numbers.split_first() else {
        return false;
    };
    let mut candidates: HashSet<_> = [(last)].into_iter().collect();
    for number in numbers.iter().copied() {
        candidates = candidates
            .into_iter()
            .flat_map(|x| {
                [
                    x.checked_add(number),
                    x.checked_mul(number),
                    concatenate(x, number),
                ]
                .into_iter()
                .flatten()
            })
            .filter(|x| *x <= target)
            .collect();
    }
    candidates.into_iter().any(|x| x == target)
}
pub fn level2(input: &str) -> u64 {
    let parsed: Result<Vec<(_, Vec<_>)>, ErrorTree<&str>> =
        final_parser::<_, _, ErrorTree<&str>, _>(collect_separated_terminated(
            separated_pair(
                u64,
                tag(": "),
                collect_separated_terminated(u64, tag(" "), newline),
            ),
            tag(""),
            eof,
        ))(input);
    let parsed = parsed.expect("parse error");
    parsed
        .into_iter()
        .filter(|(target, numbers)| can_build_with_concatenate(*target, numbers))
        .map(|(target, _)| target)
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day7.txt");
        assert_eq!(level1(test_input), 3749)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day7.txt");
        assert_eq!(level2(test_input), 11387)
    }
}
