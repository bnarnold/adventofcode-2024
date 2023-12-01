use std::collections::HashMap;

use crate::util::prelude::*;

pub fn level1(input: &str) -> u32 {
    let (first_list, second_list) = parse_numbers(input);
    first_list
        .into_iter()
        .zip(second_list)
        .map(|(first, second)| first.abs_diff(second))
        .sum()
}

fn parse_numbers(input: &str) -> (Vec<u32>, Vec<u32>) {
    let numbers: Vec<(u32, u32)> = input
        .lines()
        .map(|line| {
            let (start, end) = line.split_once(' ').expect("no space");
            (
                start.trim().parse().expect("parse start"),
                end.trim().parse().expect("parse end"),
            )
        })
        .collect();
    (
        numbers.iter().map(|(first, _)| *first).sorted().collect(),
        numbers.iter().map(|(_, second)| *second).sorted().collect(),
    )
}

pub fn level2(input: &str) -> u32 {
    let (first_list, second_list) = parse_numbers(input);
    let mut second_list_counts: HashMap<u32, u32> = HashMap::new();
    for number in second_list {
        *second_list_counts.entry(number).or_default() += 1;
    }
    first_list
        .into_iter()
        .map(|number| number * second_list_counts.get(&number).copied().unwrap_or_default())
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day1.txt");
        assert_eq!(level1(test_input), 11)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day1.txt");
        assert_eq!(level2(test_input), 31)
    }
}
