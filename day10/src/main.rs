mod mod_day10;

fn main() {
    let file = open_file().unwrap();
    let mut reader = std::io::BufReader::new(file);
    let (v, crt) = mod_day10::lines_to_result(&mut reader).unwrap();
    let part1 = v.iter().sum::<i32>();
    println!("Part 1 = {}", part1);
    println!("{:?}", crt);
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day10/src/input.txt";
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
