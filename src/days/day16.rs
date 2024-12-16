use core::panic;
use std::{
    cmp::Reverse,
    collections::{
        binary_heap::BinaryHeap, hash_map::Entry, BTreeMap, BTreeSet, HashMap, HashSet, VecDeque,
    },
    io::{self, stdout},
    iter::repeat,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::{Color, PrintStyledContent, Stylize},
    terminal::{Clear, ClearType, SetSize, WindowSize},
};

use crate::util::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}

impl Direction {
    fn step(&self, pos: [u16; 2]) -> [u16; 2] {
        let delta = match self {
            Direction::Up => [0, -1],
            Direction::Right => [1, 0],
            Direction::Down => [0, 1],
            Direction::Left => [-1, 0],
        };
        [0, 1].map(|i| pos[i].checked_add_signed(delta[i]).expect("overflow"))
    }
}

type Vertex = ([u16; 2], Direction);

#[derive(Debug)]
struct Map {
    start_pos: [u16; 2],
    end_pos: [u16; 2],
    walls: HashSet<[u16; 2]>,
}

impl Map {
    fn parse_input(input: &str) -> Self {
        let mut start_pos = None;
        let mut end_pos = None;
        let mut walls = HashSet::new();

        for (y, line) in input.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let pos = [x as u16, y as u16];
                match c {
                    '#' => {
                        walls.insert(pos);
                    }
                    'S' => {
                        start_pos = Some(pos);
                    }
                    'E' => {
                        end_pos = Some(pos);
                    }
                    '.' => {}
                    c => panic!("Unexpected map input: {c}"),
                }
            }
        }

        Self {
            start_pos: start_pos.expect("No start in map"),
            end_pos: end_pos.expect("No end in map"),
            walls,
        }
    }

    fn parents(&self) -> HashMap<Vertex, (usize, Option<Vertex>)> {
        let mut queue = BinaryHeap::new();
        queue.push((Reverse(0), self.start_pos, Direction::Right, None));
        let mut parents = HashMap::new();

        while let Some((Reverse(score), pos, direction, parent)) = queue.pop() {
            let Entry::Vacant(e) = parents.entry((pos, direction)) else {
                continue;
            };
            e.insert((score, parent));
            if pos == self.end_pos {
                break;
            }
            for new_direction in [
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ] {
                if new_direction == direction.opposite() {
                    continue;
                }
                let (new_pos, new_direction, score_delta) = if direction == new_direction {
                    (direction.step(pos), direction, 1)
                } else {
                    (pos, new_direction, 1000)
                };
                if !parents.contains_key(&(new_pos, new_direction))
                    && !self.walls.contains(&new_pos)
                {
                    queue.push((
                        Reverse(score + score_delta),
                        new_pos,
                        new_direction,
                        Some((pos, direction)),
                    ))
                }
            }
        }
        parents
    }

    fn check_parents(&self, parents: &HashMap<Vertex, (usize, Option<Vertex>)>) {
        let width = self.walls.iter().copied().map(|pos| pos[0]).max().unwrap() + 1;
        let height = self.walls.iter().copied().map(|pos| pos[1]).max().unwrap() + 1;

        let directions = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];
        for x in 0..width {
            for y in 0..height {
                for dir in directions {
                    let key = ([x, y], dir);
                    if key == (self.start_pos, Direction::Right) {
                        assert_eq!(parents.get(&key).copied(), Some((0, None)));
                        continue;
                    }
                    if self.walls.contains(&[x, y]) {
                        assert!(!parents.contains_key(&([x, y], dir)));
                        continue;
                    }
                    let mut neighbors = directions
                        .into_iter()
                        .filter(|new_dir| *new_dir != dir && *new_dir != dir.opposite())
                        .map(|new_dir| ([x, y], new_dir, 1000))
                        .collect_vec();
                    neighbors.push((dir.opposite().step([x, y]), dir, 1));
                    if let Some((score, _)) = parents.get(&([x, y], dir)).copied() {
                        assert_eq!(
                            score,
                            neighbors
                                .into_iter()
                                .flat_map(|(new_pos, new_dir, score_delta)| {
                                    let new_score = parents.get(&(new_pos, new_dir)).copied()?.0;
                                    Some(new_score + score_delta)
                                })
                                .min()
                                .expect("One neighbor must be visited")
                        );
                    };
                }
            }
        }
    }
}

fn score_to_color(score: usize, max_score: usize) -> Color {
    let turns = score / 1000;
    let steps = score % 1000;
    let max_turns = max_score / 1000;
    let max_steps = max_score % 1000;

    Color::Rgb {
        r: (127 + (128 * turns) / max_turns) as u8,
        g: (127 + (128 * steps) / max_steps) as u8,
        b: 127,
    }
}

