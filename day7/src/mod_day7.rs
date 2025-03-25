use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::{all_consuming, map},
    sequence::{preceded, separated_pair},
    Finish, IResult, Parser,
};
use std::io::prelude::*;
use thiserror::Error;
use typed_path::{Utf8PathBuf, Utf8UnixEncoding};

#[derive(Debug, Clone, PartialEq)]
pub struct FsEntry {
    path: Utf8PathBuf<Utf8UnixEncoding>,
    size: u64,
    fullpath: Utf8PathBuf<Utf8UnixEncoding>,
}

trait ArenaFsEntry {
    fn nodeid_from_fullpath(
        &self,
        fullpath: &Utf8PathBuf<Utf8UnixEncoding>,
    ) -> Option<indextree::NodeId>;
}

impl ArenaFsEntry for indextree::Arena<FsEntry> {
    fn nodeid_from_fullpath(
        &self,
        fullpath: &Utf8PathBuf<Utf8UnixEncoding>,
    ) -> Option<indextree::NodeId> {
        for node in self.iter() {
            if node.get().fullpath == *fullpath {
                return self.get_node_id(node);
            }
        }
        None
    }
}

#[derive(Error, Debug)]
pub enum MyError {
    #[error("parsing error, {0:?}")]
    Parser(nom::error::ErrorKind),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("IndexTree error, {0:?}")]
    IndexTree(String),
}

