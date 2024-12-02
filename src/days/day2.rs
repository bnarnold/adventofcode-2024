use std::{iter::once, usize};

use crate::util::prelude::*;

pub fn level1(input: &str) -> usize {
    input
        .lines()
        .filter(|line| {
            let numbers: Vec<u32> = line
                .split_whitespace()
                .map(|number| number.parse().expect("parse"))
                .collect();
            let mut increasing: Option<bool> = None;
            numbers.iter().copied().tuple_windows().all(|(old, new)| {
                let this_increasing = old < new;
                (1..=3).contains(&old.abs_diff(new))
                    && increasing.replace(this_increasing) != Some(!this_increasing)
            })
        })
        .count()
}

pub fn level2(input: &str) -> usize {
    input
        .lines()
        .filter(|line| {
            let numbers: Vec<u32> = line
                .split_whitespace()
                .map(|number| number.parse().expect("parse"))
                .collect();
            once(is_ok(numbers.iter().copied()))
                .chain((0..numbers.len()).map(|i| {
                    is_ok(
                        numbers
                            .iter()
                            .enumerate()
                            .filter(|(j, _)| *j != i)
                            .map(|(_, x)| *x),
                    )
                }))
                .any(|ok| ok)
        })
        .count()
}

fn is_ok(numbers: impl Iterator<Item = u32>) -> bool {
    let mut increasing: Option<bool> = None;
    numbers.tuple_windows().all(|(old, new)| {
        let this_increasing = old < new;
        (1..=3).contains(&old.abs_diff(new))
            && increasing.replace(this_increasing) != Some(!this_increasing)
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day2.txt");
        assert_eq!(level1(test_input), 2)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day2.txt");
        assert_eq!(level2(test_input), 4)
    }
}
