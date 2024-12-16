use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    io::{self, stdout, Write},
    iter::repeat,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::{PrintStyledContent, Stylize},
    terminal::{Clear, ClearType},
};

use crate::util::prelude::*;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn step(&self, pos: [i16; 2]) -> [i16; 2] {
        let delta = match self {
            Direction::Up => [0, -1],
            Direction::Right => [1, 0],
            Direction::Down => [0, 1],
            Direction::Left => [-1, 0],
        };
        [0, 1].map(|i| pos[i] + delta[i])
    }
}

impl TryFrom<char> for Direction {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '^' => Ok(Direction::Up),
            '>' => Ok(Direction::Right),
            'v' => Ok(Direction::Down),
            '<' => Ok(Direction::Left),
            c => Err(format!("unexpected character: {c}")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Cell {
    Wall,
    Box,
}

struct Map {
    robot_pos: [i16; 2],
    cells: HashMap<[i16; 2], Cell>,
    cell_width: i16,
}

impl Map {
    fn parse_input(input: &str, cell_width: i16) -> Self {
        let mut robot_pos = None;
        let mut cells = HashMap::new();

        for (y, line) in input.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let pos = [cell_width * x as i16, y as i16];
                match c {
                    'O' => {
                        cells.insert(pos, Cell::Box);
                    }
                    '#' => {
                        cells.insert(pos, Cell::Wall);
                    }
                    '@' => {
                        robot_pos = Some(pos);
                    }
                    '.' => {}
                    c => panic!("Unexpected map input: {c}"),
                }
            }
        }
        Self {
            robot_pos: robot_pos.expect("No robot in map"),
            cells,
            cell_width,
        }
    }
    fn move_robot(&mut self, direction: Direction) -> Vec<[i16; 2]> {
        let next_robot_pos = direction.step(self.robot_pos);

        let mut moved_box_positions = Vec::new();
        let mut collision_positions = vec![self.robot_pos];
        let mut first = true;

        while !collision_positions.is_empty() {
            let potential_cell_positions: HashSet<_> = collision_positions
                .iter()
                .flat_map(|pos| {
                    let range_end = pos[0] + if first { 1 } else { self.cell_width };
                    (pos[0] - self.cell_width + 1..range_end)
                        .map(move |x| direction.step([x, pos[1]]))
                })
                .collect();
            if potential_cell_positions
                .iter()
                .any(|pos| matches!(self.cells.get(pos), Some(Cell::Wall)))
            {
                if !first {
                    self.cells
                        .extend(moved_box_positions.into_iter().map(|pos| (pos, Cell::Box)));
                    self.cells
                        .extend(collision_positions.into_iter().map(|pos| (pos, Cell::Box)));
                }
                return Vec::new();
            }
            if !first {
                moved_box_positions.extend(collision_positions);
            }
            collision_positions = potential_cell_positions
                .into_iter()
                .filter(|pos| self.cells.remove(pos).is_some())
                .collect();
            first = false;
        }

        let mut to_clear = Vec::new();
        for moved_box_position in moved_box_positions {
            let next_position = direction.step(moved_box_position);
            to_clear.push(moved_box_position);

            self.cells.insert(next_position, Cell::Box);
        }
        to_clear.extend([self.robot_pos, next_robot_pos]);
        self.robot_pos = next_robot_pos;

        to_clear
    }

    fn gps_score(&self) -> i32 {
        self.cells
            .iter()
            .flat_map(|(pos, cell)| {
                matches!(cell, Cell::Box).then_some((pos[0] + 100 * pos[1]) as i32)
            })
            .sum()
    }

    fn plot(&self) -> io::Result<()> {
        let mut stdout = stdout();
        for (pos, cell) in &self.cells {
            let cell_content = match cell {
                Cell::Wall => "#".repeat(self.cell_width as usize).grey(),
                Cell::Box => {
                    if self.cell_width == 1 {
                        "O".to_string().yellow()
                    } else {
                        let inner = "=".repeat(self.cell_width as usize - 2);
                        format!("[{inner}]").yellow()
                    }
                }
            };
            execute!(
                stdout,
                MoveTo(pos[0] as u16, pos[1] as u16),
                PrintStyledContent(cell_content),
            )?
        }
        execute!(
            stdout,
            MoveTo(self.robot_pos[0] as u16, self.robot_pos[1] as u16),
            PrintStyledContent('@'.green()),
        )?;
        stdout.flush()?;
        Ok(())
    }
}

pub fn move_boxes(input: &str, cell_width: i16) -> i32 {
    let plot = std::env::var_os("PLOT").is_some();
    let (map_input, direction_input) = input.split_once("\n\n").expect("separator of inputs");
    let mut map = Map::parse_input(map_input, cell_width);
    if plot {
        execute!(stdout(), Clear(ClearType::All)).expect("clear");
    }
    for c in direction_input.lines().flat_map(|line| line.chars()) {
        let direction: Direction = c.try_into().expect("direction char");
        let modified = map.move_robot(direction);
        if plot {
            for pos in modified {
                let clear_text = " ".repeat(map.cell_width as usize);
                execute!(
                    stdout(),
                    MoveTo(pos[0] as u16, pos[1] as u16),
                    PrintStyledContent(clear_text.stylize())
                )
                .expect("clear");
            }
            map.plot().expect("plot");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    if plot {
        execute!(stdout(), Clear(ClearType::All), Show).expect("clear");
    }
    map.gps_score()
}

pub fn level1(input: &str) -> i32 {
    move_boxes(input, 1)
}
pub fn level2(input: &str) -> i32 {
    move_boxes(input, 2)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day15.txt");
        assert_eq!(level1(test_input), 2028)
    }

    #[test]
    fn level1_given_example_large() {
        let test_input = include_str!("./test_input/day15_large.txt");
        assert_eq!(level1(test_input), 10092)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day15_large.txt");
        assert_eq!(level2(test_input), 9021)
    }
}
