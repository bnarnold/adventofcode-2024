use core::panic;
use std::collections::{BTreeMap, BTreeSet};

use nom::AsBytes;

use crate::util::prelude::*;

pub fn level1(input: &str) -> usize {
    let mut lock_count = 0;
    let mut locks_by_cylinder_and_depth = [0; 5].map(|_| [0; 6].map(|_| BTreeSet::new()));
    let mut keys = Vec::new();

    for chunk in input.trim().split("\n\n") {
        if let Some(chunk) = chunk.strip_prefix("#####\n") {
            let chunk = chunk.strip_suffix("\n.....").expect("chunk is lock");
            lock_count += 1;
            for counts in locks_by_cylinder_and_depth.iter_mut() {
                counts[5].insert(lock_count);
            }
            for (depth, line) in chunk.lines().enumerate() {
                for (cylinder, c) in line.chars().enumerate() {
                    match c {
                        '.' => {
                            locks_by_cylinder_and_depth[cylinder][depth].insert(lock_count);
                        }
                        '#' => {}
                        _ => panic!("Unexpected char"),
                    }
                }
            }
        }
        if let Some(chunk) = chunk.strip_prefix(".....\n") {
            let chunk = chunk.strip_suffix("\n#####").expect("chunk is key");
            let lines: Vec<&[u8]> = chunk.lines().map(|line| line.as_bytes()).collect_vec();
            debug_assert_eq!(lines.len(), 5);

            let mut key = [0; 5];
            for (cylinder, key_part) in key.iter_mut().enumerate() {
                let depth = (0..5)
                    .find(|depth| lines[*depth][cylinder] == b'#')
                    .unwrap_or(5);
                *key_part = depth
            }
            keys.push(key);
        }
    }

    let mut result = 0;
    for key in keys {
        let matching_locks = locks_by_cylinder_and_depth
            .iter()
            .enumerate()
            .map(|(cylinder, counts)| -> BTreeSet<_> {
                counts[key[cylinder]].iter().copied().collect()
            })
            .reduce(|acc, set| &acc & &set)
            .expect("more than one cylinder");
        result += matching_locks.len();
    }
    result
}

pub fn level2(input: &str) -> usize {
    0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day25.txt");
        assert_eq!(level1(test_input), 3)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day25.txt");
        assert_eq!(level2(test_input), 0)
    }
}
