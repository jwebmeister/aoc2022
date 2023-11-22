mod mod_day11;

fn main() {
    let file = open_file().unwrap();
    let reader = std::io::BufReader::new(file);
    let mut ml = mod_day11::parse_all_monkeys(reader).unwrap();
    while ml.round < 20 {
        ml.complete_round().unwrap();
    }
    dbg!(&ml);

    let mut v_num_inspected = ml
        .data
        .into_iter()
        .map(|m| m.num_items_inspected)
        .collect::<Vec<_>>();
    v_num_inspected.sort_by(|a, b| b.cmp(a));
    let monkey_business = v_num_inspected[0..=1]
        .iter()
        .map(|el| *el)
        .reduce(|acc, el| acc * el)
        .unwrap();
    println!("Monkey business = {}", &monkey_business);
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day11/src/input.txt";
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
