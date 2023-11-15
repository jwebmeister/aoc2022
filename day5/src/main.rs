mod mod_day5;

use std::io::prelude::*;

fn main() {
    let mut file_input = open_file().unwrap();
    let mut buffer = String::new();
    file_input.read_to_string(&mut buffer).unwrap();

    let mut crate_columns = mod_day5::parse_crate_all_columns(&buffer).unwrap();

    println!("Initial ---- ");
    for col in &crate_columns {
        println!("{col:?}");
    }
    println!("End Initial ---- ");

    let mut moves = mod_day5::parse_move_all_lines(&buffer, crate_columns[0].len() + 2).unwrap();
    moves.reverse();

    while !moves.is_empty() {
        let m = moves.pop().unwrap();
        (0..m.0).for_each(|_| {
            let maybe_c = crate_columns[(m.1 - 1) as usize].pop();
            if maybe_c.is_some() {
                crate_columns[(m.2 - 1) as usize].push(maybe_c.unwrap())
            };
        });
    }

    println!("Final ---- ");
    for col in &crate_columns {
        println!("{col:?}");
    }
    println!("End Final ---- ");
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
