use itertools::Itertools;
use std::collections::HashSet;
use std::io::prelude::*;

fn main() {
    let file_input = open_file().ok().unwrap();
    let mut reader = std::io::BufReader::new(file_input);

    let part1_result = part1(&mut reader);
    println!("part 1 result = {}", part1_result);

    reader.rewind().unwrap();

    let part2_result = part2(&mut reader);
    println!("part 2 result = {}", part2_result);
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day3/src/input.txt";
    let file_input = match std::fs::File::open(filepath_input) {
        Ok(file) => {
            println!("Opening {}", filepath_input);
            file
        }
        Err(_) => match std::fs::File::open(alt_filepath_input) {
            Ok(file) => {
                println!("Opening {}", alt_filepath_input);
                file
            }
            Err(e) => {
                println!(
                    "Unable to open input data file from {0} or {1}.",
                    filepath_input, alt_filepath_input
                );
                println!("{}", e);
                return Err(e);
            }
        },
    };
    Ok(file_input)
}

fn part2<R: std::io::BufRead>(reader: &mut R) -> usize {
    let total_sum: usize = reader
        .lines()
        .map(|line| {
            let hs: HashSet<usize> = line
                .unwrap()
                .chars()
                .map(|c| priority(c).unwrap())
                .collect();
            hs
        })
        .chunks(3)
        .into_iter()
        .map(|chunk| {
            chunk
                .reduce(|a, b| a.intersection(&b).copied().collect())
                .unwrap()
                .iter()
                .sum::<usize>()
        })
        .sum();
    // println!("Total sum of badge priorities = {}", total_sum);
    total_sum
}

fn part1<R: std::io::BufRead>(reader: &mut R) -> usize {
    let mut total_sum: usize = 0;
    for (num, line) in reader.lines().enumerate() {
        match line {
            Ok(line) => {
                let intersect: HashSet<usize> = get_intersect(&line);
                intersect.iter().for_each(|i| total_sum += *i);
            }
            Err(e) => {
                println!("Error reading input on line {}", num);
                println!("{}", e);
            }
        }
    }
    // println!("Total = {}", total_sum);
    total_sum
}

fn priority(c: char) -> Option<usize> {
    match c {
        'a'..='z' => Some((c as u8 - 97 + 1) as usize),
        'A'..='Z' => Some((c as u8 - 65 + 27) as usize),
        _ => None,
    }
}

fn get_intersect(s: &str) -> HashSet<usize> {
    let half_len = s.chars().count() / 2;
    let comp1: HashSet<usize> = s
        .chars()
        .take(half_len)
        .map(|c| priority(c).unwrap())
        .collect();
    let comp2: HashSet<usize> = s
        .chars()
        .rev()
        .take(half_len)
        .map(|c| priority(c).unwrap())
        .collect();

    let intersect: HashSet<usize> = comp1.intersection(&comp2).copied().collect();
    intersect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_works() {
        assert_eq!(priority('p'), Some(16));
    }

    #[test]
    fn get_intersect_works() {
        let s = "vJrwpWtwJgWrhcsFMMfFFhFp";
        let mut intersect = get_intersect(s);
        for i in intersect.drain() {
            assert_eq!(i, 16);
        }

        assert!(intersect.is_empty());
    }

    #[test]
    fn part1_works() {
        let s = "vJrwpWtwJgWrhcsFMMfFFhFp\njqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL\nPmmdzqPrVvPwwTWBwg\nwMqvLMZHhHMvwLHjbvcjnnSBnvTQFn\nttgJtRGJQctTZtZT\nCrZsJsPPZsGzwwsLwLmpwMDw";
        let mut reader = std::io::BufReader::new(s.as_bytes());
        let total_sum = part1(&mut reader);
        assert_eq!(total_sum, 157);
    }

    #[test]
    fn part2_works() {
        let s = "vJrwpWtwJgWrhcsFMMfFFhFp\njqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL\nPmmdzqPrVvPwwTWBwg\nwMqvLMZHhHMvwLHjbvcjnnSBnvTQFn\nttgJtRGJQctTZtZT\nCrZsJsPPZsGzwwsLwLmpwMDw";
        let mut reader = std::io::BufReader::new(s.as_bytes());
        let total_sum = part2(&mut reader);
        assert_eq!(total_sum, 70);
    }
}
