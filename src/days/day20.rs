use std::{
    collections::{HashMap, HashSet},
    io::stdout,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo},
    event, execute,
    style::{Color, PrintStyledContent, Stylize},
    terminal::Clear,
};
use nom::combinator::opt;
use rayon::prelude::*;

use crate::util::prelude::*;

#[derive(Debug)]
struct Map {
    /// The positions of the path in order.
    fields: Vec<[u16; 2]>,
    width: u16,
    height: u16,
    /// The positions grouped by the main diagonal, i.e. the difference of the coordinates.
    main_diagonals: Vec<HashSet<usize>>,
    /// The positions grouped by the off diagonal, i.e. the sum of the coordinates.
    off_diagonals: Vec<HashSet<usize>>,
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

        let mut visited = HashSet::new();
        let mut fields = vec![start_pos];

        let mut pos = start_pos;
        loop {
            let next_pos = [[0, -1], [1, 0], [0, 1], [-1, 0]]
                .into_iter()
                .flat_map(|delta| {
                    let new_pos = [
                        pos[0].checked_add_signed(delta[0])?,
                        pos[1].checked_add_signed(delta[1])?,
                    ];
                    (field_set.contains(&new_pos) && !visited.contains(&new_pos)).then_some(new_pos)
                })
                .next()
                .ok_or("no path")?;
            visited.insert(next_pos);
            fields.push(next_pos);
            if next_pos == end_pos {
                break;
            }
            pos = next_pos
        }

        let mut main_diagonals = Vec::new();
        let diagonal_count = (width + height - 1) as usize;
        main_diagonals.resize_with(diagonal_count, HashSet::new);
        let mut off_diagonals = Vec::new();
        off_diagonals.resize_with(diagonal_count, HashSet::new);

        for (i, &pos) in fields.iter().enumerate() {
            main_diagonals[(width - pos[0] - 1 + pos[1]) as usize].insert(i);
            off_diagonals[(pos[0] + pos[1]) as usize].insert(i);
        }

        Ok(Self {
            fields,
            width,
            height,
            main_diagonals,
            off_diagonals,
        })
    }

    fn cheat_savings(
        &self,
        cheat_distance: usize,
        min_gain: usize,
    ) -> impl ParallelIterator<Item = usize> + '_ {
        let plot = std::env::var_os("PLOT").is_some();

        if plot {
            execute!(stdout(), Clear(crossterm::terminal::ClearType::All), Hide).unwrap();
        }
        let main_diagonals = (0..self.main_diagonals.len())
            .map(|diag| {
                let min = diag.saturating_sub(cheat_distance);
                let max = (diag + cheat_distance).min(self.main_diagonals.len() - 1);
                self.main_diagonals[min..=max]
                    .iter()
                    .fold(HashSet::<usize>::new(), |acc, steps| &acc | steps)
            })
            .collect_vec();
        let off_diagonals = (0..self.off_diagonals.len())
            .map(|diag| {
                let min = diag.saturating_sub(cheat_distance);
                let max = (diag + cheat_distance).min(self.off_diagonals.len() - 1);
                self.off_diagonals[min..=max]
                    .iter()
                    .fold(HashSet::<usize>::new(), |acc, steps| &acc | steps)
            })
            .collect_vec();
        self.fields
            .par_iter()
            .enumerate()
            .flat_map(move |(step, pos)| {
                let main_diagonal = (self.width - 1 - pos[0] + pos[1]) as usize;
                let off_diagonal = (pos[0] + pos[1]) as usize;

                let reachable_on_main = &main_diagonals[main_diagonal];
                let reachable_on_off = &off_diagonals[off_diagonal];

                if plot {
                    let mut stdout = stdout().lock();
                    for (plot_step, plot_pos) in self.fields.iter().copied().enumerate() {
                        let color = match (
                            &plot_pos == pos,
                            reachable_on_main.contains(&plot_step),
                            reachable_on_off.contains(&plot_step),
                        ) {
                            (true, _, _) => Color::Blue,
                            (_, true, false) => Color::Red,
                            (_, false, true) => Color::Green,
                            (_, false, false) => Color::Grey,
                            (_, true, true) => {
                                let cheat_distance =
                                    pos[0].abs_diff(plot_pos[0]) + pos[1].abs_diff(plot_pos[1]);
                                if step + cheat_distance as usize + min_gain < plot_step {
                                    Color::Yellow
                                } else {
                                    let gain =
                                        plot_step.saturating_sub(step + cheat_distance as usize);
                                    let scaled = (64 + (128 * gain) / min_gain) as u8;

                                    Color::Rgb {
                                        r: scaled,
                                        g: scaled,
                                        b: scaled,
                                    }
                                }
                            }
                        };
                        execute!(
                            stdout,
                            MoveTo(plot_pos[0], plot_pos[1]),
                            PrintStyledContent('▉'.stylize().with(color))
                        )
                        .unwrap();
                    }
                    execute!(stdout, MoveTo(0, self.height + 1)).unwrap();
                    std::thread::sleep(Duration::from_millis(20));
                }
                let reachable_fields = reachable_on_main & reachable_on_off;
                reachable_fields
                    .into_par_iter()
                    .flat_map(move |reachable_step| {
                        let path_distance = reachable_step.checked_sub(step)?;
                        let target_pos = self.fields[reachable_step];
                        let cheat_distance =
                            pos[0].abs_diff(target_pos[0]) + pos[1].abs_diff(target_pos[1]);
                        path_distance.checked_sub(cheat_distance as usize)
                    })
                    .filter(move |saving| *saving >= min_gain)
            })
    }
}

pub fn level1(input: &str) -> usize {
    let map = Map::parse_input(input).expect("parse");
    map.cheat_savings(2, 100).count()
}

pub fn level2(input: &str) -> usize {
    let map = Map::parse_input(input).expect("parse");
    map.cheat_savings(20, 100).count()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day20.txt");
        let map = Map::parse_input(test_input).expect("parse");
        let cheats: Vec<usize> = map.cheat_savings(2, 1).collect();
        let counts = cheats.into_iter().counts();
        assert_eq!(counts.get(&64).copied(), Some(1));
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day20.txt");
        let map = Map::parse_input(test_input).expect("parse");
        let cheats: Vec<usize> = map.cheat_savings(20, 50).collect();
        let counts = cheats.into_iter().counts();
        assert_eq!(counts.get(&64).copied(), Some(19));
        assert_eq!(counts.get(&76).copied(), Some(3));
    }
}
