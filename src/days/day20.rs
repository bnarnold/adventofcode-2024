use std::collections::{HashMap, HashSet};

use nom::combinator::opt;

use crate::util::prelude::*;

#[derive(Debug)]
struct Map {
    fields: Vec<[u16; 2]>,
    field_positions: HashMap<[u16; 2], usize>,
    width: u16,
    height: u16,
    start_pos: [u16; 2],
    end_pos: [u16; 2],
}

impl Map {
    fn parse_input(input: &str) -> Result<Self, &'static str> {
        let mut width = 0;
        let mut height = 0;
        let mut field_set = HashSet::new();
        let mut start_pos = None;
        let mut end_pos = None;
        for (y, line) in input.lines().enumerate() {
            let y = y as u16;
            height = y + 1;
            for (x, c) in line.chars().enumerate() {
                let x = x as u16;
                if y == 0 {
                    width = x + 1;
                }
                match c {
                    '#' => {}
                    '.' => {
                        field_set.insert([x, y]);
                    }
                    'S' => {
                        start_pos = Some([x, y]);
                        field_set.insert([x, y]);
                    }
                    'E' => {
                        end_pos = Some([x, y]);
                        field_set.insert([x, y]);
                    }
                    _ => return Err("unexpected char"),
                }
            }
        }

        let start_pos = start_pos.ok_or("no start")?;
        let end_pos = end_pos.ok_or("no end")?;

        let mut field_positions = HashMap::new();
        field_positions.insert(start_pos, 0);
        let mut fields = vec![start_pos];

        let mut pos = start_pos;
        for steps in 1.. {
            let next_pos = [[0, -1], [1, 0], [0, 1], [-1, 0]]
                .into_iter()
                .flat_map(|delta| {
                    let new_pos = [
                        pos[0].checked_add_signed(delta[0])?,
                        pos[1].checked_add_signed(delta[1])?,
                    ];
                    (field_set.contains(&new_pos) && !field_positions.contains_key(&new_pos))
                        .then_some(new_pos)
                })
                .next()
                .ok_or("no path")?;
            field_positions.insert(next_pos, steps);
            fields.push(next_pos);
            if next_pos == end_pos {
                break;
            }
            pos = next_pos
        }

        Ok(Self {
            fields,
            field_positions,
            width,
            height,
            start_pos,
            end_pos,
        })
    }

    fn cheat_savings(&self) -> Vec<usize> {
        let mut result = Vec::new();

        for (i, pos) in self.fields.iter().enumerate() {
            let cheat_start_score = i;
            let neighbors = |pos: [u16; 2]| {
                [[0, -1], [1, 0], [0, 1], [-1, 0]]
                    .into_iter()
                    .flat_map(move |delta| {
                        let new_pos = [
                            pos[0].checked_add_signed(delta[0])?,
                            pos[1].checked_add_signed(delta[1])?,
                        ];
                        (new_pos[0] < self.width && new_pos[1] < self.height).then_some(new_pos)
                    })
            };
            result.extend(
                neighbors(*pos)
                    .filter(|neighbor| !self.field_positions.contains_key(neighbor))
                    .flat_map(neighbors)
                    .flat_map(|double_neighbor| {
                        let cheat_end_score = self.field_positions.get(&double_neighbor)?;
                        let difference = cheat_end_score.checked_sub(cheat_start_score)?;
                        difference.checked_sub(2)
                    }),
            );
        }

        result
    }
}

pub fn level1(input: &str) -> usize {
    let map = Map::parse_input(input).expect("parse");
    map.cheat_savings()
        .into_iter()
        .filter(|saving| *saving >= 100)
        .count()
}

pub fn level2(input: &str) -> usize {
    0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day20.txt");
        let map = Map::parse_input(test_input).expect("parse");
        let cheats = map.cheat_savings();
        let counts = cheats.into_iter().counts();
        dbg!(&counts);
        assert_eq!(counts.get(&64).copied(), Some(1));
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day20.txt");
        assert_eq!(level2(test_input), 0)
    }
}
