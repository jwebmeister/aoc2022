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
        "Shortest path from Start to reach highest elevation E = {} steps",
        path.len() - 1
    );

    let mut bfs_down = mod_day12::Bfs::new();
    bfs_down.step_down(&grid);
    while !bfs_down.current.is_empty()
        && !(bfs_down
            .current
            .iter()
            .map(|coord| grid.get_cell_from_coord(*coord).unwrap().elevation())
            .any(|x| x == 0))
    {
        bfs_down.step_down(&grid);
        if bfs_down.num_steps >= 100_000_000 {
            panic!("Too many steps")
        };
    }
    let end_coords = bfs_down
        .current
        .iter()
        .filter(|coord| grid.get_cell_from_coord(**coord).unwrap().elevation() == 0)
        .collect::<Vec<_>>();
    let down_path1 = bfs_down.trace_back_path(*end_coords[0]).unwrap();
    println!(
        "Down: Shortest path from elevation 0 to reach highest elevation E = {} steps",
        down_path1.len() - 1
    );

    let mut bfs_up = mod_day12::Bfs::new();
    bfs_up.step_up(&grid);
    while !bfs_up.current.contains(&grid.get_end_coord().unwrap()) {
        bfs_up.step(&grid);
    }
    let mut up_path1 = bfs_up
        .trace_back_path(grid.get_end_coord().unwrap())
        .unwrap();
    up_path1.reverse();
    println!(
        "Up: Shortest path from elevation 0 to reach highest elevation E = {} steps",
        up_path1.len() - 1
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