impl<T> From<nom::error::Error<T>> for MyError {
    fn from(err: nom::error::Error<T>) -> Self {
        // Get details from the error you want,
        // or even implement for both T variants.
        Self::Parser(err.code)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Command(Command),
    Entry(Entry),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Ls,
    Cd(Utf8PathBuf<Utf8UnixEncoding>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Dir(Utf8PathBuf<Utf8UnixEncoding>),
    File(u64, Utf8PathBuf<Utf8UnixEncoding>),
}

pub fn part2(v: &[(Utf8PathBuf<Utf8UnixEncoding>, u64)]) -> Option<u64> {
    const TOTAL_AVAIL: u64 = 70000000;
    const NEED_FREE: u64 = 30000000;

    let Some(root_size) = v.iter().map(|d| d.1).max() else {
        return None;
    };

    let goal_free = std::cmp::max(NEED_FREE - (TOTAL_AVAIL - root_size), 0);

    v.iter()
        .filter_map(|d| if d.1 >= goal_free { Some(d.1) } else { None })
        .min()
}

pub fn sum_dir_sizes_part1(v: &[(Utf8PathBuf<Utf8UnixEncoding>, u64)]) -> u64 {
    v.iter()
        .filter_map(|ds| {
            let size = ds.1;
            match size <= 100000 {
                true => Some(size),
                false => None,
            }
        })
        .sum::<u64>()
}

pub fn dir_sizes(
    arena: &indextree::Arena<FsEntry>,
) -> Result<Vec<(Utf8PathBuf<Utf8UnixEncoding>, u64)>, MyError> {
    let mut v: Vec<(Utf8PathBuf<Utf8UnixEncoding>, u64)> = vec![];
    for node in arena.iter() {
        let fs = node.get();
        if fs.size == 0 {
            let Some(node_id) = arena.get_node_id(node) else {
                return Err(MyError::IndexTree("Unable to get current node".into()));
            };
            let sum = node_id
                .descendants(arena)
                .filter_map(|d_id| arena.get(d_id))
                .map(|d_node: &indextree::Node<FsEntry>| d_node.get().size)
                .sum::<u64>();
            v.push((fs.fullpath.clone(), sum));
        }
    }
    Ok(v)
}

pub fn all_lines_into_tree(lines: &[Line]) -> Result<indextree::Arena<FsEntry>, MyError> {
    let mut arena = indextree::Arena::<FsEntry>::new();
    let root_node = arena.new_node(FsEntry {
        path: "/".into(),
        size: 0,
        fullpath: Utf8PathBuf::<Utf8UnixEncoding>::from("/"),
    });
    let mut current_node = root_node;

    for line in lines {
        match line {
            Line::Command(cmd) => match cmd {
                Command::Ls => {}
                Command::Cd(path) => match path.as_str() {
                    "/" => {}
                    ".." => {
                        let Some(c_node) = arena.get(current_node) else {
                            return Err(MyError::IndexTree("Unable to get current node".into()));
                        };
                        match c_node.parent() {
                            Some(p) => current_node = p,
                            None => {
                                return Err(MyError::IndexTree(
                                    "Unable to get current nodes parent".into(),
                                ))
                            }
                        };
                    }
                    _ => {
                        let Some(c_node) = arena.get(current_node) else {
                            return Err(MyError::IndexTree("Unable to get current node".into()));
                        };
                        let new_fullpath = c_node.get().fullpath.join(path);
                        match arena.nodeid_from_fullpath(&new_fullpath) {
                            Some(existing_node) => {
                                current_node = existing_node;
                            }
                            None => {
                                let node = current_node.append_value(
                                    FsEntry {
                                        path: path.clone(),
                                        size: 0,
                                        fullpath: new_fullpath,
                                    },
                                    &mut arena,
                                );
                                current_node = node;
                            }
                        };
                    }
                },
            },
            Line::Entry(entry) => match entry {
                Entry::Dir(path) => {
                    let Some(c_node) = arena.get(current_node) else {
                        return Err(MyError::IndexTree("Unable to get current node".into()));
                    };
                    let new_fullpath = c_node.get().fullpath.join(path);
                    if arena.nodeid_from_fullpath(&new_fullpath).is_some() {
                        continue;
                    };
                    let _node = current_node.append_value(
                        FsEntry {
                            path: path.clone(),
                            size: 0,
                            fullpath: new_fullpath,
                        },
                        &mut arena,
                    );
                }
                Entry::File(size, path) => {
                    let Some(c_node) = arena.get(current_node) else {
                        return Err(MyError::IndexTree("Unable to get current node".into()));
                    };
                    let new_fullpath = c_node.get().fullpath.join(path);
                    if arena.nodeid_from_fullpath(&new_fullpath).is_some() {
                        continue;
                    };
                    let _node = current_node.append_value(
                        FsEntry {
                            path: path.clone(),
                            size: *size,
                            fullpath: new_fullpath,
                        },
                        &mut arena,
                    );
                }
            },
        }
    }

    Ok(arena)
}

pub fn parse_all_lines<R: std::io::BufRead>(reader: &mut R) -> Result<Vec<Line>, MyError> {
    reader
        .lines()
        .map(|r_line| match r_line {
            Ok(line) => match all_consuming(parse_line).parse(&line).finish() {
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
    )).parse(i)
}

fn parse_entry(i: &str) -> IResult<&str, Entry> {
    alt((parse_dir, parse_file)).parse(i)
}

fn parse_file(i: &str) -> IResult<&str, Entry> {
    map(
        separated_pair(nom::character::complete::u64, tag(" "), parse_path),
        |(size, path)| Entry::File(size, path),
    ).parse(i)
}

fn parse_dir(i: &str) -> IResult<&str, Entry> {
    map(preceded(tag("dir "), parse_path), Entry::Dir).parse(i)
}

fn parse_command(i: &str) -> IResult<&str, Command> {
    let (i, _) = tag("$ ")(i)?;
    alt((parse_ls, parse_cd)).parse(i)
}

fn parse_ls(i: &str) -> IResult<&str, Command> {
    map(tag("ls"), |_| Command::Ls).parse(i)
}

fn parse_cd(i: &str) -> IResult<&str, Command> {
    map(preceded(tag("cd "), parse_path), Command::Cd).parse(i)
}

fn parse_path(i: &str) -> IResult<&str, Utf8PathBuf<Utf8UnixEncoding>> {
    map(
        take_while1(|c: char| "abcdefghijklmnopqrstuvwxyz./".contains(c)),
        Into::into,
    ).parse(i)
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

    #[test]
    fn all_lines_into_tree_works() {
        let lines = vec![
            Line::Command(Command::Cd("/".into())),
            Line::Command(Command::Ls),
            Line::Entry(Entry::Dir("a".into())),
            Line::Entry(Entry::File(14848514, "r1.txt".into())),
            Line::Command(Command::Cd("a".into())),
            Line::Command(Command::Ls),
            Line::Entry(Entry::File(100, "a1.txt".into())),
            Line::Command(Command::Cd("..".into())),
        ];

        let result = all_lines_into_tree(&lines);
        assert!(result.is_ok());
        dbg!(result.unwrap());
    }

    #[test]
    fn dir_sizes_works() {
        let lines = vec![
            Line::Command(Command::Cd("/".into())),
            Line::Command(Command::Ls),
            Line::Entry(Entry::Dir("a".into())),
            Line::Entry(Entry::File(14848514, "r1.txt".into())),
            Line::Command(Command::Cd("a".into())),
            Line::Command(Command::Ls),
            Line::Entry(Entry::File(100, "a1.txt".into())),
        ];

        let r = vec![
            (Utf8PathBuf::<Utf8UnixEncoding>::from("/"), 14848614),
            (Utf8PathBuf::<Utf8UnixEncoding>::from("/a"), 100),
        ];

        let arena = all_lines_into_tree(&lines).unwrap();
        let dir_sizes = dir_sizes(&arena).unwrap();

        assert_eq!(r, dir_sizes);
    }

    #[test]
    fn sum_dir_sizes_part1_works() {
        let s = "\
$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k\
";
        let mut reader = std::io::BufReader::new(s.as_bytes());
        let lines = parse_all_lines(&mut reader).unwrap();
        let tree = all_lines_into_tree(&lines).unwrap();
        let dir_sizes = dir_sizes(&tree).unwrap();
        let sum_dir = sum_dir_sizes_part1(&dir_sizes);
        assert_eq!(95437, sum_dir);
    }

    #[test]
    fn part2_works() {
        let s = "\
$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k\
";
        let mut reader = std::io::BufReader::new(s.as_bytes());
        let lines = parse_all_lines(&mut reader).unwrap();
        let tree = all_lines_into_tree(&lines).unwrap();
        let dir_sizes = dir_sizes(&tree).unwrap();
        let sum_part2 = part2(&dir_sizes).unwrap();
        assert_eq!(24933642, sum_part2);
    }
}
