use camino::Utf8PathBuf;
use nom::{
    branch::alt,
    bytes::complete::tag,
    bytes::complete::take_while1,
    combinator::{all_consuming, map},
    sequence::preceded,
    sequence::separated_pair,
    Finish, IResult,
};
use std::io::prelude::*;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Ls,
    Cd(Utf8PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Dir(Utf8PathBuf),
    File(u64, Utf8PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Command(Command),
    Entry(Entry),
}

#[derive(Error, Debug)]
pub enum MyError {
    #[error("parsing error, {0:?}")]
    ParseErr(nom::error::ErrorKind),
    #[error(transparent)]
    IoErr(#[from] std::io::Error),
}

impl<T> From<nom::error::Error<T>> for MyError {
    fn from(err: nom::error::Error<T>) -> Self {
        // Get details from the error you want,
        // or even implement for both T variants.
        Self::ParseErr(err.code)
    }
}

pub fn parse_all_lines<R: std::io::BufRead>(reader: &mut R) -> Result<Vec<Line>, MyError> {
    reader
        .lines()
        .map(|r_line| match r_line {
            Ok(line) => match all_consuming(parse_line)(&line).finish() {
                Ok(o) => Ok(o.1),
                Err(e) => Err(MyError::from(e)),
            },
            Err(e) => Err(MyError::from(e)),
        })
        .collect::<Result<Vec<_>, _>>()
}

fn parse_line(i: &str) -> IResult<&str, Line> {
    alt((
        map(parse_command, Line::Command),
        map(parse_entry, Line::Entry),
    ))(i)
}

fn parse_entry(i: &str) -> IResult<&str, Entry> {
    alt((parse_dir, parse_file))(i)
}

fn parse_file(i: &str) -> IResult<&str, Entry> {
    map(
        separated_pair(nom::character::complete::u64, tag(" "), parse_path),
        |(size, path)| Entry::File(size, path),
    )(i)
}

fn parse_dir(i: &str) -> IResult<&str, Entry> {
    map(preceded(tag("dir "), parse_path), Entry::Dir)(i)
}

fn parse_command(i: &str) -> IResult<&str, Command> {
    let (i, _) = tag("$ ")(i)?;
    alt((parse_ls, parse_cd))(i)
}

fn parse_ls(i: &str) -> IResult<&str, Command> {
    map(tag("ls"), |_| Command::Ls)(i)
}

fn parse_cd(i: &str) -> IResult<&str, Command> {
    map(preceded(tag("cd "), parse_path), Command::Cd)(i)
}

fn parse_path(i: &str) -> IResult<&str, Utf8PathBuf> {
    map(
        take_while1(|c: char| "abcdefghijklmnopqrstuvwxyz./".contains(c)),
        Into::into,
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_command_works() {
        let s = "$ ls\n  ";
        assert_eq!(Command::Ls, parse_command(s).unwrap().1);

        let s = "$ cd a\n  ";
        assert_eq!(Command::Cd("a".into()), parse_command(s).unwrap().1);

        let s = "$ cd abc";
        assert_eq!(Command::Cd("abc".into()), parse_command(s).unwrap().1);

        let s = "$ cd ..   ";
        assert_eq!(Command::Cd("..".into()), parse_command(s).unwrap().1);

        let s = "$ rubbish";
        assert!(matches!(parse_command(s), Err(nom::Err::Error(_))));
    }

    #[test]
    fn parse_entry_works() {
        let s = "dir abc";
        assert_eq!(Entry::Dir("abc".into()), parse_entry(s).unwrap().1);

        let s = "12345 acbd.efg";
        assert_eq!(
            Entry::File(12345, "acbd.efg".into()),
            parse_entry(s).unwrap().1
        );

        let s = "rubbish rubbish";
        assert!(matches!(parse_entry(s), Err(nom::Err::Error(_))));
    }

    #[test]
    fn parse_line_works() {
        let s = "$ ls";
        assert_eq!(Line::Command(Command::Ls), parse_line(s).unwrap().1);

        let s = "$ cd abc";
        assert_eq!(
            Line::Command(Command::Cd("abc".into())),
            parse_line(s).unwrap().1
        );

        let s = "dir abcd";
        assert_eq!(
            Line::Entry(Entry::Dir("abcd".into())),
            parse_line(s).unwrap().1
        );

        let s = "1234 abcd.efg";
        assert_eq!(
            Line::Entry(Entry::File(1234, "abcd.efg".into())),
            parse_line(s).unwrap().1
        );
    }

    #[test]
    fn parse_all_lines_works() {
        let s = "\
            $ cd /\n\
            $ ls\n\
            dir a\n\
            14848514 b.txt\n\
            $ cd ..\n\
            ";
        let r = vec![
            Line::Command(Command::Cd("/".into())),
            Line::Command(Command::Ls),
            Line::Entry(Entry::Dir("a".into())),
            Line::Entry(Entry::File(14848514, "b.txt".into())),
            Line::Command(Command::Cd("..".into())),
        ];
        let mut reader = std::io::BufReader::new(s.as_bytes());
        let result = parse_all_lines(&mut reader);
        assert_eq!(r, result.unwrap());
    }
}
