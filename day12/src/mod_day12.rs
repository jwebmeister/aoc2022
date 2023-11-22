use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Row width doesn't match first row, error while parsing")]
    ParseMismatchRowWidth,
}

#[derive(Clone, Copy)]
pub enum Cell {
    Start,
    End,
    Square(u8),
}

impl Cell {
    pub fn elevation(&self) -> u8 {
        match self {
            Cell::Start => 0,
            Cell::End => 25,
            Cell::Square(n) => *n,
        }
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Start => write!(f, "S"),
            Cell::End => write!(f, "E"),
            Cell::Square(n) => write!(f, "{}", (n + b'a') as char),
        }
    }
}

pub struct Grid {
    width: usize,
    height: usize,
    data: Vec<Cell>,
}

impl Grid {
    pub fn get_start_data_idx(&self) -> Option<usize> {
        self.data.iter().position(|c| matches!(c, Cell::Start))
    }

    pub fn get_end_data_idx(&self) -> Option<usize> {
        self.data.iter().position(|c| matches!(c, Cell::End))
    }

    pub fn get_start_coord(&self) -> Option<(usize, usize)> {
        self.data_idx_to_coord(self.get_start_data_idx()?)
    }

    pub fn get_end_coord(&self) -> Option<(usize, usize)> {
        self.data_idx_to_coord(self.get_end_data_idx()?)
    }

    pub fn data_idx_to_coord(&self, data_idx: usize) -> Option<(usize, usize)> {
        let row = data_idx / (self.width - 1);
        let col = data_idx % self.width;
        match (0..self.data.len()).contains(&data_idx) {
            true => Some((row, col)),
            false => None,
        }
    }

    pub fn coord_to_data_idx(&self, coord: (usize, usize)) -> Option<usize> {
        let data_idx = (coord.0 * self.width) + coord.1;
        match (0..self.data.len()).contains(&data_idx) {
            true => Some(data_idx),
            false => None,
        }
    }
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for row in 0..self.height {
            let idx_start = row * self.width;
            let idx_end = (row + 1) * self.width;
            let line = self.data[idx_start..idx_end]
                .iter()
                .map(|n| format!("{:?}", n))
                .collect::<String>();
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl std::ops::Index<(usize, usize)> for Grid {
    type Output = Cell;
    fn index(&self, index: (usize, usize)) -> &Cell {
        let i = (index.0 * self.width) + index.1;
        &self.data[i]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Cell {
        let i = (index.0 * self.width) + index.1;
        &mut self.data[i]
    }
}

fn parse_into_grid<R: std::io::BufRead>(mut reader: R) -> Result<Grid, MyError> {
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    let first_v = first_line
        .chars()
        .filter_map(parse_cell)
        .collect::<Vec<_>>();

    let width = first_v.len();
    let mut data: Vec<Cell> = Vec::with_capacity(width.pow(2));
    let mut height: usize = 1;

    data.extend(first_v);

    for line in reader.lines() {
        let l = line?;
        let mut v_line = Vec::with_capacity(width);
        v_line.extend(l.chars().filter_map(parse_cell));
        if v_line.len() != width {
            return Err(MyError::ParseMismatchRowWidth);
        };
        data.extend(v_line);
        height += 1;
    }
    let grid = Grid {
        width,
        height,
        data,
    };
    Ok(grid)
}

fn parse_cell(c: char) -> Option<Cell> {
    match c {
        'S' => Some(Cell::Start),
        'E' => Some(Cell::End),
        'a'..='z' => Some(Cell::Square(c as u8 - b'a')),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_into_grid_works() {
        #[rustfmt::skip]
        let s = 
"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

        let reader = std::io::BufReader::new(s.as_bytes());

        let g = parse_into_grid(reader).unwrap();

        assert_eq!(s, format!("{:?}", g).trim());
    }
}
