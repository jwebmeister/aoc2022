use std::io::prelude::*;
use thiserror::Error;

fn main() {
    let mut file_input = open_file().unwrap();
    let mut reader = std::io::BufReader::new(file_input);
}

#[derive(Error, Debug)]
enum MyError {
    #[error("Not a lowercase character, {0}")]
    NotLowercaseChar(char),
    #[error("Not UTF-8, {0:?}")]
    NotUtf8(#[from] std::str::Utf8Error),
    #[error("Not enough data for a marker size, {0}")]
    NotEnoughData(usize),
}

fn find_marker_idx<R: std::io::BufRead>(reader: &mut R) -> Result<Option<usize>, MyError> {
    const MARKER_SIZE: usize = 4;
    let mut marker_idx: usize = MARKER_SIZE;

    let mut n_buf = [0_u8; MARKER_SIZE * 4];
    match reader.read_exact(&mut n_buf) {
        Ok(_) => {}
        Err(_) => return Err(MyError::NotEnoughData(MARKER_SIZE)),
    };
    let mut c_buf = std::str::from_utf8(&n_buf)?;

    match unique_chars(c_buf) {
        Ok(b) => {
            if b {
                return Ok(Some(marker_idx));
            }
        }
        Err(e) => return Err(e),
    }

    todo!()
}

fn unique_chars(s: &str) -> Result<bool, MyError> {
    let n = s.len();
    let mut bs: u32 = 0b_0;
    for c in s.chars() {
        bs |= char_to_u32_bitset(c)?;
    }
    Ok(bs.count_ones() as usize == n)
}

fn char_to_u32_bitset(c: char) -> Result<u32, MyError> {
    let n = c as u8;
    if !c.is_ascii_lowercase() {
        return Err(MyError::NotLowercaseChar(c));
    }
    Ok(1 << (n - b'a'))
}

fn open_file() -> std::io::Result<std::fs::File> {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day6/src/input.txt";
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
    fn char_to_u32_bitset_works() {
        assert_eq!(char_to_u32_bitset('a').unwrap(), 0b_1);
        assert_eq!(char_to_u32_bitset('b').unwrap(), 0b_10);
        assert_eq!(char_to_u32_bitset('c').unwrap(), 0b_100);
        assert_eq!(
            char_to_u32_bitset('z').unwrap(),
            0b_10000000000000000000000000
        );
        assert!(matches!(
            char_to_u32_bitset('A'),
            Err(MyError::NotLowercaseChar('A'))
        ));
        assert!(matches!(
            char_to_u32_bitset('0'),
            Err(MyError::NotLowercaseChar('0'))
        ));
    }

    #[test]
    fn unique_chars_works() {
        assert!(unique_chars("abc").unwrap());
        assert!(unique_chars("abcdef").unwrap());
        assert!(unique_chars("zxy").unwrap());
        assert!(!unique_chars("aaaaa").unwrap());
        assert!(matches!(
            unique_chars("AAA"),
            Err(MyError::NotLowercaseChar('A'))
        ));
        assert!(matches!(
            unique_chars("0000"),
            Err(MyError::NotLowercaseChar('0'))
        ));
    }
}
