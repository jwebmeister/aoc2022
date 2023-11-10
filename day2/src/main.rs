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
    ours: Move,
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
        let ours: Move = match c.next() {
            Some(ours) => ours.try_into()?,
            _ => {
                return Err(());
            }
        };
        Ok(Self { theirs, ours })
    }
}

impl Round {
    fn outcome(self) -> Outcome {
        if self.ours == self.theirs {
            return Outcome::Draw;
        };
        if (self.ours as isize) == (self.theirs as isize + 1) {
            return Outcome::Win;
        };
        if (self.ours == Move::Rock) && (self.theirs == Move::Scissors) {
            return Outcome::Win;
        };
        Outcome::Loss
    }

    fn score(self) -> isize {
        (self.ours as isize) + (self.outcome() as isize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Move {
    Rock = 1,
    Paper = 2,
    Scissors = 3,
}

impl TryFrom<char> for Move {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'A' | 'X' => Ok(Move::Rock),
            'B' | 'Y' => Ok(Move::Paper),
            'C' | 'Z' => Ok(Move::Scissors),
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn rock_beats_scissors() {
        assert_eq!(
            Outcome::Win,
            Round {
                theirs: Move::Scissors,
                ours: Move::Rock
            }
            .outcome()
        );
        assert_eq!(
            Outcome::Loss,
            Round {
                theirs: Move::Rock,
                ours: Move::Scissors
            }
            .outcome()
        );
    }

    #[test]
    fn paper_beats_rock() {
        assert_eq!(
            Outcome::Win,
            Round {
                theirs: Move::Rock,
                ours: Move::Paper
            }
            .outcome()
        );
        assert_eq!(
            Outcome::Loss,
            Round {
                theirs: Move::Paper,
                ours: Move::Rock
            }
            .outcome()
        );
    }

    #[test]
    fn scissors_beats_paper() {
        assert_eq!(
            Outcome::Win,
            Round {
                theirs: Move::Paper,
                ours: Move::Scissors
            }
            .outcome()
        );
        assert_eq!(
            Outcome::Loss,
            Round {
                theirs: Move::Scissors,
                ours: Move::Paper
            }
            .outcome()
        );
    }

    #[test]
    fn same_move_equals_draw() {
        assert_eq!(
            Outcome::Draw,
            Round {
                theirs: Move::Rock,
                ours: Move::Rock
            }
            .outcome()
        );
        assert_eq!(
            Outcome::Draw,
            Round {
                theirs: Move::Paper,
                ours: Move::Paper
            }
            .outcome()
        );
        assert_eq!(
            Outcome::Draw,
            Round {
                theirs: Move::Scissors,
                ours: Move::Scissors
            }
            .outcome()
        );
    }

    #[test]
    fn move_from_char_works() {
        let a = Move::try_from('A').unwrap();
        let x = Move::try_from('X').unwrap();
        assert_eq!(a, Move::Rock);
        assert_eq!(x, Move::Rock);

        let b = Move::try_from('B').unwrap();
        let y = Move::try_from('Y').unwrap();
        assert_eq!(b, Move::Paper);
        assert_eq!(y, Move::Paper);

        let c = Move::try_from('C').unwrap();
        let z = Move::try_from('Z').unwrap();
        assert_eq!(c, Move::Scissors);
        assert_eq!(z, Move::Scissors);

        assert_eq!(Move::try_from(' '), Err(()));
    }

    #[test]
    fn round_from_string_works() {
        let s = "A Y";
        assert_eq!(
            Round::from_str(s).unwrap(),
            Round {
                theirs: Move::Rock,
                ours: Move::Paper
            }
        );
    }

    #[test]
    fn game_scoring_works() {
        let s = "A Y\nB X\nC Z";
        let mut r = s.lines().map(|l| Round::from_str(l));
        let mut total_score = 0;

        let Some(Ok(x)) = r.next() else {
            return;
        };
        assert_eq!(x.outcome(), Outcome::Win);
        assert_eq!(x.score(), 8);
        total_score += x.score();

        let Some(Ok(x)) = r.next() else {
            return;
        };
        assert_eq!(x.outcome(), Outcome::Loss);
        assert_eq!(x.score(), 1);
        total_score += x.score();

        let Some(Ok(x)) = r.next() else {
            return;
        };
        assert_eq!(x.outcome(), Outcome::Draw);
        assert_eq!(x.score(), 6);
        total_score += x.score();

        assert_eq!(total_score, 15);
    }
}
