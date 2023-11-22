mod mod_day12;

fn main() {
    let file = open_file().unwrap();
    let reader = std::io::BufReader::new(file);
    let grid = mod_day12::parse_into_grid(reader).unwrap();
    let mut bfs = mod_day12::Bfs::new();
    bfs.step(&grid);
    while !bfs.current.contains(&grid.get_end_coord().unwrap()) {
        bfs.step(&grid);
    }
    let mut path = bfs.trace_back_path(grid.get_end_coord().unwrap()).unwrap();
    path.reverse();
    println!(
        "Shortest path to reach highest elevation = {} steps",
        path.len() - 1
    );
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day12/src/input.txt";
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
