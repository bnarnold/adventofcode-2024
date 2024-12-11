use std::{collections::HashMap, hash::Hash};

use crate::util::prelude::*;

#[derive(Debug)]
struct SumMap<K>(HashMap<K, usize>);

impl<K> FromIterator<(K, usize)> for SumMap<K>
where
    K: Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = (K, usize)>>(iter: T) -> Self {
        let mut result = HashMap::new();
        for (k, count) in iter {
            *result.entry(k).or_default() += count;
        }
        Self(result)
    }
}

fn split_stones(input: &str, iterations: usize) -> usize {
    let mut sum_map: SumMap<u64> = input
        .trim()
        .split(' ')
        .map(|number| (number.parse().expect("Input only has numbers"), 1))
        .collect();
    for _ in 0..iterations {
        sum_map = sum_map
            .0
            .into_iter()
            .flat_map(|(number, count)| {
                if number == 0 {
                    return vec![(1, count)];
                }
                let mut power_of_100 = 100;
                let mut power_of_10 = 10;
                while power_of_100 <= number {
                    power_of_10 *= 10;
                    power_of_100 *= 100;
                }
                if power_of_100 > 10 * number {
                    vec![(number * 2024, count)]
                } else {
                    [number / power_of_10, number % power_of_10]
                        .into_iter()
                        .map(|new_number| (new_number, count))
                        .collect()
                }
            })
            .collect();
    }
    sum_map.0.into_values().sum()
}

pub fn level1(input: &str) -> usize {
    split_stones(input, 25)
}

pub fn level2(input: &str) -> usize {
    split_stones(input, 75)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day11.txt");
        assert_eq!(level1(test_input), 55312)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day11.txt");
        assert_eq!(level2(test_input), 0)
    }
}
