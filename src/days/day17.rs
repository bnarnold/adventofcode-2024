use core::panic;

use crate::util::prelude::*;

#[derive(Debug, Clone)]
struct Computer {
    registers: [u64; 3],
    instructions: Vec<u8>,
    eip: usize,
}

impl Computer {
    fn parse_input(input: &str) -> Result<Self, &'static str> {
        let mut lines = input.lines();
        let mut registers = [0; 3];
        for register in &mut registers {
            let line = lines.next().ok_or("too few registers")?;
            let (_, value) = line.split_once(": ").ok_or("unexpected line")?;
            *register = value.parse().map_err(|_| "not a number")?;
        }
        lines.next().expect("empty line separator");
        let (_, instructions_text) = lines
            .next()
            .ok_or("no instructions")?
            .split_once(": ")
            .ok_or("unexpected line")?;
        let instructions: Result<Vec<u8>, _> = instructions_text
            .split(',')
            .map(|x| -> Result<u8, _> { x.parse().map_err(|_| "not a number") })
            .collect();
        Ok(Self {
            registers,
            instructions: instructions?,
            eip: 0,
        })
    }

    fn get_first_output_bit(&self, register_a: u64) -> u8 {
        let cutoff = self.instructions.len() - 6;
        let mut iter = ComputerIterator {
            registers: [register_a, 0, 0],
            eip: 0,
            instructions: &self.instructions[..cutoff],
        };
        assert!(iter.next().is_none());
        (iter.registers[1] & 7) as u8
    }

    fn find_fixed_point(&self) -> Option<u64> {
        let mut candidates = vec![0];
        for instruction in self.instructions.iter().rev() {
            candidates = candidates
                .into_iter()
                .flat_map(|candidate| (0..8).map(move |byte| (candidate << 3) + byte as u64))
                .filter(|candidate| self.get_first_output_bit(*candidate) == *instruction)
                .collect()
        }
        candidates.into_iter().min()
    }
}

impl<'a> IntoIterator for &'a Computer {
    type Item = u8;

    type IntoIter = ComputerIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ComputerIterator {
            registers: self.registers,
            instructions: &self.instructions,
            eip: self.eip,
        }
    }
}

#[derive(Debug, Clone)]
struct ComputerIterator<'a> {
    registers: [u64; 3],
    instructions: &'a [u8],
    eip: usize,
}

impl ComputerIterator<'_> {
    fn combo_value(&self, input: u8) -> u64 {
        match input {
            0..4 => input as u64,
            4..7 => self.registers[input as usize - 4],
            _ => panic!("Unexpected operand"),
        }
    }

    fn adv(&mut self, input: u8) {
        let input = self.combo_value(input);
        self.registers[0] >>= input;
        self.eip += 2;
    }

    fn bxl(&mut self, input: u8) {
        self.registers[1] ^= input as u64;
        self.eip += 2;
    }

    fn bst(&mut self, input: u8) {
        self.registers[1] = self.combo_value(input) & 0b111;
        self.eip += 2;
    }

    fn jnz(&mut self, input: u8) {
        if self.registers[0] == 0 {
            self.eip += 2;
        } else {
            self.eip = input as usize
        }
    }

    fn bxc(&mut self, _input: u8) {
        self.registers[1] ^= self.registers[2];
        self.eip += 2;
    }

    fn out(&mut self, input: u8) -> u8 {
        self.eip += 2;
        (self.combo_value(input) & 0b111) as u8
    }

    fn bdv(&mut self, input: u8) {
        self.registers[1] = self.registers[0] >> self.combo_value(input);
        self.eip += 2
    }

    fn cdv(&mut self, input: u8) {
        self.registers[2] = self.registers[0] >> self.combo_value(input);
        self.eip += 2
    }

    fn step(&mut self) -> Option<Option<u8>> {
        let opcode = self.instructions.get(self.eip)?;
        let operand = *self.instructions.get(self.eip + 1)?;
        Some(match opcode {
            0 => {
                self.adv(operand);
                None
            }
            1 => {
                self.bxl(operand);
                None
            }
            2 => {
                self.bst(operand);
                None
            }
            3 => {
                self.jnz(operand);
                None
            }
            4 => {
                self.bxc(operand);
                None
            }
            5 => Some(self.out(operand)),
            6 => {
                self.bdv(operand);
                None
            }
            7 => {
                self.cdv(operand);
                None
            }
            _ => panic!("Unexpected opcode"),
        })
    }
}

impl Iterator for ComputerIterator<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(output) = self.step()? {
                break Some(output);
            }
        }
    }
}

pub fn level1(input: &str) -> String {
    let computer = Computer::parse_input(input).expect("parse");
    computer.into_iter().join(",")
}

pub fn level2(input: &str) -> String {
    let computer = Computer::parse_input(input).expect("parse");
    computer
        .find_fixed_point()
        .expect("no fixed point")
        .to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn level1_given_example_small() {
        let test_input = include_str!("./test_input/day17_small.txt");
        assert_eq!(&level1(test_input), "4,2,5,6,7,7,7,7,3,1,0")
    }

    #[test]
    fn level1_given_example() {
        let test_input = include_str!("./test_input/day17.txt");
        assert_eq!(&level1(test_input), "4,6,3,5,6,3,5,2,1,0")
    }
}
