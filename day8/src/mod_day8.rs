use std::{collections::HashSet, io::prelude::*};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("error parsing row to u8, row = {0}")]
    RowParse(usize),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Ndarray(#[from] ndarray::ShapeError),
}

pub fn read_into_matrix<R: std::io::BufRead>(
    reader: &mut R,
) -> Result<ndarray::Array2<u8>, MyError> {
    let mut first_s = String::new();
    let _ = reader.read_line(&mut first_s);

    let mut data = first_s
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|n| n as u8)
        .collect::<Vec<_>>();

    let mut row_count: usize = 1;
    let col_count: usize = data.len();

    for (row_idx, line) in reader.lines().enumerate() {
        let s = line?;
        let mut v = s
            .chars()
            .filter_map(|c| c.to_digit(10))
            .map(|n| n as u8)
            .collect::<Vec<_>>();
        if v.len() > 0 && v.len() != col_count {
            return Err(MyError::RowParse(row_idx));
        }
        if v.len() != col_count {
            break;
        }
        row_count += 1;
        data.append(&mut v);
    }

    let arr = ndarray::Array2::from_shape_vec((row_count, col_count), data)?;
    Ok(arr)
}

pub fn visible_any_side<R: std::io::Read + std::io::Seek>(
    mut input: R,
) -> Result<HashSet<(usize, usize)>, MyError> {
    let mut fwd_reader = std::io::BufReader::new(&mut input);
    let (mut hs1, lines_count) = visible_top_left_right(&mut fwd_reader)?;

    let mut back_reader = rev_buf_reader::RevBufReader::new(&mut input);
    hs1.extend(visible_bottom(&mut back_reader, lines_count)?);

    Ok(hs1)
}

fn visible_top_left_right<R: std::io::BufRead>(
    reader: &mut R,
) -> Result<(HashSet<(usize, usize)>, usize), MyError> {
    let mut first_row = String::new();
    let _ = reader.read_line(&mut first_row);

    // retain max tree height for each column, use to compare against each row
    let mut top_max = first_row
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|n| n as u8)
        .collect::<Vec<_>>();

    // first row is visible, on outer perimeter
    let x_max = top_max.len() - 1;
    let mut visible: HashSet<(usize, usize)> = HashSet::with_capacity(x_max * 6);
    visible.extend((0..=x_max).map(|x| (x, 0)));

    // stores current rows tree heights
    let mut row_ns: Vec<u8> = Vec::with_capacity(x_max + 1);

    // stores lines count, for running bottom up fn
    let mut lines_count: usize = 1;

    for (y_top, line) in reader.lines().enumerate() {
        let y = y_top + 1;
        lines_count += 1;

        let row = line?;
        row_ns.clear();
        row_ns.extend(row.chars().filter_map(|c| c.to_digit(10)).map(|n| n as u8));
        if row_ns.len() != (x_max + 1) {
            return Err(MyError::RowParse(y));
        };
        // Visible from top + on outer perimeter
        visible.extend(
            std::iter::zip(row_ns.iter().enumerate(), top_max.iter_mut()).filter_map(
                |((x, n), top_n)| {
                    let xy = (x, y);
                    // visible left outer perimeter
                    if x == 0 {
                        return Some(xy);
                    };
                    // visible right outer perimeter
                    if x == x_max {
                        return Some(xy);
                    };
                    // visible from top, larger or equal to previous trees
                    if *n > *top_n {
                        *top_n = *n;
                        return Some(xy);
                    };
                    None
                },
            ),
        );
        // Visible from left, larger or equal to previous trees
        let mut left_n: u8 = 0;
        visible.extend(row_ns.iter().enumerate().filter_map(|(x, n)| {
            let xy = (x, y);
            if *n > left_n {
                left_n = *n;
                return Some(xy);
            };
            None
        }));
        // Visible from right, larger or equal to previous trees
        let mut right_n: u8 = 0;
        visible.extend(row_ns.iter().enumerate().rev().filter_map(|(x, n)| {
            let xy = (x, y);
            if *n > right_n {
                right_n = *n;
                return Some(xy);
            };
            None
        }));
    }
    Ok((visible, lines_count))
}

fn visible_bottom<R: std::io::BufRead + std::io::Read + std::io::Seek>(
    reader: &mut R,
    lines_count: usize,
) -> Result<HashSet<(usize, usize)>, MyError> {
    let mut first_row = String::new();
    let _ = reader.read_line(&mut first_row);

    // retain max tree height for each column, use to compare against each row
    let mut bottom_max = first_row
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|n| n as u8)
        .collect::<Vec<_>>();

    // first row is visible, on outer perimeter
    let x_max = bottom_max.len() - 1;
    let mut visible: HashSet<(usize, usize)> = HashSet::with_capacity(x_max * 6);
    visible.extend((0..=x_max).map(|x| (x, lines_count - 1)));

    // stores current rows tree heights
    let mut row_ns: Vec<u8> = Vec::with_capacity(x_max + 1);

    for (y_rev, line) in reader.lines().enumerate() {
        let y_bottom = y_rev + 1;
        let y = (lines_count - 1) - y_bottom;

        let row = line?;
        row_ns.clear();
        row_ns.extend(row.chars().filter_map(|c| c.to_digit(10)).map(|n| n as u8));
        if row_ns.len() != (x_max + 1) {
            return Err(MyError::RowParse(y));
        };
        // Visible from bottom
        visible.extend(
            std::iter::zip(row_ns.iter().enumerate(), bottom_max.iter_mut()).filter_map(
                |((x, n), bottom_n)| {
                    let xy = (x, y);
                    // visible from bottom, larger or equal to previous trees
                    if *n > *bottom_n {
                        *bottom_n = *n;
                        return Some(xy);
                    };
                    None
                },
            ),
        );
    }
    Ok(visible)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_any_side_works() {
        let s = "30373
25512
65332
33549
35390";
        let mut r: HashSet<(usize, usize)> = HashSet::new();
        r.extend((0..5).map(|x| (x, 0)));
        r.extend((0..5).map(|x| (x, 4)));
        r.extend((0..5).map(|y| (0, y)));
        r.extend((0..5).map(|y| (4, y)));
        r.extend([(1, 1), (2, 1), (1, 2), (3, 2), (2, 3)].into_iter());

        let inner = std::io::Cursor::new(&s);
        let hs1 = visible_any_side(inner).unwrap();

        let diff1: HashSet<_> = hs1.difference(&r).collect();
        let diff2: HashSet<_> = r.difference(&hs1).collect();

        assert!(diff1.is_empty());
        assert!(diff2.is_empty());
    }

    #[test]
    fn read_into_matrix_works() {
        let s = "30373
25512
65332
33549
35390";
        let a: Vec<u8> = vec![
            3, 0, 3, 7, 3, 2, 5, 5, 1, 2, 6, 5, 3, 3, 2, 3, 3, 5, 4, 9, 3, 5, 3, 9, 0,
        ];
        let r = ndarray::Array2::from_shape_vec((5, 5), a).unwrap();

        let mut reader = std::io::BufReader::new(s.as_bytes());
        let matrix = read_into_matrix(&mut reader).unwrap();

        assert_eq!(r, matrix);
    }
}
