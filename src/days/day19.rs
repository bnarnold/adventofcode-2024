use std::collections::{HashMap, VecDeque};

use crate::util::prelude::*;

pub fn level1(input: &str) -> usize {
    let mut lines = input.trim().lines();

    let design_line = lines.next().expect("designs");
    let pattern = design_line.replace(", ", "|");
    let r = regex::Regex::new(&format!("^({})+$", pattern)).expect("compile regex");

    lines.filter(|line| r.is_match(line)).count()
}

fn match_count<'a>(
    haystack: &'a str,
    needles: &[&str],
    cache: &mut HashMap<&'a str, usize>,
) -> usize {
    if let Some(result) = cache.get(haystack) {
        return *result;
    }
    let count = needles
        .iter()
        .flat_map(|design| haystack.strip_prefix(design))
        .map(|tail| match_count(tail, needles, cache))
        .sum();
    cache.insert(haystack, count);
    count
}

pub fn level2(input: &str) -> usize {
    let mut lines = input.trim().lines();

    let designs = lines.next().expect("designs").split(", ").collect_vec();

    let mut cache = HashMap::new();
    cache.insert("", 1);

    lines
        .filter(|line| !line.is_empty())
        .map(|line| match_count(line, &designs, &mut cache))
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day19.txt");
        assert_eq!(level1(test_input), 6)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day19.txt");
        assert_eq!(level2(test_input), 16)
    }
}
