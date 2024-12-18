use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    io::{self, stdout},
    time::Duration,
};

use anyhow::{anyhow, Context};
use crossterm::{
    cursor::{Hide, MoveTo, MoveToNextLine, Show},
    event, execute,
    style::{Color, PrintStyledContent, Stylize},
    terminal::Clear,
};

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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum WallPos {
    TopRight,
    BottomLeft,
    Interior([u16; 2]),
}

#[derive(Debug)]
struct ConnectedComponentMap {
    side_length: u16,
    connected_components: HashMap<WallPos, Option<WallPos>>,
}

impl ConnectedComponentMap {
    fn new(side_length: u16) -> Self {
        let mut connected_components = HashMap::new();
        connected_components.insert(WallPos::TopRight, None);
        connected_components.insert(WallPos::BottomLeft, None);
        Self {
            connected_components,
            side_length,
        }
    }

    fn sides_connected(&self) -> bool {
        self.canonical_representative(WallPos::TopRight)
            .expect("sides are in components")
            == self
                .canonical_representative(WallPos::BottomLeft)
                .expect("sides are in components")
    }

    fn neighbor(&self, pos: [u16; 2], delta: [i16; 2]) -> WallPos {
        let Some(new_x) = pos[0].checked_add_signed(delta[0]) else {
            return WallPos::BottomLeft;
        };
        let Some(new_y) = pos[1].checked_add_signed(delta[1]) else {
            return WallPos::TopRight;
        };
        if new_x > self.side_length {
            WallPos::TopRight
        } else if new_y > self.side_length {
            WallPos::BottomLeft
        } else {
            WallPos::Interior([new_x, new_y])
        }
    }

    fn canonical_representative(&self, mut pos: WallPos) -> Option<WallPos> {
        while let Some(parent) = *self.connected_components.get(&pos)? {
            pos = parent;
        }
        debug_assert!(self.connected_components.get(&pos).unwrap().is_none());
        Some(pos)
    }

    fn canonical_representative_with_update(&mut self, mut pos: WallPos) -> Option<WallPos> {
        let mut ancestors = Vec::new();
        while let Some(parent) = *self.connected_components.get(&pos)? {
            ancestors.push(pos);
            pos = parent;
        }
        let representative = pos;
        for ancestor in ancestors {
            *self
                .connected_components
                .get_mut(&ancestor)
                .expect("already visited") = Some(representative);
        }
        Some(representative)
    }

    fn add_interior_wall(&mut self, pos: [u16; 2]) {
        let mut to_update = vec![WallPos::Interior(pos)];
        to_update.extend(
            [-1, 0, 1]
                .into_iter()
                .cartesian_product([-1, 0, 1])
                .flat_map(|(x, y)| (x != 0 || y != 0).then_some([x, y]))
                .flat_map(|delta| {
                    let mut ancestor_of_neighbor = self.neighbor(pos, delta);
                    if !self
                        .connected_components
                        .contains_key(&ancestor_of_neighbor)
                    {
                        return Vec::new();
                    };
                    let mut ancestors = vec![ancestor_of_neighbor];
                    while let Some(parent) = *self
                        .connected_components
                        .get(&ancestor_of_neighbor)
                        .expect("parents are in map")
                    {
                        ancestors.push(parent);
                        ancestor_of_neighbor = parent;
                    }
                    ancestors
                }),
        );
        let representative = to_update
            .last()
            .copied()
            .expect("vector started with one value");
        for ancestor in to_update {
            self.connected_components.insert(
                ancestor,
                (ancestor != representative).then_some(representative),
            );
        }
    }

    fn plot(&self) -> io::Result<()> {
        let mut stdout = stdout();
        let connected_compononents = self
            .connected_components
            .keys()
            .copied()
            .into_group_map_by(|pos| self.canonical_representative(*pos).expect("key in map"));
        for (representative, component) in connected_compononents {
            let hash = match representative {
                WallPos::TopRight => 1,
                WallPos::BottomLeft => 2,
                WallPos::Interior([x, y]) => 97 * (x as u32) + (y as u32) + 3,
            };
            let color = Color::Rgb {
                r: ((37 * hash + 32) & 255) as u8,
                g: ((149 * hash + 82) & 255) as u8,
                b: ((67 * hash + 191) & 255) as u8,
            };
            let content = '▉'.stylize().with(color);
            for pos in component {
                match pos {
                    WallPos::TopRight => {
                        for i in 0..=self.side_length {
                            execute!(
                                stdout,
                                MoveTo(i + 1, 0),
                                PrintStyledContent(content),
                                MoveTo(self.side_length + 2, i + 1),
                                PrintStyledContent(content)
                            )?;
                        }
                    }
                    WallPos::BottomLeft => {
                        for i in 0..=self.side_length {
                            execute!(
                                stdout,
                                MoveTo(i + 1, self.side_length + 2),
                                PrintStyledContent(content),
                                MoveTo(0, i + 1),
                                PrintStyledContent(content)
                            )?;
                        }
                    }
                    WallPos::Interior([x, y]) => {
                        execute!(stdout, MoveTo(x + 1, y + 1), PrintStyledContent(content))?;
                    }
                }
            }
        }
        execute!(stdout, MoveTo(0, self.side_length + 3))?;
        Ok(())
    }
}

fn level2_parametric(input: &str, side_length: u16) -> Option<(u16, u16)> {
    let plot = std::env::var_os("PLOT").is_some();
    if plot {
        execute!(stdout(), Clear(crossterm::terminal::ClearType::All), Hide).unwrap();
    }
    let mut connected_component_map = ConnectedComponentMap::new(side_length);
    for line in input.trim().lines() {
        let (x, y) = line.split_once(',').expect("line has comma");
        let x = x.parse().expect("parse");
        let y = y.parse().expect("parse");

        connected_component_map.add_interior_wall([x, y]);
        if plot {
            connected_component_map.plot().unwrap();
            std::thread::sleep(Duration::from_millis(3));
        }
        if connected_component_map.sides_connected() {
            if plot {
                execute!(
                    stdout(),
                    MoveTo(x + 1, y + 1),
                    PrintStyledContent('▉'.red()),
                    MoveTo(0, side_length + 3),
                    Show
                )
                .unwrap();
            }
            return Some((x, y));
        }
    }
    None
}

pub fn level2(input: &str) -> String {
    let (x, y) = level2_parametric(input, 70).expect("still can connect endpoints");
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
        assert_eq!(level2_parametric(test_input, 6), Some((6, 1)))
    }
}
