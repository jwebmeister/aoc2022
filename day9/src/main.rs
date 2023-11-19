use std::collections::HashSet;
use thiserror::Error;

fn main() {
    println!("Hello, world!");
}

struct Move {
    dir: Direction,
    qty: usize,
}

impl Move {
    fn to_coords(&self) -> (isize, isize) {
        let qty = self.qty as isize;
        match self.dir {
            Direction::Up => (qty, 0),
            Direction::Down => (-qty, 0),
            Direction::Left => (0, -qty),
            Direction::Right => (0, qty),
        }
    }

    fn dir_coords(&self) -> (isize, isize) {
        match self.dir {
            Direction::Up => (1, 0),
            Direction::Down => (-1, 0),
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
        }
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Error, Debug)]
enum MyError {
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Couldn't parse direction of steps")]
    ParseDir,
    #[error("Couldn't parse quantity of steps")]
    ParseQty,
}

fn tail_visits<R: std::io::BufRead>(reader: &mut R) -> HashSet<(isize, isize)> {
    todo!()
}

fn parse_move<S: Into<String>>(s: S) -> Result<Move, MyError> {
    let string1: String = s.into();
    let trim1 = string1.trim();
    let mut iter = trim1.split_ascii_whitespace();

    let maybe_dir = iter.next().ok_or(MyError::ParseDir)?;
    let maybe_qty = iter.next().ok_or(MyError::ParseQty)?;

    let dir = match maybe_dir {
        "U" => Direction::Up,
        "D" => Direction::Down,
        "L" => Direction::Left,
        "R" => Direction::Right,
        _ => return Err(MyError::ParseDir),
    };
    let qty = maybe_qty.parse::<usize>()?;
    Ok(Move { dir, qty })
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day9/src/input.txt";
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
