use std::io::prelude::*;

fn main() {
    let filepath_input = "./src/input.txt";
    let alt_filepath_input = "./day2/src/input.txt";
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
                return;
            }
        },
    };

    let reader = std::io::BufReader::new(file_input);

    let mut ourtotalscore: isize = 0;
    for (num, line) in reader.lines().enumerate() {
        match line {
            Ok(line) => {
                let round = match line.parse::<Round>() {
                    Ok(round) => round,
                    _ => {
                        println!("Error parsing input on line {}", num);
                        return;
                    }
                };
                ourtotalscore += round.score();
            }
            Err(e) => {
                println!("Error reading input on line {}", num);
                println!("{}", e);
            }
        }
    }

    println!("Our Total Score = {}", ourtotalscore);
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Round {
    theirs: Move,
    outcome: Outcome,
}

impl std::str::FromStr for Round {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut c = s.chars();
        let theirs: Move = match c.next() {
            Some(theirs) => theirs.try_into()?,
            _ => {
                return Err(());
            }
        };
        c.next();
        let outcome: Outcome = match c.next() {
            Some(outcome) => outcome.try_into()?,
            _ => {
                return Err(());
            }
        };
        Ok(Self { theirs, outcome })
    }
}

impl Round {
    fn ours(self) -> Move {
        match self.outcome {
            Outcome::Loss => self.theirs.winvs(),
            Outcome::Draw => self.theirs,
            Outcome::Win => self.theirs.lossvs(),
        }
    }

    fn score(self) -> isize {
        (self.outcome as isize) + (self.ours() as isize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Move {
    Rock = 1,
    Paper = 2,
    Scissors = 3,
}

impl Move {
    fn winvs(self) -> Move {
        match self {
            Move::Rock => Move::Scissors,
            Move::Paper => Move::Rock,
            Move::Scissors => Move::Paper,
        }
    }

    fn lossvs(self) -> Move {
        match self {
            Move::Rock => Move::Paper,
            Move::Paper => Move::Scissors,
            Move::Scissors => Move::Rock,
        }
    }
}

impl TryFrom<char> for Move {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'A' => Ok(Move::Rock),
            'B' => Ok(Move::Paper),
            'C' => Ok(Move::Scissors),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Outcome {
    Loss = 0,
    Draw = 3,
    Win = 6,
}

impl TryFrom<char> for Outcome {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'X' => Ok(Outcome::Loss),
            'Y' => Ok(Outcome::Draw),
            'Z' => Ok(Outcome::Win),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_from_char_works() {
        let a = Move::try_from('A').unwrap();
        assert_eq!(a, Move::Rock);

        let b = Move::try_from('B').unwrap();
        assert_eq!(b, Move::Paper);

        let c = Move::try_from('C').unwrap();
        assert_eq!(c, Move::Scissors);

        assert_eq!(Move::try_from(' '), Err(()));
    }

    #[test]
    fn outcome_from_char_works() {
        let x = Outcome::try_from('X').unwrap();
        assert_eq!(x, Outcome::Loss);

        let y = Outcome::try_from('Y').unwrap();
        assert_eq!(y, Outcome::Draw);

        let z = Outcome::try_from('Z').unwrap();
        assert_eq!(z, Outcome::Win);

        assert_eq!(Outcome::try_from(' '), Err(()));
    }

    #[test]
    fn round_from_string_works() {
        let s = "A Y";
        assert_eq!(
            s.parse::<Round>().unwrap(),
            Round {
                theirs: Move::Rock,
                outcome: Outcome::Draw,
            }
        );
    }

    #[test]
    fn game_scoring_works() {
        let s = "A Y\nB X\nC Z";
        let mut r = s.lines().map(|l| l.parse::<Round>());
        let mut total_score = 0;

        let Some(Ok(x)) = r.next() else {
            return;
        };
        assert_eq!(x.ours(), Move::Rock);
        assert_eq!(x.score(), 4);
        total_score += x.score();

        let Some(Ok(x)) = r.next() else {
            return;
        };
        assert_eq!(x.ours(), Move::Rock);
        assert_eq!(x.score(), 1);
        total_score += x.score();

        let Some(Ok(x)) = r.next() else {
            return;
        };
        assert_eq!(x.ours(), Move::Rock);
        assert_eq!(x.score(), 7);
        total_score += x.score();

        assert_eq!(total_score, 12);
    }
}
