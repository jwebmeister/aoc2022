use thiserror::Error;

fn main() {
    let file_input = open_file().unwrap();
    let mut reader = std::io::BufReader::new(file_input);
    let marker_idx = find_marker_idx(&mut reader).unwrap().unwrap();

    println!("Marker index = {}", marker_idx);
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

    let mut n_buf = [0_u8; MARKER_SIZE];
    match reader.read_exact(&mut n_buf) {
        Ok(_) => {}
        Err(_) => return Err(MyError::NotEnoughData(MARKER_SIZE)),
    };

    match unique_chars(&n_buf) {
        Ok(b) => {
            if b {
                return Ok(Some(marker_idx));
            }
        }
        Err(e) => return Err(e),
    }

    let mut next_n = [0_u8; 1];
    while reader.read_exact(&mut next_n).is_ok() {
        marker_idx += 1;
        match next_n[0].is_ascii_lowercase() {
            true => n_buf[(marker_idx - 1) % MARKER_SIZE] = next_n[0],
            false => return Err(MyError::NotLowercaseChar(next_n[0] as char)),
        };
        match unique_chars(&n_buf) {
            Ok(b) => {
                if b {
                    return Ok(Some(marker_idx));
                }
            }
            Err(e) => return Err(e),
        }
    }

    Ok(None)
}

fn unique_chars(n_buf: &[u8]) -> Result<bool, MyError> {
    let count = n_buf.len();
    let mut bs: u32 = 0b_0;
    for n in n_buf {
        bs |= u8_to_u32_bitset(*n)?;
    }
    Ok(bs.count_ones() as usize == count)
}

fn u8_to_u32_bitset(n: u8) -> Result<u32, MyError> {
    if !n.is_ascii_lowercase() {
        return Err(MyError::NotLowercaseChar(n as char));
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
    fn u8_to_u32_bitset_works() {
        assert_eq!(u8_to_u32_bitset('a' as u8).unwrap(), 0b_1);
        assert_eq!(u8_to_u32_bitset('b' as u8).unwrap(), 0b_10);
        assert_eq!(u8_to_u32_bitset('c' as u8).unwrap(), 0b_100);
        assert_eq!(
            u8_to_u32_bitset('z' as u8).unwrap(),
            0b_10000000000000000000000000
        );
        assert!(matches!(
            u8_to_u32_bitset('A' as u8),
            Err(MyError::NotLowercaseChar('A'))
        ));
        assert!(matches!(
            u8_to_u32_bitset('0' as u8),
            Err(MyError::NotLowercaseChar('0'))
        ));
        assert!(matches!(
            u8_to_u32_bitset(255 as u8),
            Err(MyError::NotLowercaseChar(_))
        ));
    }

    #[test]
    fn unique_chars_works() {
        assert!(unique_chars(&[b'a', b'b', b'c']).unwrap());
        assert!(matches!(
            unique_chars(&[b'A', b'A', b'A']),
            Err(MyError::NotLowercaseChar('A'))
        ));
        assert!(matches!(
            unique_chars(&[b'0', b'0', b'0']),
            Err(MyError::NotLowercaseChar('0'))
        ));
    }

    #[test]
    fn find_marker_idx_works() {
        let s = "mjqjpqmgbljsphdztnvjfqwrcgsmlb".as_bytes();
        let mut reader = std::io::BufReader::new(s);
        assert_eq!(Some(7), find_marker_idx(&mut reader).unwrap());

        let s = "bvwbjplbgvbhsrlpgdmjqwftvncz".as_bytes();
        let mut reader = std::io::BufReader::new(s);
        assert_eq!(Some(5), find_marker_idx(&mut reader).unwrap());

        let s = "nppdvjthqldpwncqszvftbrmjlhg".as_bytes();
        let mut reader = std::io::BufReader::new(s);
        assert_eq!(Some(6), find_marker_idx(&mut reader).unwrap());

        let s = "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg".as_bytes();
        let mut reader = std::io::BufReader::new(s);
        assert_eq!(Some(10), find_marker_idx(&mut reader).unwrap());

        let s = "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw".as_bytes();
        let mut reader = std::io::BufReader::new(s);
        assert_eq!(Some(11), find_marker_idx(&mut reader).unwrap());
    }
}
