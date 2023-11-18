mod mod_day8;

fn main() {
    let file = open_file().unwrap();
    let part1 = mod_day8::visible_any_side(file).unwrap();
    println!("Trees visible from any side = {0}", part1.len());

    let file = open_file().unwrap();
    let mut reader = std::io::BufReader::new(file);
    let matrix = mod_day8::read_into_matrix(&mut reader).unwrap();
    let part2 = mod_day8::highest_score(matrix);
    println!("Highest score = {0}", part2);
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day8/src/input.txt";
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
