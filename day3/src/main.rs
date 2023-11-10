use std::collections::HashSet;
use std::io::prelude::*;

fn main() {
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
                return;
            }
        },
    };

    let reader = std::io::BufReader::new(file_input);
    let mut total_sum_priority: isize = 0;
    for (num, line) in reader.lines().enumerate() {
        match line {
            Ok(line) => {
                let intersect: HashSet<usize> = get_intersect(&line);
                intersect
                    .iter()
                    .for_each(|i| total_sum_priority += *i as isize);
            }
            Err(e) => {
                println!("Error reading input on line {}", num);
                println!("{}", e);
                return;
            }
        }
    }

    println!("Total = {}", total_sum_priority);
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
    fn sum_intersects_works() {
        let s = "vJrwpWtwJgWrhcsFMMfFFhFp\njqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL\nPmmdzqPrVvPwwTWBwg\nwMqvLMZHhHMvwLHjbvcjnnSBnvTQFn\nttgJtRGJQctTZtZT\nCrZsJsPPZsGzwwsLwLmpwMDw";
        let mut total_sum_priority: isize = 0;
        for line in s.lines() {
            let intersect: HashSet<usize> = get_intersect(&line);
            intersect
                .iter()
                .for_each(|i| total_sum_priority += *i as isize);
        }
        assert_eq!(total_sum_priority, 157);
    }
}
