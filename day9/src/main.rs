use std::collections::HashSet;
use std::io::prelude::*;
use thiserror::Error;

const NUM_ROPE_KNOTS: usize = 10;

fn main() {
    let file = open_file().unwrap();
    let mut reader = std::io::BufReader::new(file);
    let mut ml = parse_moves_list(&mut reader).unwrap();
    let hs = exec_moves_list(&mut ml);
    println!("Unique locations visited by tail = {0}", hs.len());
}

#[derive(Debug, Clone, PartialEq)]
struct Move {
    dir: Direction,
    qty: usize,
}

impl Move {
    fn dir_coords(&self) -> (isize, isize) {
        match self.dir {
            Direction::Up => (1, 0),
            Direction::Down => (-1, 0),
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
struct RopeKnot {
    row: isize,
    col: isize,
}

impl RopeKnot {
    fn get_coords(&self) -> (isize, isize) {
        (self.row, self.col)
    }
}

#[derive(Error, Debug)]
enum MyError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Couldn't parse direction of steps")]
    ParseDir,
    #[error("Couldn't parse quantity of steps")]
    ParseQty,
}

fn exec_moves_list(ml: &mut [Move]) -> HashSet<(isize, isize)> {
    let mut hs = HashSet::new();
    let mut knots = [RopeKnot { row: 0, col: 0 }; NUM_ROPE_KNOTS];
    hs.insert(knots[knots.len() - 1].get_coords());

    let result = ml
        .iter_mut()
        .map(|m| exec_move(m, &mut knots))
        .fold(hs, |mut acc, h| {
            acc.extend(&h);
            acc
        });

    result
}

fn exec_move(m: &mut Move, knots: &mut [RopeKnot]) -> HashSet<(isize, isize)> {
    let mut hs = HashSet::with_capacity((m.qty / (knots.len() - 1)) + 1);
    while m.qty >= 1 {
        let dir_coord = m.dir_coords();
        let _ = step_head(&mut knots[0], dir_coord);
        let num_middle_knots = (knots.len() as isize - 2) as usize;
        if num_middle_knots >= 1 {
            for i in 1..(num_middle_knots + 1) {
                let head_coord = knots[i - 1].get_coords();
                let _ = step_tail(&mut knots[i], head_coord);
            }
        }
        let head_coord = knots[knots.len() - 2].get_coords();
        let tail_coord = step_tail(&mut knots[knots.len() - 1], head_coord);
        hs.insert(tail_coord);
        m.qty -= 1;
    }
    hs
}

fn step_head(head: &mut RopeKnot, dir_coord: (isize, isize)) -> (isize, isize) {
    head.row += dir_coord.0;
    head.col += dir_coord.1;
    (head.row, head.col)
}

fn step_tail(tail: &mut RopeKnot, head_coord: (isize, isize)) -> (isize, isize) {
    let row_diff = head_coord.0 - tail.row;
    let col_diff = head_coord.1 - tail.col;

    if (-1..=1).contains(&row_diff) && (-1..=1).contains(&col_diff) {
        return (tail.row, tail.col);
    };

    let row_diff_dir = (head_coord.0 - tail.row).clamp(-1, 1);
    let col_diff_dir = (head_coord.1 - tail.col).clamp(-1, 1);

    tail.row += row_diff_dir;
    tail.col += col_diff_dir;

    (tail.row, tail.col)
}

fn parse_moves_list<R: std::io::BufRead>(reader: &mut R) -> Result<Vec<Move>, MyError> {
    let mut v = Vec::new();
    for line in reader.lines() {
        let l = line?;
        v.push(parse_move(l)?);
    }
    Ok(v)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_move_works() {
        let s = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";
        let v = s
            .lines()
            .map(|l| parse_move(l))
            .collect::<Result<Vec<_>, MyError>>()
            .unwrap();
        let r = vec![
            Move {
                dir: Direction::Right,
                qty: 4,
            },
            Move {
                dir: Direction::Up,
                qty: 4,
            },
            Move {
                dir: Direction::Left,
                qty: 3,
            },
            Move {
                dir: Direction::Down,
                qty: 1,
            },
            Move {
                dir: Direction::Right,
                qty: 4,
            },
            Move {
                dir: Direction::Down,
                qty: 1,
            },
            Move {
                dir: Direction::Left,
                qty: 5,
            },
            Move {
                dir: Direction::Right,
                qty: 2,
            },
        ];

        assert_eq!(r, v);
    }

    #[test]
    fn parse_moves_list_works() {
        let s = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";
        let mut reader = std::io::BufReader::new(s.as_bytes());
        let v = parse_moves_list(&mut reader).unwrap();
        let r = vec![
            Move {
                dir: Direction::Right,
                qty: 4,
            },
            Move {
                dir: Direction::Up,
                qty: 4,
            },
            Move {
                dir: Direction::Left,
                qty: 3,
            },
            Move {
                dir: Direction::Down,
                qty: 1,
            },
            Move {
                dir: Direction::Right,
                qty: 4,
            },
            Move {
                dir: Direction::Down,
                qty: 1,
            },
            Move {
                dir: Direction::Left,
                qty: 5,
            },
            Move {
                dir: Direction::Right,
                qty: 2,
            },
        ];

        assert_eq!(r, v);
    }

    #[test]
    fn exec_moves_list_works() {
        let mut ml = vec![
            Move {
                dir: Direction::Right,
                qty: 4,
            },
            Move {
                dir: Direction::Up,
                qty: 4,
            },
            Move {
                dir: Direction::Left,
                qty: 3,
            },
            Move {
                dir: Direction::Down,
                qty: 1,
            },
            Move {
                dir: Direction::Right,
                qty: 4,
            },
            Move {
                dir: Direction::Down,
                qty: 1,
            },
            Move {
                dir: Direction::Left,
                qty: 5,
            },
            Move {
                dir: Direction::Right,
                qty: 2,
            },
        ];
        let hs = exec_moves_list(&mut ml);

        // when NUM_ROPE_KNOTS = 2
        // assert_eq!(13, hs.len());

        // when NUM_ROPE_KNOTS = 10
        assert_eq!(1, hs.len());
    }
}