pub fn level1_visualizer(input: &str, plot: bool) -> io::Result<usize> {
    let map = Map::parse_input(input);
    let parents = map.parents();

    #[cfg(debug_assertions)]
    map.check_parents(&parents);

    let mut dir = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ]
    .into_iter()
    .find(|direction| {
        if let Some((_, Some((parent_pos, _)))) = parents.get(&(map.end_pos, *direction)) {
            *parent_pos != map.end_pos
        } else {
            false
        }
    })
    .expect("end position not reachable");

    let mut stdout = stdout();
    let size = crossterm::terminal::size()?;

    if plot {
        execute!(
            stdout,
            Clear(ClearType::All),
            Hide,
            SetSize(
                map.walls.iter().copied().map(|pos| pos[0]).max().unwrap(),
                map.walls.iter().copied().map(|pos| pos[1]).max().unwrap(),
            )
        )?;
        for pos in &map.walls {
            execute!(
                stdout,
                MoveTo(pos[0], pos[1]),
                PrintStyledContent('#'.grey())
            )?
        }
    }

    let mut pos = map.end_pos;
    let total_score = parents
        .get(&(map.end_pos, dir))
        .expect("end position not reachable")
        .0;
    if plot {
        let mut score = 0;
        while let Some((current_score, Some((next_pos, next_dir)))) =
            parents.get(&(pos, dir)).copied()
        {
            let c = match dir {
                Direction::Up => '^',
                Direction::Right => '>',
                Direction::Down => 'v',
                Direction::Left => '<',
            };
            execute!(
                stdout,
                MoveTo(pos[0], pos[1]),
                PrintStyledContent(c.stylize().with(score_to_color(current_score, total_score)))
            )?;

            debug_assert_eq!(total_score, score + current_score);
            score += if pos == next_pos { 1000 } else { 1 };
            pos = next_pos;
            dir = next_dir;
        }
        debug_assert_eq!((pos, dir), (map.start_pos, Direction::Right));

        execute!(
            stdout,
            MoveTo(
                0,
                map.walls.iter().copied().map(|pos| pos[1]).max().unwrap() + 1,
            ),
            Show,
            SetSize(size.0, size.1)
        )?;
    }
    Ok(total_score)
}

pub fn level1(input: &str) -> usize {
    level1_visualizer(input, std::env::var_os("PLOT").is_some()).expect("plot")
}

pub fn level2_visualizer(input: &str, plot: bool) -> io::Result<usize> {
    let map = Map::parse_input(input);
    let parents = map.parents();

    let mut stdout = stdout();
    let size = crossterm::terminal::size()?;

    if plot {
        execute!(
            stdout,
            Clear(ClearType::All),
            Hide,
            SetSize(
                map.walls.iter().copied().map(|pos| pos[0]).max().unwrap(),
                map.walls.iter().copied().map(|pos| pos[1]).max().unwrap(),
            )
        )?;
        for pos in &map.walls {
            execute!(
                stdout,
                MoveTo(pos[0], pos[1]),
                PrintStyledContent('#'.grey())
            )?
        }
    }

    let dir = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ]
    .into_iter()
    .find(|direction| {
        if let Some((_, Some((parent_pos, _)))) = parents.get(&(map.end_pos, *direction)) {
            *parent_pos != map.end_pos
        } else {
            false
        }
    })
    .expect("end position not reachable");
    let total_score = parents
        .get(&(map.end_pos, dir))
        .expect("end position not reachable")
        .0;

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((map.end_pos, dir));

    while let Some((pos, dir)) = queue.pop_front() {
        visited.insert(pos);
        let current_score = parents.get(&(pos, dir)).expect("was visited").0;
        let c = match dir {
            Direction::Up => '^',
            Direction::Right => '>',
            Direction::Down => 'v',
            Direction::Left => '<',
        };
        if plot {
            execute!(
                stdout,
                MoveTo(pos[0], pos[1]),
                PrintStyledContent(c.stylize().with(score_to_color(current_score, total_score)))
            )?;
        }
        let directions = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];
        let mut neighbors = directions
            .into_iter()
            .filter(|new_dir| *new_dir != dir && *new_dir != dir.opposite())
            .map(|new_dir| (pos, new_dir, 1000))
            .collect_vec();
        neighbors.push((dir.opposite().step(pos), dir, 1));

        queue.extend(
            neighbors
                .into_iter()
                .flat_map(|(new_pos, new_dir, score_delta)| {
                    let new_score = parents.get(&(new_pos, new_dir))?.0;
                    (new_score + score_delta == current_score).then_some((new_pos, new_dir))
                }),
        );
    }
    if plot {
        execute!(
            stdout,
            MoveTo(
                0,
                map.walls.iter().copied().map(|pos| pos[1]).max().unwrap() + 1,
            ),
            Show,
            SetSize(size.0, size.1)
        )?;
    }
    Ok(visited.len())
}

pub fn level2(input: &str) -> usize {
    level2_visualizer(input, std::env::var_os("PLOT").is_some()).expect("plot")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example_first() {
        let test_input = include_str!("./test_input/day16.txt");
        assert_eq!(level1(test_input), 7036)
    }

    #[test]
    fn level1_given_example_zigzag() {
        let test_input = include_str!("./test_input/day16_zigzag.txt");
        assert_eq!(level1(test_input), 21148)
    }

    #[test]
    fn level1_given_example_large() {
        let test_input = include_str!("./test_input/day16_large.txt");
        assert_eq!(level1(test_input), 11048)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day16.txt");
        assert_eq!(level2(test_input), 45)
    }

    #[test]
    fn level2_given_example_large() {
        let test_input = include_str!("./test_input/day16_large.txt");
        assert_eq!(level2(test_input), 64)
    }

    #[test]
    fn level2_given_example_zigzag() {
        let test_input = include_str!("./test_input/day16_zigzag.txt");
        assert_eq!(level2(test_input), 149)
    }
}
