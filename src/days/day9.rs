use core::panic;
use std::num::NonZeroUsize;

use crate::util::prelude::*;

pub fn level1(input: &str) -> usize {
    let mut running_length = 0;
    let (mut file_ids_and_lengths, mut gaps): (Vec<_>, Vec<_>) = input
        .trim_end()
        .chars()
        .map(|c| (c.to_digit(10).expect("not a digit") as usize))
        .chunks(2)
        .into_iter()
        .enumerate()
        .map(|(i, mut c)| {
            let file_length = c.next().expect("chunk nonempty");
            let file_description = (i, file_length, running_length);
            running_length += file_length;

            let gap_length = c.next().unwrap_or_default();
            let gap_description = (gap_length, running_length);
            running_length += gap_length;
            (file_description, gap_description)
        })
        .unzip();
    gaps.reverse();
    let mut moved_file_ids_and_lengths = Vec::new();
    'outer: loop {
        let (mut gap_len, mut gap_pos) = gaps.pop().expect("checked gaps nonempty");
        while gap_len > 0 {
            let (file_id, file_length, file_pos) = file_ids_and_lengths.last_mut().unwrap();
            if *file_pos < gap_pos {
                break 'outer;
            }
            let moved_places = gap_len.min(*file_length);
            moved_file_ids_and_lengths.push((*file_id, moved_places, gap_pos));
            gap_len -= moved_places;
            *file_length -= moved_places;
            gap_pos += moved_places;
            if *file_length == 0 {
                file_ids_and_lengths.pop();
            }
        }
    }
    file_ids_and_lengths.extend(moved_file_ids_and_lengths);
    file_ids_and_lengths
        .into_iter()
        .map(|(id, length, pos)| id * (length * (2 * pos + length - 1)) / 2)
        .sum()
}

pub fn level2(input: &str) -> usize {
    let mut running_length = 0;
    let (file_ids_and_lengths, mut gaps): (Vec<_>, Vec<_>) = input
        .trim_end()
        .chars()
        .map(|c| (c.to_digit(10).expect("not a digit") as usize))
        .chunks(2)
        .into_iter()
        .enumerate()
        .map(|(i, mut c)| {
            let file_length = c.next().expect("chunk nonempty");
            let file_description = (i, file_length, running_length);
            running_length += file_length;

            let gap_length = c.next().unwrap_or_default();
            let gap_description = (gap_length, running_length);
            running_length += gap_length;
            (file_description, gap_description)
        })
        .unzip();
    let mut moved_file_ids_and_lengths = Vec::new();
    for file_info @ (file_id, file_length, file_pos) in file_ids_and_lengths.into_iter().rev() {
        let gap = gaps
            .iter_mut()
            .take_while(|(_, gap_pos)| *gap_pos < file_pos)
            .find(|(gap_len, _)| *gap_len >= file_length);
        let Some((gap_len, gap_pos)) = gap else {
            moved_file_ids_and_lengths.push(file_info);
            continue;
        };
        moved_file_ids_and_lengths.push((file_id, file_length, *gap_pos));
        *gap_len -= file_length;
        *gap_pos += file_length;
    }
    moved_file_ids_and_lengths
        .into_iter()
        .map(|(id, length, pos)| id * (length * (2 * pos + length - 1)) / 2)
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day9.txt");
        assert_eq!(level1(test_input), 1928)
    }

    #[test]
    fn level2_given_example() {
        let test_input = include_str!("./test_input/day9.txt");
        assert_eq!(level2(test_input), 2858)
    }
}
