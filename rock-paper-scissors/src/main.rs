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
Returns the sum of those scores.
----------
'B X' => 1
'C Y' => 2
'A Z' => 3
'A X' => 4
'B Y' => 5
'C Z' => 6
'C X' => 7
'A Y' => 8
'B Z' => 9",
            version: (0, 1, 0),
        },
        |input| {
            let matches = input.parse::<Matches>()?;
            let total_score = matches.score();

            println!("{total_score}");

            Ok(())
        },
    )
}

struct Matches(Vec<Match>);

impl Matches {
    fn score(&self) -> u64 {
        self.0.iter().map(Match::score).sum()
    }
}

impl FromStr for Matches {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        input
            .lines()
            .map(|row| {
                let mut chars = row.chars();

                let opponent = match chars.next() {
                    Some('A') => Hand::Rock,
                    Some('B') => Hand::Paper,
                    Some('C') => Hand::Scissors,
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

                let you = match chars.next() {
                    Some('X') => Hand::Rock,
                    Some('Y') => Hand::Paper,
                    Some('Z') => Hand::Scissors,
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

                Ok(Match { you, opponent })
            })
            .collect::<Result<Vec<Match>, ParseError>>()
            .map(Self)
    }
}

struct Match {
    you: Hand,
    opponent: Hand,
}

impl Match {
    fn score(&self) -> u64 {
        self.outcome().score() + self.you.score()
    }

    fn outcome(&self) -> Outcome {
        if self.is_win() {
            Outcome::Win
        } else if self.you == self.opponent {
            Outcome::Draw
        } else {
            Outcome::Loss
        }
    }

    fn is_win(&self) -> bool {
        matches!(
            (self.you, self.opponent),
            (Hand::Rock, Hand::Scissors)
                | (Hand::Paper, Hand::Rock)
                | (Hand::Scissors, Hand::Paper)
        )
    }
}

#[derive(PartialEq, Eq)]
enum Outcome {
    Loss,
    Draw,
    Win,
}

impl Outcome {
    fn score(&self) -> u64 {
        match self {
            Self::Loss => 0,
            Self::Draw => 3,
            Self::Win => 6,
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
    fn score(self) -> u64 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }
}

#[derive(Debug)]
struct ParseError {
    invalid: String,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let invalid = self.invalid.as_str();
        write!(f, "found invalid input '{invalid}'")
    }
}
