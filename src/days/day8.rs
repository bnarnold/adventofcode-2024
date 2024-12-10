use std::collections::{HashMap, HashSet};

use crate::util::prelude::*;

fn gcd(mut x: usize, mut y: usize) -> usize {
    let bits = x.trailing_zeros().min(y.trailing_zeros());
    x >>= x.trailing_zeros();
    y >>= y.trailing_zeros();

    if x > y {
        std::mem::swap(&mut y, &mut x);
    }
    while x > 0 {
        x >>= x.trailing_zeros();
        let z = x;
        x = y.abs_diff(x);
        y = z;
    }
    y << bits
}

pub fn level1(input: &str) -> usize {
    let (map, width, height) = parse_input(input);

    let mut antinodes: HashSet<(usize, usize)> = HashSet::new();

    for antennas in map.values() {
        for (first, second) in antennas.iter().tuple_combinations() {
            let delta = (
                second.0 as isize - first.0 as isize,
                second.1 as isize - first.1 as isize,
            );
            antinodes.extend([-1, 2].into_iter().flat_map(|w| {
                let x = first.0.checked_add_signed(w * delta.0)?;
                let y = first.1.checked_add_signed(w * delta.1)?;
                (x < width && y < height).then_some((x, y))
            }));

            if (delta.0 % 3 == 0) && (delta.1 % 3 == 0) {
                antinodes.extend([1, 2].into_iter().map(|w| {
                    (
                        first.0.wrapping_add_signed(w * delta.0 / 3),
                        first.1.wrapping_add_signed(w * delta.1 / 3),
                    )
                }));
            }
        }
    }

    antinodes.len()
}

type AntennaMap = (HashMap<char, Vec<(usize, usize)>>, usize, usize);

fn parse_input(input: &str) -> AntennaMap {
    let mut map: HashMap<char, Vec<(usize, usize)>> = HashMap::new();
    let mut width = 0;
    let mut height = 0;

    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if y == 0 {
                width = x + 1;
            }
            if c != '.' {
                map.entry(c).or_default().push((x, y))
            };
            height = y + 1;
        }
    }
    (map, width, height)
}

pub fn level2(input: &str) -> usize {
    let (map, width, height) = parse_input(input);

    let mut antinodes: HashSet<(usize, usize)> = HashSet::new();

    for antennas in map.values() {
        for (first, second) in antennas.iter().tuple_combinations() {
            let delta = (
                second.0 as isize - first.0 as isize,
                second.1 as isize - first.1 as isize,
            );
            let d = gcd(delta.0.unsigned_abs(), delta.1.unsigned_abs()) as isize;
            let step = (delta.0 / d, delta.1 / d);
            antinodes.extend(
                (1..)
                    .map(|w| {
                        let x = first.0.checked_add_signed(-w * step.0)?;
                        let y = first.1.checked_add_signed(-w * step.1)?;
                        (x < width && y < height).then_some((x, y))
                    })
                    .while_some(),
            );
            antinodes.extend(
                (0..)
                    .map(|w| {
                        let x = first.0.checked_add_signed(w * step.0)?;
                        let y = first.1.checked_add_signed(w * step.1)?;
                        (x < width && y < height).then_some((x, y))
                    })
                    .while_some(),
            );
        }
    }

    antinodes.len()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day8.txt");
        assert_eq!(level1(test_input), 14)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day8.txt");
        assert_eq!(level2(test_input), 34)
    }
}
