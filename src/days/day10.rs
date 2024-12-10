use std::collections::{HashSet, VecDeque};

use crate::util::prelude::*;

pub fn level1(input: &str) -> usize {
    let map = input
        .lines()
        .map(|line| {
            line.chars()
                .map(|c| c.to_digit(10).expect("digit"))
                .collect_vec()
        })
        .collect_vec();

    let mut descendants = vec![vec![None; map[0].len()]; map.len()];
    let mut queue = VecDeque::new();

    for (y, row) in map.iter().enumerate() {
        for (x, d) in row.iter().enumerate() {
            if *d == 9 {
                queue.push_back((x, y));
                let mut end_descendants = HashSet::new();
                end_descendants.insert((x, y));
                descendants[y][x] = Some(end_descendants);
            }
        }
    }

    let mut count = 0;
    while let Some((x, y)) = queue.pop_front() {
        let height = map[y][x];
        if height == 0 {
            count += std::mem::take(&mut descendants[y][x])
                .unwrap_or_default()
                .len();
            continue;
        }
        let neighbors = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .flat_map(|(dx, dy)| {
                let child_x = x.checked_add_signed(dx)?;
                let child_y = y.checked_add_signed(dy)?;
                (map.get(child_y)?.get(child_x)? + 1 == height).then_some((child_x, child_y))
            });
        for (new_x, new_y) in neighbors {
            descendants[new_y][new_x] = Some(
                std::mem::take(&mut descendants[new_y][new_x])
                    .unwrap_or_default()
                    .union(descendants[y][x].as_ref().unwrap())
                    .copied()
                    .collect(),
            );
            queue.push_back((new_x, new_y));
        }
    }
    count
}

pub fn level2(input: &str) -> usize {
    let map = input
        .lines()
        .map(|line| {
            line.chars()
                .map(|c| c.to_digit(10).expect("digit"))
                .collect_vec()
        })
        .collect_vec();

    let mut path_counts = vec![vec![0; map[0].len()]; map.len()];
    let queue = map
        .iter()
        .enumerate()
        .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, &d)| (d, x, y)))
        .sorted()
        .map(|(_, x, y)| (x, y))
        .collect_vec();

    let mut count = 0;
    for (x, y) in queue {
        let height = map[y][x];
        if height == 9 {
            count += path_counts[y][x];
            continue;
        }
        if height == 0 {
            path_counts[y][x] = 1;
        }
        let neighbors = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .flat_map(|(dx, dy)| {
                let child_x = x.checked_add_signed(dx)?;
                let child_y = y.checked_add_signed(dy)?;
                (*map.get(child_y)?.get(child_x)? == height + 1).then_some((child_x, child_y))
            });
        for (new_x, new_y) in neighbors {
            path_counts[new_y][new_x] += path_counts[y][x];
        }
    }
    count
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day10.txt");
        assert_eq!(level1(test_input), 36)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day10.txt");
        assert_eq!(level2(test_input), 81)
    }
}
