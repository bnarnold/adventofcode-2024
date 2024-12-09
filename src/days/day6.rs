use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::util::prelude::*;

#[derive(Debug, Default, Clone)]
struct Map {
    obstacles: HashSet<(usize, usize)>,
    width: usize,
    height: usize,
}

impl Map {
    fn steps_till_escape(&self, mut pos: (usize, usize)) -> Option<HashSet<(usize, usize)>> {
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
        enum Dir {
            North,
            East,
            South,
            West,
        }
        let mut dir = Dir::North;
        let mut visited: HashSet<_> = std::iter::once(pos).collect();
        let mut visited_dir: HashSet<_> = std::iter::once((pos, dir)).collect();
        loop {
            let next_pos = match dir {
                Dir::North => {
                    let Some(y) = pos.1.checked_sub(1) else {
                        return Some(visited);
                    };
                    (pos.0, y)
                }
                Dir::East => {
                    let x = pos.0 + 1;
                    if x >= self.width {
                        return Some(visited);
                    }
                    (x, pos.1)
                }
                Dir::South => {
                    let y = pos.1 + 1;
                    if y >= self.height {
                        return Some(visited);
                    }
                    (pos.0, y)
                }
                Dir::West => {
                    let Some(x) = pos.0.checked_sub(1) else {
                        return Some(visited);
                    };
                    (x, pos.1)
                }
            };
            if self.obstacles.contains(&next_pos) {
                dir = match dir {
                    Dir::North => Dir::East,
                    Dir::East => Dir::South,
                    Dir::South => Dir::West,
                    Dir::West => Dir::North,
                }
            } else {
                visited.insert(next_pos);
                if !visited_dir.insert((next_pos, dir)) {
                    return None;
                };
                pos = next_pos;
            }
        }
    }
}

fn parse_input(input: &str) -> (Map, (usize, usize)) {
    let mut pos = None;
    let mut map = Map::default();

    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            match c {
                '.' => {}
                '^' => pos = Some((x, y)),
                '#' => {
                    map.obstacles.insert((x, y));
                }
                _ => panic!("unexpected char"),
            }
            if x == 0 {
                map.width += 1;
            }
        }
        map.height += 1;
    }

    (map, pos.expect("No starting position found"))
}

pub fn level1(input: &str) -> usize {
    let (map, pos) = parse_input(input);
    map.steps_till_escape(pos).unwrap().len()
}

pub fn level2(input: &str) -> usize {
    use rayon::prelude::*;
    let (map, start_pos) = parse_input(input);
    let visited = map.steps_till_escape(start_pos).unwrap();
    visited
        .into_par_iter()
        .filter(|pos| {
            let mut map = map.clone();
            map.obstacles.insert(*pos);
            map.steps_till_escape(start_pos).is_none()
        })
        .count()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day6.txt");
        assert_eq!(level1(test_input), 41)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day6.txt");
        assert_eq!(level2(test_input), 6)
    }
}
