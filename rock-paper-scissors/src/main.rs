use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use input::Description;

fn main() {
    input::with(
        Description {
            name: "rock-paper-scissors",
            bin_name: "rock-paper-scissors".into(),
            description: "\
Takes a newline separated list,
where each row starts with 'A', 'B', or 'C',
then a space, then 'X', 'Y', or 'Z'. 
Each row is assigned a score based on the following lookup table.
Returns the sum of scores using the first values,
then the sum of scores using the second values.
-------------
'A X' => 4 | 3
'A Y' => 8 | 4
'A Z' => 3 | 8
'B X' => 1 | 1
'B Y' => 5 | 5
'B Z' => 9 | 9
'C X' => 7 | 2
'C Y' => 2 | 6
'C Z' => 6 | 7
            ",
            version: (0, 1, 0),
        },
        |input| {
            let matches_score = input.parse::<Matches<Match>>()?.score();
            let strategic_score = input.parse::<Matches<Strategy>>()?.score();

            println!("{matches_score}");
            println!("{strategic_score}");

            Ok(())
        },
    );
}

struct Matches<T>(Vec<T>);

impl<T: Score> Score for Matches<T> {
    fn score(&self) -> u64 {
        self.0.iter().map(T::score).sum()
    }
}

impl<T: From<Row>> FromStr for Matches<T> {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        input
            .lines()
            .map(Row::from_str)
            .map(|row| row.map(Row::into))
            .collect::<Result<Vec<T>, ParseError>>()
            .map(Self)
    }
}

struct Row {
    left: Left,
    right: Right,
}

impl FromStr for Row {
    type Err = ParseError;

    fn from_str(row: &str) -> Result<Self, Self::Err> {
        let mut chars = row.chars();

        let left = match chars.next() {
            Some('A') => Left::A,
            Some('B') => Left::B,
            Some('C') => Left::C,
            _ => {
                return Err(ParseError {
                    invalid: row[0..].to_owned(),
                });
            }
        };

        if chars.next() != Some(' ') {
            return Err(ParseError {
                invalid: row[1..].to_owned(),
            });
        }

        let right = match chars.next() {
            Some('X') => Right::X,
            Some('Y') => Right::Y,
            Some('Z') => Right::Z,
            _ => {
                return Err(ParseError {
                    invalid: row[2..].to_owned(),
                });
            }
        };

        if chars.next().is_some() {
            return Err(ParseError {
                invalid: row[3..].to_owned(),
            });
        }

        Ok(Self { left, right })
    }
}

enum Left {
    A,
    B,
    C,
}

enum Right {
    X,
    Y,
    Z,
}

struct Match {
    you: Hand,
    opponent: Hand,
}

impl Score for Match {
    fn score(&self) -> u64 {
        self.you.match_with(self.opponent).score() + self.you.score()
    }
}

impl From<Row> for Match {
    fn from(row: Row) -> Self {
        Self {
            you: row.right.into(),
            opponent: row.left.into(),
        }
    }
}

struct Strategy {
    choice: Outcome,
    opponent: Hand,
}

impl Score for Strategy {
    fn score(&self) -> u64 {
        self.choice.score() + self.opponent.results_in(self.choice).score()
    }
}

impl From<Row> for Strategy {
    fn from(row: Row) -> Self {
        Self {
            choice: row.right.into(),
            opponent: row.left.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Loss,
    Draw,
    Win,
}

impl Score for Outcome {
    fn score(&self) -> u64 {
        match self {
            Self::Loss => 0,
            Self::Draw => 3,
            Self::Win => 6,
        }
    }
}

impl From<Right> for Outcome {
    fn from(right: Right) -> Self {
        match right {
            Right::X => Self::Loss,
            Right::Y => Self::Draw,
            Right::Z => Self::Win,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Hand {
    Rock,
    Paper,
    Scissors,
}

impl Hand {
    fn match_with(self, opponent: Self) -> Outcome {
        use Hand::{Paper, Rock, Scissors};

        if matches!(
            (self, opponent),
            (Rock, Scissors) | (Paper, Rock) | (Scissors, Paper)
        ) {
            Outcome::Win
        } else if self == opponent {
            Outcome::Draw
        } else {
            Outcome::Loss
        }
    }

    fn results_in(self, outcome: Outcome) -> Self {
        use Hand::{Paper, Rock, Scissors};
        use Outcome::{Draw, Loss, Win};

        match (self, outcome) {
            (Rock, Loss) | (Paper, Win) | (Scissors, Draw) => Scissors,
            (Rock, Win) | (Paper, Draw) | (Scissors, Loss) => Paper,
            (Rock, Draw) | (Paper, Loss) | (Scissors, Win) => Rock,
        }
    }
}

impl Score for Hand {
    fn score(&self) -> u64 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }
}

impl From<Left> for Hand {
    fn from(left: Left) -> Self {
        match left {
            Left::A => Self::Rock,
            Left::B => Self::Paper,
            Left::C => Self::Scissors,
        }
    }
}

impl From<Right> for Hand {
    fn from(right: Right) -> Self {
        match right {
            Right::X => Self::Rock,
            Right::Y => Self::Paper,
            Right::Z => Self::Scissors,
        }
    }
}

#[derive(Debug)]
struct ParseError {
    invalid: String,
}

trait Score {
    fn score(&self) -> u64;
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let invalid = self.invalid.as_str();
        write!(f, "found invalid input '{invalid}'")
    }
}
