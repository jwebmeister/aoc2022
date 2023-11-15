extern crate nom;

use nom::Finish;
use std::io::prelude::*;
use thiserror::Error;

fn main() {
    let mut file_input = open_file().unwrap();
    let mut buffer = String::new();
    file_input.read_to_string(&mut buffer).unwrap();

    let crate_lines = parse_crate_all_lines(&buffer);
    let crate_columns = transpose_rev(crate_lines).unwrap();

    for col in &crate_columns {
        println!("{col:?}");
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
struct Crate(char);

impl std::fmt::Debug for Crate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

#[derive(Error, Debug)]
enum MyError {
    #[error("transpose_rev error, {0}")]
    TransposeRev(String),
}

fn transpose_rev<T>(v: Vec<Vec<Option<T>>>) -> Result<Vec<Vec<T>>, MyError> {
    if v.is_empty() {
        return Err(MyError::TransposeRev("input Vec `v` is empty".to_string()));
    };

    let len = v[0].len();
    if len <= 0 {
        return Err(MyError::TransposeRev("interior Vec is empty".to_string()));
    };

    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .rev()
                .map(|n| match n.next() {
                    Some(x) => Ok(x),
                    None => Err(MyError::TransposeRev(
                        "interior Vec's have mismatching dimensions".to_string(),
                    )),
                })
                .filter(|n| match n {
                    Ok(x) => x.is_some(),
                    Err(_) => true,
                })
                .map(|n| match n {
                    Ok(x) => x.ok_or_else(|| {
                        MyError::TransposeRev(
                            "did not filter `None` elements correctly".to_string(),
                        )
                    }),
                    Err(e) => Err(e),
                })
                .collect::<Result<Vec<T>, MyError>>()
        })
        .collect()
}

fn parse_crate_all_lines(buffer: &str) -> Vec<Vec<Option<Crate>>> {
    let mut crate_lines = vec![];
    for line in buffer.lines() {
        if let Ok((_rest, crate_line)) =
            nom::combinator::all_consuming(parse_crate_line)(line).finish()
        {
            crate_lines.push(crate_line);
        }
    }
    crate_lines
}

fn parse_crate_line(i: &str) -> nom::IResult<&str, Vec<Option<Crate>>> {
    let (mut i, c) = parse_crate_or_hole(i)?;
    let mut v = vec![c];

    loop {
        let (next_i, maybe_c) = nom::combinator::opt(nom::sequence::preceded(
            nom::bytes::complete::tag(" "),
            parse_crate_or_hole,
        ))(i)?;
        match maybe_c {
            Some(c) => v.push(c),
            None => break,
        }
        i = next_i;
    }

    Ok((i, v))
}

fn parse_crate_or_hole(i: &str) -> nom::IResult<&str, Option<Crate>> {
    nom::branch::alt((
        nom::combinator::map(parse_crate, Some),
        nom::combinator::map(parse_hole, |_| None),
    ))(i)
}

fn parse_crate(i: &str) -> nom::IResult<&str, Crate> {
    let first_char = |s: &str| -> Result<Crate, nom::error::Error<&str>> {
        let c = s.chars().next().ok_or_else(|| nom::error::Error {
            input: "",
            code: nom::error::ErrorKind::MapRes,
        })?;
        Ok(Crate(c))
    };
    let f = nom::sequence::delimited(
        nom::bytes::complete::tag("["),
        nom::bytes::complete::take(1_usize),
        nom::bytes::complete::tag("]"),
    );
    nom::combinator::map_res(f, first_char)(i)
}

fn parse_hole(i: &str) -> nom::IResult<&str, ()> {
    nom::combinator::map(nom::bytes::complete::tag("   "), drop)(i)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_crate_works() {
        let s = "[D]";
        let result = parse_crate(s);
        let crate_char = result.unwrap().1 .0;
        assert_eq!('D', crate_char);
    }

    #[test]
    fn parse_hole_works() {
        let s = "   ";
        let result = parse_hole(s);
        let hole = result.unwrap().1;
        assert_eq!((), hole);
    }

    #[test]
    fn parse_crate_or_hole_works() {
        let s = "[D]        \n";
        let result1 = parse_crate_or_hole(s).unwrap();
        let crate1 = result1.1.unwrap();
        assert_eq!('D', crate1.0);

        let result2 = parse_crate_or_hole(result1.0).unwrap();
        let crate2 = result2.1;
        assert!(crate2.is_none());
    }

    #[test]
    fn parse_crate_line_works() {
        let s = "\
            [D]        \n\
            [N] [C]    \n\
            [Z] [M] [P]\n\
             1   2   3 \n\
            \n\
            move 1 from 2 to 1\n\
            move 3 from 1 to 3\n\
            move 2 from 2 to 1\n\
            move 1 from 1 to 2\n\
            ";
        let result = parse_crate_line(s).unwrap();
        assert_eq!(vec![Some(Crate('D'),), None, None,], result.1)
    }

    #[test]
    fn parse_crate_all_lines_works() {
        let s = "\
            [D]        \n\
            [N] [C]    \n\
            [Z] [M] [P]\n\
             1   2   3 \n\
            \n\
            move 1 from 2 to 1\n\
            move 3 from 1 to 3\n\
            move 2 from 2 to 1\n\
            move 1 from 1 to 2\n\
            ";

        let r = vec![
            vec![Some(Crate('D')), None, None],
            vec![Some(Crate('N')), Some(Crate('C')), None],
            vec![Some(Crate('Z')), Some(Crate('M')), Some(Crate('P'))],
        ];

        let crate_lines = parse_crate_all_lines(s);

        assert_eq!(crate_lines, r);
    }

    #[test]
    fn transpose_rev_works() {
        let v = vec![
            vec![Some(Crate('D')), None, None],
            vec![Some(Crate('N')), Some(Crate('C')), None],
            vec![Some(Crate('Z')), Some(Crate('M')), Some(Crate('P'))],
        ];

        let r = vec![
            vec![Crate('Z'), Crate('N'), Crate('D')],
            vec![Crate('M'), Crate('C')],
            vec![Crate('P')],
        ];

        let crate_cols = transpose_rev(v).unwrap();

        assert_eq!(r, crate_cols);
    }

    #[test]
    fn transpose_rev_errorhandling_works() {
        let misdim_vec = vec![
            vec![Some(Crate('D')), None, None],
            vec![Some(Crate('N'))],
            vec![Some(Crate('Z')), Some(Crate('M')), Some(Crate('P'))],
        ];
        let result_misdim = transpose_rev(misdim_vec);

        assert!(matches!(result_misdim, Err(MyError::TransposeRev(_))));

        let empty_vec: Vec<Vec<Option<Crate>>> = vec![];
        let result_empty = transpose_rev(empty_vec);

        assert!(matches!(result_empty, Err(MyError::TransposeRev(_))));
    }
}
