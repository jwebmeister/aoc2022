extern crate nom;

use nom::Finish;
use thiserror::Error;

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Crate(char);

impl std::fmt::Debug for Crate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct MoveQtyFromTo(pub u8, pub u8, pub u8);

#[derive(Error, Debug)]
pub enum MyError {
    #[error("transpose_rev error, {0}")]
    TransposeRevErr(String),
    #[error("parsing error, {0:?}")]
    ParseErr(nom::error::ErrorKind),
}

impl<T> From<nom::error::Error<T>> for MyError {
    fn from(err: nom::error::Error<T>) -> Self {
        // Get details from the error you want,
        // or even implement for both T variants.
        Self::ParseErr(err.code)
    }
}

pub fn exec_moves_part2(crate_columns: &mut [Vec<Crate>], moves: &mut Vec<MoveQtyFromTo>) {
    moves.reverse();

    while let Some(m) = moves.pop() {
        let mut temp_stack = vec![];
        (0..m.0).for_each(|_| {
            if let Some(maybe_c) = crate_columns[(m.1 - 1) as usize].pop() {
                temp_stack.push(maybe_c)
            };
        });
        temp_stack.reverse();
        crate_columns[(m.2 - 1) as usize].append(&mut temp_stack);
    }
}

pub fn exec_moves_part1(crate_columns: &mut [Vec<Crate>], moves: &mut Vec<MoveQtyFromTo>) {
    moves.reverse();

    while let Some(m) = moves.pop() {
        (0..m.0).for_each(|_| {
            if let Some(maybe_c) = crate_columns[(m.1 - 1) as usize].pop() {
                crate_columns[(m.2 - 1) as usize].push(maybe_c)
            };
        });
    }
}

pub fn top_of_crate_columns(crate_columns: Vec<Vec<Crate>>) -> String {
    let mut top_c = String::new();
    for v in &crate_columns {
        if let Some(maybe_c) = v.last() {
            top_c.push(maybe_c.0);
        }
    }
    top_c
}

pub fn parse_move_all_lines(
    buffer: &str,
    skip_lines: usize,
) -> Result<Vec<MoveQtyFromTo>, MyError> {
    let mut move_lines = vec![];
    for (idx, line) in buffer.lines().enumerate() {
        if idx < skip_lines {
            continue;
        };
        match nom::combinator::all_consuming(parse_move_line)(line).finish() {
            Ok((_rest, move_line)) => move_lines.push(move_line),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(move_lines)
}

fn parse_move_line(i: &str) -> nom::IResult<&str, MoveQtyFromTo> {
    let f_qty = nom::sequence::delimited(
        nom::bytes::complete::tag("move "),
        nom::character::complete::digit1,
        nom::bytes::complete::tag(" "),
    );
    let f_parse = |s: &str| -> Result<u8, nom::error::Error<&str>> {
        s.parse::<u8>().map_err(move |_| nom::error::Error {
            input: "",
            code: nom::error::ErrorKind::Digit,
        })
    };
    let (i_next, n_qty) = nom::combinator::map_res(f_qty, f_parse)(i)?;

    let f_from = nom::sequence::delimited(
        nom::bytes::complete::tag("from "),
        nom::character::complete::digit1,
        nom::bytes::complete::tag(" "),
    );
    let (i_next, n_from) = nom::combinator::map_res(f_from, f_parse)(i_next)?;

    let f_to = nom::sequence::preceded(
        nom::bytes::complete::tag("to "),
        nom::character::complete::digit1,
    );
    let (i_next, n_to) = nom::combinator::map_res(f_to, f_parse)(i_next)?;

    Ok((i_next, MoveQtyFromTo(n_qty, n_from, n_to)))
}

pub fn parse_crate_all_columns(buffer: &str) -> Result<Vec<Vec<Crate>>, MyError> {
    let crate_lines = parse_crate_all_lines(buffer)?;
    transpose_rev(crate_lines)
}

fn transpose_rev<T>(v: Vec<Vec<Option<T>>>) -> Result<Vec<Vec<T>>, MyError> {
    if v.is_empty() {
        return Err(MyError::TransposeRevErr(
            "input Vec `v` is empty".to_string(),
        ));
    };

    let len = v[0].len();
    if len == 0 {
        return Err(MyError::TransposeRevErr(
            "interior Vec is empty".to_string(),
        ));
    };

    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .rev()
                .map(|n| match n.next() {
                    Some(x) => Ok(x),
                    None => Err(MyError::TransposeRevErr(
                        "interior Vec's have mismatching dimensions".to_string(),
                    )),
                })
                .filter(|n| match n {
                    Ok(x) => x.is_some(),
                    Err(_) => true,
                })
                .map(|n| match n {
                    Ok(x) => x.ok_or_else(|| {
                        MyError::TransposeRevErr(
                            "did not filter `None` elements correctly".to_string(),
                        )
                    }),
                    Err(e) => Err(e),
                })
                .collect::<Result<Vec<T>, MyError>>()
        })
        .collect()
}

