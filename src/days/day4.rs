use crate::util::prelude::*;

pub fn level1(input: &str) -> usize {
    let chars = input
        .lines()
        .map(|line| line.chars().collect_vec())
        .collect_vec();
    let needle = "XMAS".chars().collect_vec();
    let needle_reverse = needle.iter().rev().copied().collect_vec();

    search_grid(&needle, &chars) + search_grid(&needle_reverse, &chars)
}

fn search_grid<'a, T>(needle: &'a [T], haystack: &'a [Vec<T>]) -> usize
where
    &'a T: Eq,
{
    let horizontal: usize = haystack
        .iter()
        .map(|line| {
            line.windows(needle.len())
                .filter(|chunk| chunk.iter().zip(needle).all(|(left, right)| left == right))
                .count()
        })
        .sum();
    let vertical: usize = haystack
        .windows(needle.len())
        .map(|chunk| {
            (0..chunk[0].len())
                .filter(|i| {
                    chunk
                        .iter()
                        .map(|line| &line[*i])
                        .zip(needle)
                        .all(|(left, right)| left == right)
                })
                .count()
        })
        .sum();
    let diagonal: usize = haystack
        .windows(needle.len())
        .map(|chunk| -> usize {
            (0..=(chunk[0].len() - needle.len()))
                .map(|offset| {
                    let south_east = needle
                        .iter()
                        .zip(chunk.iter().enumerate().map(|(i, line)| &line[offset + i]))
                        .all(|(left, right)| left == right)
                        as usize;
                    let south_west = needle
                        .iter()
                        .zip(
                            chunk
                                .iter()
                                .enumerate()
                                .map(|(i, line)| &line[line.len() - (offset + i) - 1]),
                        )
                        .all(|(left, right)| left == right)
                        as usize;
                    south_east + south_west
                })
                .sum()
        })
        .sum();

    horizontal + vertical + diagonal
}

pub fn level2(input: &str) -> usize {
    let chars = input
        .lines()
        .map(|line| line.chars().collect_vec())
        .collect_vec();
    let needle = "MAS".chars().collect_vec();

    find_needle_cross(&needle, &chars)
}

fn find_needle_cross<'a, T>(needle: &'a [T], haystack: &'a [Vec<T>]) -> usize
where
    &'a T: Eq,
{
    let matches_needle = |matchee: &[&'a T]| -> bool {
        matchee
            .iter()
            .zip(needle)
            .all(|(left, right)| *left == right)
            || matchee
                .iter()
                .rev()
                .zip(needle)
                .all(|(left, right)| *left == right)
    };
    (0..=(haystack.len() - needle.len()))
        .map(|i| -> usize {
            (0..=(haystack[i].len() - needle.len()))
                .map(|j| -> usize {
                    let south_east = (0..needle.len())
                        .map(|k| &haystack[i + k][j + k])
                        .collect_vec();
                    let north_east = (0..needle.len())
                        .map(|k| &haystack[i + needle.len() - k - 1][j + k])
                        .collect_vec();
                    (matches_needle(&south_east) && matches_needle(&north_east)) as usize
                })
                .sum()
        })
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_small() {
        let test_input = include_str!("./test_input/day4_small.txt");
        assert_eq!(level1(test_input), 4)
    }

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day4.txt");
        assert_eq!(level1(test_input), 18)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day4.txt");
        assert_eq!(level2(test_input), 9)
    }
}
