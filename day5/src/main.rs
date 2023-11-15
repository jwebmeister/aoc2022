mod mod_day5;

use std::io::prelude::*;

fn main() {
    let mut file_input = open_file().unwrap();
    let mut buffer = String::new();
    file_input.read_to_string(&mut buffer).unwrap();

    let crate_columns = mod_day5::parse_crate_all_columns(&buffer).unwrap();

    for col in &crate_columns {
        println!("{col:?}");
    }
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
