use std::io::prelude::*;

fn main() {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day1/src/input.txt";
    let file_input = match std::fs::File::open(filepath_input) {
        Ok(file) => file,
        Err(_) => match std::fs::File::open(alt_filepath_input) {
            Ok(file) => file,
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

    let mut elf_cal: usize = 0;
    let mut elves: Vec<usize> = Vec::new();
    for (num, line) in reader.lines().enumerate() {
        match line {
            Ok(line) => {
                if line.is_empty() {
                    elves.push(elf_cal);
                    elf_cal = 0;
                } else {
                    let cal: usize = match line.parse() {
                        Ok(cal) => cal,
                        Err(e) => {
                            println!("Error reading input on line {}", num);
                            println!("{}", e);
                            return;
                        }
                    };
                    elf_cal += cal;
                }
            }
            Err(e) => {
                println!("Error reading input on line {}", num);
                println!("{}", e);
            }
        }
    }

    elves.sort();
    let mut sum_top3_cals: usize = 0;
    for cal in elves.iter().rev().take(3) {
        sum_top3_cals += cal;
    }

    println!("Top 1 elf calories = {}", elves.last().unwrap());
    println!("Sum top 3 elf calories = {}", sum_top3_cals);
}
