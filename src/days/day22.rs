use rayon::prelude::*;
use std::{
    collections::{hash_map::Entry, HashMap},
    iter::successors,
};

use crate::util::prelude::*;

pub fn level1(input: &str) -> i64 {
    input
        .lines()
        .map(|line| line.parse().unwrap())
        .map(|secret| {
            std::iter::successors(Some(secret), |secret| Some(rotate(*secret)))
                .nth(2000)
                .unwrap()
        })
        .sum()
}

fn rotate(mut secret: i64) -> i64 {
    const MASK: i64 = (1 << 24) - 1;
    secret ^= secret << 6;
    secret &= MASK;
    secret ^= secret >> 5;
    secret &= MASK;
    secret ^= secret << 11;
    secret &= MASK;
    secret
}

fn first_buy(secret: i64) -> HashMap<[i8; 4], i64> {
    let mut result = HashMap::new();
    for (a, b, c, d, e) in successors(Some(secret), |secret| Some(rotate(*secret)))
        .map(|hash| (hash % 10) as i8)
        .take(2001)
        .tuple_windows()
    {
        if let Entry::Vacant(vacant_entry) = result.entry([b - a, c - b, d - c, e - d]) {
            vacant_entry.insert(e as i64);
        }
    }
    result
}

pub fn level2(input: &str) -> i64 {
    input
        .lines()
        .map(|line| line.parse().unwrap())
        .map(first_buy)
        .fold(HashMap::new(), |mut left, right| {
            for (changes, price) in right {
                *left.entry(changes).or_default() += price;
            }
            left
        })
        .into_values()
        .max()
        .unwrap_or_default()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day22.txt");
        assert_eq!(level1(test_input), 37327623)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day22_2.txt");
        assert_eq!(level2(test_input), 23)
    }
}