fn parse_crate_all_lines(buffer: &str) -> Result<Vec<Vec<Option<Crate>>>, MyError> {
    let mut crate_lines = vec![];
    for line in buffer.lines() {
        match nom::combinator::all_consuming(parse_crate_line)(line).finish() {
            Ok((_rest, crate_line)) => crate_lines.push(crate_line),
            Err(e) => match parse_is_stack_numbers_line(e.input) {
                true => break,
                false => return Err(e.into()),
            },
        }
    }
    Ok(crate_lines)
}

fn parse_is_stack_numbers_line(i: &str) -> bool {
    let result: nom::IResult<&str, &str> = nom::bytes::complete::tag(" 1   2")(i);
    result.is_ok()
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
        let c = s.chars().next().ok_or(nom::error::Error {
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
        let remaining_text = result.unwrap().0;
        assert_eq!("", remaining_text);
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
            [Z] [M] [P]\n \
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
            [Z] [M] [P]\n \
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

        let crate_lines = parse_crate_all_lines(s).unwrap();

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

        assert!(matches!(result_misdim, Err(MyError::TransposeRevErr(_))));

        let empty_vec: Vec<Vec<Option<Crate>>> = vec![];
        let result_empty = transpose_rev(empty_vec);

        assert!(matches!(result_empty, Err(MyError::TransposeRevErr(_))));
    }

    #[test]
    fn parse_move_line_works() {
        let s = "move 2 from 8 to 1\n";

        let r = MoveQtyFromTo(2, 8, 1);

        let move_qty_from_to = parse_move_line(s).unwrap();

        assert_eq!(move_qty_from_to.1, r);
    }

    #[test]
    fn parse_move_all_lines_works() {
        let s = "\
        move 2 from 8 to 1\n\
        move 4 from 9 to 8\n\
        ";

        let r = vec![MoveQtyFromTo(2, 8, 1), MoveQtyFromTo(4, 9, 8)];

        let v_move = parse_move_all_lines(s, 0).unwrap();

        assert_eq!(v_move, r);
    }

    #[test]
    fn exec_moves_part1_works() {
        let s = "\
         \x20   [D]    \n\
            [N] [C]    \n\
            [Z] [M] [P]\n \
            1   2   3 \n\
            \n\
            move 1 from 2 to 1\n\
            move 3 from 1 to 3\n\
            move 2 from 2 to 1\n\
            move 1 from 1 to 2\n\
            ";

        let mut crate_columns = parse_crate_all_columns(s).unwrap();
        let max_vlen = crate_columns.iter().map(|v| v.len()).max().unwrap();
        let mut moves = parse_move_all_lines(s, max_vlen + 2).unwrap();

        exec_moves_part1(&mut crate_columns, &mut moves);

        let top_c = top_of_crate_columns(crate_columns);

        assert_eq!(top_c, "CMZ");
    }

    #[test]
    fn exec_moves_part2_works() {
        let s = "\
         \x20   [D]    \n\
            [N] [C]    \n\
            [Z] [M] [P]\n \
            1   2   3 \n\
            \n\
            move 1 from 2 to 1\n\
            move 3 from 1 to 3\n\
            move 2 from 2 to 1\n\
            move 1 from 1 to 2\n\
            ";

        let mut crate_columns = parse_crate_all_columns(s).unwrap();
        let max_vlen = crate_columns.iter().map(|v| v.len()).max().unwrap();
        let mut moves = parse_move_all_lines(s, max_vlen + 2).unwrap();

        exec_moves_part2(&mut crate_columns, &mut moves);

        let top_c = top_of_crate_columns(crate_columns);

        assert_eq!(top_c, "MCD");
    }
}
