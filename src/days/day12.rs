use std::collections::VecDeque;

use crate::util::prelude::*;

pub fn level1(input: &str) -> usize {
    let map = input
        .lines()
        .map(|line| line.chars().collect_vec())
        .collect_vec();
    let width = map[0].len();
    let height = map.len();

    let mut result = 0;
    let mut visited = vec![vec![false; width]; height];

    for (y, row) in map.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            let mut area = 0;
            let mut perimeter = 0;
            let mut queue = VecDeque::new();
            queue.push_back((x, y));
            while let Some((x, y)) = queue.pop_front() {
                if visited[y][x] {
                    continue;
                }
                visited[y][x] = true;
                let neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .into_iter()
                    .flat_map(|(dx, dy)| {
                        let next_x = x.checked_add_signed(dx)?;
                        let next_y = y.checked_add_signed(dy)?;
                        (map.get(next_y)?.get(next_x)? == c).then_some((next_x, next_y))
                    })
                    .collect_vec();
                area += 1;
                perimeter += 4 - neighbors.len();

                queue.extend(neighbors);
            }
            result += area * perimeter;
        }
    }
    result
}

pub fn level2(input: &str) -> usize {
    let map = input
        .lines()
        .map(|line| line.chars().collect_vec())
        .collect_vec();
    let width = map[0].len();
    let height = map.len();

    let mut result = 0;
    let mut visited = vec![vec![false; width]; height];

    for (y, row) in map.iter().enumerate() {
        for (x, c) in row.iter().enumerate() {
            let mut area = 0;
            let mut corners = 0;
            let mut queue = VecDeque::new();
            queue.push_back((x, y));
            while let Some((x, y)) = queue.pop_front() {
                if visited[y][x] {
                    continue;
                }
                visited[y][x] = true;
                let (neighbor_pos, neighbors): (Vec<_>, Vec<_>) =
                    [(1, 0), (0, 1), (-1, 0), (0, -1)]
                        .into_iter()
                        .enumerate()
                        .flat_map(|(i, (dx, dy))| {
                            let next_x = x.checked_add_signed(dx)?;
                            let next_y = y.checked_add_signed(dy)?;
                            (map.get(next_y)?.get(next_x)? == c).then_some((i, (next_x, next_y)))
                        })
                        .unzip();
                area += 1;
                let convex_corners = (0..4)
                    .filter(|corner_pos| {
                        !(neighbor_pos.contains(corner_pos)
                            || neighbor_pos.contains(&((corner_pos + 1) % 4)))
                    })
                    .count();
                let concave_corners = [-1, 1]
                    .into_iter()
                    .cartesian_product([-1, 1])
                    .flat_map(|(dx, dy)| {
                        let next_x = x.checked_add_signed(dx)?;
                        let next_y = y.checked_add_signed(dy)?;
                        (map.get(next_y)?.get(next_x)? != c
                            && map.get(y)?.get(next_x)? == c
                            && map.get(next_y)?.get(x)? == c)
                            .then_some(())
                    })
                    .count();
                corners += convex_corners + concave_corners;

                queue.extend(neighbors.into_iter().filter(|(x, y)| !visited[*y][*x]));
            }
            result += area * corners;
        }
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example_small() {
        let test_input = include_str!("./test_input/day12.txt");
        assert_eq!(level1(test_input), 140)
    }

    #[test]
    fn level1_given_example_holes() {
        let test_input = include_str!("./test_input/day12_holes.txt");
        assert_eq!(level1(test_input), 772)
    }

    #[test]
    fn level1_given_example_large() {
        let test_input = include_str!("./test_input/day12_large.txt");
        assert_eq!(level1(test_input), 1930)
    }

    #[test]
    fn level2_given_example_touching_holes() {
        let test_input = include_str!("./test_input/day12_touching_holes.txt");
        assert_eq!(level2(test_input), 368)
    }

    #[test]
    fn level2_given_example_large() {
        let test_input = include_str!("./test_input/day12_large.txt");
        assert_eq!(level2(test_input), 1206)
    }
}
