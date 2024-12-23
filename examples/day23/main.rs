use aoc::{days::day23, util::infra::*};

fn main() {
    let (level, should_submit) = parse_args().unwrap();
    let input = include_str!("../../input/day23.txt");
    match level {
        Level::One => println!("{}", day23::level1(input)),
        Level::Two => println!("{}", day23::level2(input)),
    };
}
