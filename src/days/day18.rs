use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet, VecDeque},
};

use anyhow::{anyhow, Context};

use crate::util::prelude::*;

#[derive(Debug)]
struct Map {
    side_length: u16,
    walls: HashSet<[u16; 2]>,
}

impl Map {
    fn from_wall_locations(
        input: &str,
        side_length: u16,
        line_count: usize,
    ) -> anyhow::Result<Self> {
        let walls: anyhow::Result<HashSet<_>> = input
            .lines()
            .take(line_count)
            .map(|line| {
                let (start, end) = line.split_once(',').ok_or(anyhow!("no comma"))?;
                Ok([start.parse()?, end.parse()?])
            })
            .collect();
        Ok(Self {
            side_length,
            walls: walls.context("parse")?,
        })
    }

    fn shortest_path(&self) -> Option<usize> {
        let start_pos = [0, 0];
        let end_pos = [self.side_length, self.side_length];

        let mut visited = HashSet::new();
        let mut queue = BinaryHeap::new();
        queue.push((Reverse(2 * self.side_length as usize), 0, start_pos));

        while let Some((_, steps, pos)) = queue.pop() {
            if !visited.insert(pos) {
                continue;
            }
            if pos == end_pos {
                return Some(steps);
            }

            let deltas = [[0, -1], [1, 0], [0, 1], [-1, 0]];

            queue.extend(deltas.into_iter().flat_map(|delta| {
                let new_x = pos[0].checked_add_signed(delta[0])?;
                let new_y = pos[1].checked_add_signed(delta[1])?;
                let new_pos = (new_x <= self.side_length && new_y <= self.side_length)
                    .then_some([new_x, new_y])?;
                if visited.contains(&new_pos) || self.walls.contains(&new_pos) {
                    return None;
                };

                let new_steps = steps + 1;
                let new_score = new_steps
                    + self.side_length.abs_diff(new_x) as usize
                    + self.side_length.abs_diff(new_y) as usize;
                Some((Reverse(new_score), new_steps, new_pos))
            }));
        }
        None
    }
}

fn level1_parametric(input: &str, side_length: u16, line_count: usize) -> usize {
    let map = Map::from_wall_locations(input, side_length, line_count).expect("parse");
    map.shortest_path().expect("path exists")
}

pub fn level1(input: &str) -> String {
    level1_parametric(input, 70, 1024).to_string()
}
fn level2_parametric(input: &str, side_length: u16) -> (u16, u16) {
    // TODO: analyze connected components iteratively

    let mut input_acc = String::new();
    for line in input.lines() {
        let next = if input_acc.is_empty() {
            line.to_string()
        } else {
            format!("\n{line}")
        };
        input_acc += &next;
        let map = Map::from_wall_locations(&input_acc, side_length, usize::MAX).expect("parse");
        if map.shortest_path().is_none() {
            let (x, y) = line.split_once(',').unwrap();
            return (x.parse().unwrap(), y.parse().unwrap());
        }
    }
    panic!("still can reach end")
}

pub fn level2(input: &str) -> String {
    let (x, y) = level2_parametric(input, 70);
    format!("{x},{y}")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day18.txt");
        assert_eq!(level1_parametric(test_input, 6, 12), 22)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day18.txt");
        assert_eq!(level2_parametric(test_input, 6), (6, 1))
    }
}
