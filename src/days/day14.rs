use std::{
    io::{self, stdout, Write},
    time::Duration,
    usize,
};

use crossterm::{
    cursor::{self, MoveTo},
    event::{KeyEvent, KeyModifiers},
    execute,
    style::{PrintStyledContent, StyledContent, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType, SetSize},
    ExecutableCommand, QueueableCommand,
};
use nom::{
    character::complete::{i32, newline},
    combinator::eof,
    sequence::separated_pair,
    IResult, Parser,
};
use nom_supreme::{
    error::ErrorTree, final_parser::final_parser, multi::collect_separated_terminated,
    tag::complete::tag, ParserExt,
};

use crate::util::prelude::*;

#[derive(Debug, Clone)]
struct Robot {
    position: [i32; 2],
    velocity: [i32; 2],
}

impl Robot {
    fn step(&mut self, seconds: i32, bounds: [i32; 2]) {
        self.position = [0, 1].map(|i| {
            let result = (self.position[i] + seconds * self.velocity[i]) % bounds[i];
            if result < 0 {
                result + bounds[i]
            } else {
                result
            }
        })
    }

    fn quadrant(&self, bounds: [i32; 2]) -> Option<[i32; 2]> {
        let mut result = [0; 2];
        for i in 0..2 {
            result[i] = match (2 * self.position[i] + 1).cmp(&bounds[i]) {
                std::cmp::Ordering::Less => Some(0),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(1),
            }?
        }
        Some(result)
    }
}

fn parse_robots(input: &str) -> Result<Vec<Robot>, ErrorTree<&str>> {
    fn robot_parser(input: &str) -> IResult<&str, Robot, ErrorTree<&str>> {
        separated_pair(
            separated_pair(i32, tag(","), i32)
                .preceded_by(tag("p="))
                .context("position"),
            tag(" "),
            separated_pair(i32, tag(","), i32)
                .preceded_by(tag("v="))
                .context("velocity"),
        )
        .map(|(p, v)| Robot {
            position: [p.0, p.1],
            velocity: [v.0, v.1],
        })
        .context("robot")
        .parse(input)
    }
    final_parser(collect_separated_terminated(
        robot_parser,
        newline,
        eof.opt_preceded_by(newline),
    ))(input)
}

fn predict_positions(input: &str, width: i32, height: i32) -> usize {
    let robots = parse_robots(input).expect("parse");
    let steps = 100;
    let bounds = [width, height];
    robots
        .into_iter()
        .flat_map(|mut robot| {
            robot.step(steps, bounds);
            robot.quadrant(bounds)
        })
        .counts()
        .into_values()
        .product()
}

pub fn level1(input: &str) -> usize {
    predict_positions(input, 101, 103)
}

fn print_robots<'a>(robots: impl IntoIterator<Item = &'a Robot>, step: i32) -> io::Result<()> {
    let mut stdout = stdout();
    stdout.execute(Clear(ClearType::All))?;
    for robot in robots {
        execute!(
            stdout,
            MoveTo(robot.position[0] as u16, robot.position[1] as u16 + 1,),
            PrintStyledContent("█".green()),
            MoveTo(0, 0),
            PrintStyledContent(step.to_string().red()),
        )?;
    }
    stdout.flush()?;
    Ok(())
}

fn robot_explorer(input: &str, width: i32, height: i32) -> io::Result<usize> {
    let robots = parse_robots(input).expect("parse");
    let bounds = [width, height];
    let mut step = 0;
    let size = crossterm::terminal::size()?;
    execute!(io::stdout(), SetSize(width as u16, height as u16))?;
    enable_raw_mode()?;
    let result = 'steps: loop {
        let mut robots = robots.clone();
        for robot in &mut robots {
            robot.step(step, bounds);
        }
        print_robots(&robots, step).unwrap();
        let next = loop {
            if let crossterm::event::Event::Key(event) = crossterm::event::read().unwrap() {
                let mut step_size = 1;
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    step_size *= bounds[0];
                }
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    step_size *= bounds[1];
                }
                match event.code {
                    crossterm::event::KeyCode::Left => break -step_size,
                    crossterm::event::KeyCode::Right => break step_size,
                    crossterm::event::KeyCode::Esc => break 'steps step,
                    _ => {}
                }
            }
        };
        step += next;
        for robot in &mut robots {
            robot.step(next as i32, bounds);
        }
    };
    disable_raw_mode()?;
    execute!(io::stdout(), Clear(ClearType::All), SetSize(size.0, size.1))?;
    Ok(result as usize)
}

pub fn level2(input: &str) -> usize {
    robot_explorer(input, 101, 103).expect("interactive search failed")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day14.txt");
        assert_eq!(predict_positions(test_input, 11, 7), 12)
    }
}
