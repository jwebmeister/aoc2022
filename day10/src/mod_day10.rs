use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, map},
    sequence::preceded,
    Finish, IResult,
};
use std::{collections::VecDeque, io::prelude::*};
use thiserror::Error;

const CRT_WIDTH: usize = 40;
const CRT_HEIGHT: usize = 6;

pub struct Crt([i32; CRT_WIDTH * CRT_HEIGHT]);

impl std::fmt::Debug for Crt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        for row in 0..CRT_HEIGHT {
            let idx_start = row * CRT_WIDTH;
            let idx_end = (row + 1) * CRT_WIDTH;
            let line = self.0[idx_start..idx_end]
                .iter()
                .map(|n| if *n >= 1 { "#" } else { "." })
                .collect::<String>();
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub enum Command {
    Noop,
    Addx(i32),
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Command::Noop => write!(f, "noop"),
            Command::Addx(n) => write!(f, "addx {}", n),
        }
    }
}

#[derive(Error, Debug)]
pub enum MyError {
    #[error("parsing error, {0:?}")]
    Parser(nom::error::ErrorKind),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl<T> From<nom::error::Error<T>> for MyError {
    fn from(err: nom::error::Error<T>) -> Self {
        Self::Parser(err.code)
    }
}

pub fn lines_to_result<R: std::io::BufRead>(
    reader: &mut R,
) -> Result<(Vec<i32>, Crt), MyError> {
    let mut v: Vec<i32> = Vec::new();
    let mut crt = [0; CRT_WIDTH * CRT_HEIGHT];
    let mut cycle: i32 = 0;
    let mut x: i32 = 1;
    for line in reader.lines() {
        let l = line?;
        match all_consuming(parse_command)(&l).finish() {
            Ok((_, cmd)) => match cmd {
                Command::Noop => {
                    cycle += 1;
                    if (cycle - 20) % 40 == 0 {
                        v.push(cycle * x);
                    };
                    if ((x - 1)..=(x + 1)).contains(&(cycle % 40)) && (cycle as usize) < crt.len() {
                        crt[(cycle - 1) as usize] = 1;
                    };
                }
                Command::Addx(n) => {
                    cycle += 1;
                    if (cycle - 20) % 40 == 0 {
                        v.push(cycle * x);
                    };
                    if ((x - 1)..=(x + 1)).contains(&(cycle % 40)) && (cycle as usize) < crt.len() {
                        crt[(cycle - 1) as usize] = 1;
                    };
                    cycle += 1;
                    if (cycle - 20) % 40 == 0 {
                        v.push(cycle * x);
                    };
                    if ((x - 1)..=(x + 1)).contains(&(cycle % 40)) && (cycle as usize) < crt.len() {
                        crt[(cycle - 1) as usize] = 1;
                    };
                    x += n;
                }
            },
            Err(e) => return Err(e.into()),
        };
    }

    Ok((v, Crt(crt)))
}

pub fn parse_lines_to_commands<R: std::io::BufRead>(
    reader: &mut R,
) -> Result<VecDeque<Command>, MyError> {
    let mut v: VecDeque<Command> = VecDeque::new();
    for line in reader.lines() {
        let l = line?;
        match all_consuming(parse_command)(&l).finish() {
            Ok(cmd) => v.push_back(cmd.1),
            Err(e) => return Err(e.into()),
        };
    }

    Ok(v)
}

fn parse_command(i: &str) -> IResult<&str, Command> {
    alt((parse_noop, parse_addx))(i)
}

fn parse_addx(i: &str) -> IResult<&str, Command> {
    map(preceded(tag("addx "), nom::character::complete::i32), |x| {
        Command::Addx(x)
    })(i)
}

fn parse_noop(i: &str) -> IResult<&str, Command> {
    map(tag("noop"), |_| Command::Noop)(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_to_result_works() {
        let s = "addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop";

        let img = 
"##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######.....";

        let mut reader = std::io::BufReader::new(s.as_bytes());

        let (v, crt) = lines_to_result(&mut reader).unwrap();

        assert_eq!(13140, v.iter().sum::<i32>());

        assert_eq!(img, format!("{:?}", crt));
    }
}
