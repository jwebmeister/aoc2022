use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Row width doesn't match first row, error while parsing")]
    ParseMismatchRowWidth,
    #[error("Invalid coordinate for grid. Coord={0:?}")]
    InvalidGridCoordinate((usize, usize)),
    #[error("Couldn't find coordinate in BFS visited coordinates. Coord={0:?}")]
    BFSNotVisitedCoord((usize, usize)),
}

#[derive(Debug, Default)]
pub struct Bfs {
    pub visited: HashMap<(usize, usize), Option<(usize, usize)>>,
    pub current: HashSet<(usize, usize)>,
    pub num_steps: usize,
}

impl Bfs {
    pub fn new() -> Self {
        Bfs::default()
    }

    pub fn step(&mut self, grid: &Grid) {
        if self.current.is_empty() && self.num_steps == 0 {
            let start_coord = grid.get_start_coord().unwrap();
            self.current.insert(start_coord);
            self.visited.insert(start_coord, None);
            return;
        };

        let mut next: HashSet<(usize, usize)> = HashSet::new();

        for curr in &self.current {
            for next_coord in grid.get_available_moves(*curr).unwrap() {
                if self.visited.contains_key(&next_coord) {
                    continue;
                };
                self.visited.insert(next_coord, Some(*curr));
                next.insert(next_coord);
            }
        }
        self.current = next;
        self.num_steps += 1;
    }

    pub fn step_up(&mut self, grid: &Grid) {
        if self.current.is_empty() && self.num_steps == 0 {
            let start_coords = grid.get_coords_elev_zero();
            self.current.extend(start_coords.iter());
            self.visited.extend(start_coords.iter().map(|x| (*x, None)));
            return;
        };

        let mut next: HashSet<(usize, usize)> = HashSet::new();

        for curr in &self.current {
            for next_coord in grid.get_available_moves(*curr).unwrap() {
                if self.visited.contains_key(&next_coord) {
                    continue;
                };
                self.visited.insert(next_coord, Some(*curr));
                next.insert(next_coord);
            }
        }
        self.current = next;
        self.num_steps += 1;
    }

    pub fn step_down(&mut self, grid: &Grid) {
        if self.current.is_empty() && self.num_steps == 0 {
            let start_coord = grid.get_end_coord().unwrap();
            self.current.insert(start_coord);
            self.visited.insert(start_coord, None);
            return;
        };

        let mut next: HashSet<(usize, usize)> = HashSet::new();

        for curr in &self.current {
            for next_coord in grid.get_available_moves_down(*curr).unwrap() {
                if self.visited.contains_key(&next_coord) {
                    continue;
                };
                self.visited.insert(next_coord, Some(*curr));
                next.insert(next_coord);
            }
        }
        self.current = next;
        self.num_steps += 1;
    }

    pub fn trace_back_path(&self, coord: (usize, usize)) -> Result<Vec<(usize, usize)>, MyError> {
        let mut back_path: Vec<(usize, usize)> = Vec::new();
        if !self.visited.contains_key(&coord) {
            return Err(MyError::BFSNotVisitedCoord(coord));
        }
        back_path.push(coord);

        let maybe_next_coord = self.visited[&coord];
        match maybe_next_coord {
            Some(next_coord) => {
                let r = self.trace_back_path(next_coord)?;
                back_path.extend(r);
            }
            None => return Ok(back_path),
        };

        Ok(back_path)
    }
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
    pub fn get_available_moves(
        &self,
        coord: (usize, usize),
    ) -> Result<HashSet<(usize, usize)>, MyError> {
        let irow = coord.0 as isize;
        let icol = coord.1 as isize;

        let ref_cell = self
            .get_cell_from_coord(coord)
            .ok_or(MyError::InvalidGridCoordinate(coord))?;

        if matches!(ref_cell, Cell::End) {
            return Ok(HashSet::new());
        }

        let imoves = [
            (irow - 1, icol),
            (irow + 1, icol),
            (irow, icol - 1),
            (irow, icol + 1),
        ];

        let umoves = imoves
            .iter()
            .filter(|x| {
                !(x.0 < 0 || x.1 < 0 || x.0 >= self.height as isize || x.1 >= self.width as isize)
            })
            .map(|x| (x.0 as usize, x.1 as usize))
            .filter(|x| match self.get_cell_from_coord(*x) {
                Some(cell) => cell.elevation() <= (ref_cell.elevation() + 1),
                None => false,
            })
            .collect::<HashSet<(usize, usize)>>();

        Ok(umoves)
    }

