use itertools::Itertools;
use std::io::prelude::*;

type BoxedError = Box<dyn std::error::Error>;
type SectionAssignment = (usize, usize, usize, usize);

fn main() {
    let file_input = open_file().ok().unwrap();
    let mut reader = std::io::BufReader::new(file_input);
    let parsed = parse_input(&mut reader).unwrap();
    let count_contained: usize = count_complete_overlap(&parsed);
    println!("Fully contained pairs = {}", count_contained);
    let count_overlap: usize = count_any_overlap(&parsed);
    println!("Any overlap pairs = {}", count_overlap);
}

fn count_any_overlap(parsed: &Vec<SectionAssignment>) -> usize {
    parsed
        .iter()
        .map(|sa| check_any_overlap(*sa) as usize)
        .sum()
}

fn check_any_overlap(sa: SectionAssignment) -> bool {
    let amin = sa.0;
    let amax = sa.1;
    let bmin = sa.2;
    let bmax = sa.3;

    let amax_lt_bmin = amax < bmin;
    let bmax_lt_amin = bmax < amin;
    let amin_gt_bmax = amin > bmax;
    let bmin_gt_amax = bmin > amax;

    !(amax_lt_bmin | bmax_lt_amin | amin_gt_bmax | bmin_gt_amax)
}

fn count_complete_overlap(parsed: &Vec<SectionAssignment>) -> usize {
    parsed
        .iter()
        .map(|sa| check_complete_overlap(*sa) as usize)
        .sum()
}

fn check_complete_overlap(sa: SectionAssignment) -> bool {
    let amin = sa.0;
    let amax = sa.1;
    let bmin = sa.2;
    let bmax = sa.3;

    let a_in_b = amin >= bmin && amax <= bmax;
    let b_in_a = bmin >= amin && bmax <= amax;

    a_in_b | b_in_a
}

fn parse_input<R: std::io::BufRead>(reader: &mut R) -> Result<Vec<SectionAssignment>, BoxedError> {
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
        let v: Vec<SectionAssignment> = vec![
            (2, 4, 6, 8),
            (2, 3, 4, 5),
            (5, 7, 7, 9),
            (2, 8, 3, 7),
            (6, 6, 4, 6),
            (2, 6, 4, 8),
        ];
        assert_eq!(parsed, v);
    }

    #[test]
    fn check_complete_overlap_works() {
        let v: Vec<SectionAssignment> = vec![
            (2, 4, 6, 8),
            (2, 3, 4, 5),
            (5, 7, 7, 9),
            (2, 8, 3, 7),
            (6, 6, 4, 6),
            (2, 6, 4, 8),
        ];

        let mut overlaps = v.iter().map(|sa| check_complete_overlap(*sa));

        assert_eq!(false, overlaps.next().unwrap());
        assert_eq!(false, overlaps.next().unwrap());
        assert_eq!(false, overlaps.next().unwrap());
        assert_eq!(true, overlaps.next().unwrap());
        assert_eq!(true, overlaps.next().unwrap());
        assert_eq!(false, overlaps.next().unwrap());
    }

    #[test]
    fn count_complete_overlap_works() {
        let v: Vec<SectionAssignment> = vec![
            (2, 4, 6, 8),
            (2, 3, 4, 5),
            (5, 7, 7, 9),
            (2, 8, 3, 7),
            (6, 6, 4, 6),
            (2, 6, 4, 8),
        ];

        let count = count_complete_overlap(&v);

        assert_eq!(2, count);
    }

    #[test]
    fn check_any_overlap_works() {
        let v: Vec<SectionAssignment> = vec![
            (2, 4, 6, 8),
            (2, 3, 4, 5),
            (5, 7, 7, 9),
            (2, 8, 3, 7),
            (6, 6, 4, 6),
            (2, 6, 4, 8),
        ];

        let mut overlaps = v.iter().map(|sa| check_any_overlap(*sa));

        assert_eq!(false, overlaps.next().unwrap());
        assert_eq!(false, overlaps.next().unwrap());
        assert_eq!(true, overlaps.next().unwrap());
        assert_eq!(true, overlaps.next().unwrap());
        assert_eq!(true, overlaps.next().unwrap());
        assert_eq!(true, overlaps.next().unwrap());
    }

    #[test]
    fn count_any_overlap_works() {
        let v: Vec<SectionAssignment> = vec![
            (2, 4, 6, 8),
            (2, 3, 4, 5),
            (5, 7, 7, 9),
            (2, 8, 3, 7),
            (6, 6, 4, 6),
            (2, 6, 4, 8),
        ];

        let count = count_any_overlap(&v);

        assert_eq!(4, count);
    }
}
