use std::collections::{HashMap, HashSet};

pub fn level1(input: &str) -> usize {
    let mut graph: HashMap<&str, HashSet<&str>> = HashMap::new();
    for line in input.lines() {
        let (start, end) = line.split_once('-').expect("parse");
        graph.entry(start).or_default().insert(end);
        graph.entry(end).or_default().insert(start);
    }

    let mut result = 0;
    for (node, neighbors) in &graph {
        if !node.starts_with('t') {
            continue;
        }
        for neighbor in neighbors {
            let common = neighbors & graph.get(neighbor).unwrap_or(&HashSet::new());
            for third in common {
                // each triengle gets visited 6 times.
                // I count it if the distinguished vertex has a t.
                // If all are t, I need a factor of 1/6, as all 6 visits contribute.
                // If two are t, 4 visits contribute, so 1/4.
                // If one is t, 2 visits contribute, so 1/2.
                let factor = match (neighbor.starts_with('t'), third.starts_with('t')) {
                    (true, true) => 12 / 6,
                    (true, false) => 12 / 4,
                    (false, true) => 12 / 4,
                    (false, false) => 12 / 2,
                };
                result += factor
            }
        }
    }
    debug_assert_eq!(result % 12, 0);
    result / 12
}

pub fn level2(input: &str) -> usize {
    0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day23.txt");
        assert_eq!(level1(test_input), 7)
    }

    #[test]
    fn level1_two() {
        let test_input = "ta-ab\nab-tb\nta-tb";
        assert_eq!(level1(test_input), 1)
    }

    #[test]
    fn level1_all() {
        let test_input = "ta-tb\ntb-tc\nta-tc";
        assert_eq!(level1(test_input), 1)
    }

    #[test]
    fn level1_one_vertex() {
        let test_input = "ta-ab\nab-ac\nta-ac";
        assert_eq!(level1(test_input), 1)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day23.txt");
        assert_eq!(level2(test_input), 0)
    }
}
