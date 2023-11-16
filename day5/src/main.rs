mod mod_day5;

use std::io::prelude::*;

fn main() {
    let mut file_input = open_file().unwrap();
    let mut buffer = String::new();
    file_input.read_to_string(&mut buffer).unwrap();

    let mut crate_columns = mod_day5::parse_crate_all_columns(&buffer).unwrap();
    let max_vlen = crate_columns.iter().map(|v| v.len()).max().unwrap();
    let mut moves = mod_day5::parse_move_all_lines(&buffer, max_vlen + 2).unwrap();
    mod_day5::exec_moves_part1(&mut crate_columns, &mut moves);
    let top_c = mod_day5::top_of_crate_columns(crate_columns);
    println!("Part 1 top crates = {}", &top_c);

    let mut crate_columns = mod_day5::parse_crate_all_columns(&buffer).unwrap();
    let max_vlen = crate_columns.iter().map(|v| v.len()).max().unwrap();
    let mut moves = mod_day5::parse_move_all_lines(&buffer, max_vlen + 2).unwrap();
    mod_day5::exec_moves_part2(&mut crate_columns, &mut moves);
    let top_c = mod_day5::top_of_crate_columns(crate_columns);
    println!("Part 2 top crates = {}", &top_c);
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day5/src/input.txt";
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