    pub fn get_available_moves_down(
        &self,
        coord: (usize, usize),
    ) -> Result<HashSet<(usize, usize)>, MyError> {
        let irow = coord.0 as isize;
        let icol = coord.1 as isize;

        let ref_cell = self
            .get_cell_from_coord(coord)
            .ok_or(MyError::InvalidGridCoordinate(coord))?;

        if ref_cell.elevation() == 0 {
            return Ok(HashSet::new());
        }

        let imoves = [
            (irow - 1, icol),
            (irow + 1, icol),
            (irow, icol - 1),
            (irow, icol + 1),
        ];

        let umoves = imoves
            .iter()
            .filter(|x| {
                !(x.0 < 0 || x.1 < 0 || x.0 >= self.height as isize || x.1 >= self.width as isize)
            })
            .map(|x| (x.0 as usize, x.1 as usize))
            .filter(|x| match self.get_cell_from_coord(*x) {
                Some(cell) => cell.elevation() >= (ref_cell.elevation() - 1),
                None => false,
            })
            .collect::<HashSet<(usize, usize)>>();

        Ok(umoves)
    }

    pub fn is_valid_coord(&self, coord: (usize, usize)) -> bool {
        let row = coord.0;
        let col = coord.1;
        let data_idx = (row * self.width) + col;
        (0..self.height).contains(&row)
            && (0..self.width).contains(&col)
            && (0..self.data.len()).contains(&data_idx)
    }

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

    pub fn get_cell_from_coord(&self, coord: (usize, usize)) -> Option<&Cell> {
        if !self.is_valid_coord(coord) {
            return None;
        };
        let i = (coord.0 * self.width) + coord.1;
        Some(&self.data[i])
    }

    pub fn _get_mut_cell_from_coord(&mut self, coord: (usize, usize)) -> Option<&mut Cell> {
        if !self.is_valid_coord(coord) {
            return None;
        };
        let i = (coord.0 * self.width) + coord.1;
        Some(&mut self.data[i])
    }

    pub fn data_idx_to_coord(&self, data_idx: usize) -> Option<(usize, usize)> {
        let row = data_idx / self.width;
        let col = data_idx % self.width;
        let coord = (row, col);
        match self.is_valid_coord(coord) {
            true => Some(coord),
            false => None,
        }
    }

    pub fn _coord_to_data_idx(&self, coord: (usize, usize)) -> Option<usize> {
        let row = coord.0;
        let col = coord.1;
        let data_idx = (row * self.width) + col;
        match self.is_valid_coord(coord) {
            true => Some(data_idx),
            false => None,
        }
    }

    pub fn get_coords_elev_zero(&self) -> Vec<(usize, usize)> {
        let mut v: Vec<(usize, usize)> = Vec::new();
        for (data_idx, cell) in self.data.iter().enumerate() {
            if cell.elevation() == 0 {
                v.push(self.data_idx_to_coord(data_idx).unwrap());
            }
        }
        v
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
                .fold(String::new(), |mut output, n| {
                    let _ = write!(output, "{:?}", *n);
                    output
                });
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

pub fn parse_into_grid<R: std::io::BufRead>(mut reader: R) -> Result<Grid, MyError> {
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

    #[test]
    fn bfs_works() {
        #[rustfmt::skip]
        let s = 
"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

        let reader = std::io::BufReader::new(s.as_bytes());

        let grid = parse_into_grid(reader).unwrap();

        let mut bfs = Bfs::new();
        bfs.step(&grid);
        while !bfs.current.contains(&grid.get_end_coord().unwrap()) {
            bfs.step(&grid);
        }
        let mut path = bfs.trace_back_path(grid.get_end_coord().unwrap()).unwrap();
        path.reverse();

        assert_eq!(31, &path.len() - 1);
    }

    #[test]
    fn bfs_up_works() {
        #[rustfmt::skip]
        let s = 
"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

        let reader = std::io::BufReader::new(s.as_bytes());

        let grid = parse_into_grid(reader).unwrap();

        let mut bfs = Bfs::new();
        bfs.step_up(&grid);
        while !bfs.current.contains(&grid.get_end_coord().unwrap()) {
            bfs.step_up(&grid);
        }
        let mut path = bfs.trace_back_path(grid.get_end_coord().unwrap()).unwrap();
        path.reverse();

        assert_eq!(29, &path.len() - 1);
    }

    #[test]
    fn bfs_down_works() {
        #[rustfmt::skip]
        let s = 
"Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

        let reader = std::io::BufReader::new(s.as_bytes());

        let grid = parse_into_grid(reader).unwrap();

        let mut bfs = Bfs::new();
        bfs.step_down(&grid);
        while !bfs.current.is_empty()
            && !bfs
                .current
                .iter()
                .map(|coord| grid.get_cell_from_coord(*coord).unwrap().elevation())
                .any(|x| x == 0)
        {
            bfs.step_down(&grid);
            if bfs.num_steps >= 300 {
                panic!("Too many steps")
            };
        }
        let end_coords = bfs
            .current
            .iter()
            .filter(|coord| grid.get_cell_from_coord(**coord).unwrap().elevation() == 0)
            .collect::<Vec<_>>();
        let path1 = bfs.trace_back_path(*end_coords[0]).unwrap();

        assert_eq!(29, &path1.len() - 1);
    }
}
