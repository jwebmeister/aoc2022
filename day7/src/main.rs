use crate::mod_day7::sum_dir_sizes_part1;

mod mod_day7;

fn main() {
    let file = open_file().unwrap();
    let mut reader = std::io::BufReader::new(file);
    let lines = mod_day7::parse_all_lines(&mut reader).unwrap();
    let tree = mod_day7::all_lines_into_tree(&lines).unwrap();
    let dir_sizes = mod_day7::dir_sizes(&tree).unwrap();

    let sum_dir = sum_dir_sizes_part1(dir_sizes);
    println!("Sum dir sizes (for dir <= 100000) = {0}", sum_dir);
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day7/src/input.txt";
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
