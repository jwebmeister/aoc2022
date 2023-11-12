use itertools::Itertools;
use std::io::prelude::*;

type BoxedError = Box<dyn std::error::Error>;
type SectionAssignments = (usize, usize, usize, usize);

fn main() {
    let file_input = open_file().ok().unwrap();
    let mut reader = std::io::BufReader::new(file_input);
    let _parsed = parse_input(&mut reader);
}

fn parse_input<R: std::io::BufRead>(reader: &mut R) -> Result<Vec<SectionAssignments>, BoxedError> {
    reader
        .lines()
        .map(|line| -> Result<(usize, usize, usize, usize), BoxedError> {
            let l = line?;
            let v = l
                .split(',')
                .map(|s| {
                    s.split('-')
                        .map(|n| n.parse::<usize>())
                        .collect::<Result<Vec<_>, _>>()
                })
                .flatten_ok()
                .collect::<Result<Vec<_>, _>>()?;
            let t = (v[0], v[1], v[2], v[3]);
            Ok(t)
        })
        .collect::<Result<Vec<_>, _>>()
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day4/src/input.txt";
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
    fn parse_input_works() {
        let s = "2-4,6-8\n2-3,4-5\n5-7,7-9\n2-8,3-7\n6-6,4-6\n2-6,4-8";
        let mut reader = std::io::BufReader::new(s.as_bytes());
        let parsed = parse_input(&mut reader).unwrap();
        let v: Vec<SectionAssignments> = vec![
            (2, 4, 6, 8),
            (2, 3, 4, 5),
            (5, 7, 7, 9),
            (2, 8, 3, 7),
            (6, 6, 4, 6),
            (2, 6, 4, 8),
        ];
        assert_eq!(parsed, v);
    }
}
