use std::error::Error;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

use input::Description;

fn main() {
    input::with(
        Description {
            name: "calorie-counting",
            bin_name: "calorie-counting".into(),
            description: "\
Takes a list of numbers, zero or one per line.
Sums all consecutive numbers not separated by an empty line,
then returns the largest sum and the sum of the largest 3 sums.",
            version: (0, 1, 0),
        },
        |input| {
            let elves = input.parse::<Elves>()?;
            let top = elves.sum_calories_top::<1>();
            let top_three = elves.sum_calories_top::<3>();

            println!("{top}");
            println!("{top_three}");

            Ok(())
        },
    );
}

struct Elves {
    elves: Vec<Elf>,
}

impl Elves {
    fn sum_calories_top<const N: usize>(&self) -> u64 {
        self.elves
            .iter()
            .map(|elf| elf.rations.iter().map(|ration| ration.calories).sum())
            .fold([0; N], |mut tops, calories: u64| {
                tops.sort_unstable();

                if let Some(lowest) = tops.first_mut() {
                    *lowest = calories.max(*lowest);
                }

                tops
            })
            .iter()
            .sum()
    }
}

impl FromStr for Elves {
    type Err = ParseError;

    fn from_str(calories: &str) -> Result<Self, ParseError> {
        let mut elves = Vec::new();
        let mut rations = Vec::new();

        for line in calories.lines() {
            if line.is_empty() {
                elves.push(Elf { rations });
                rations = Vec::new();
            } else {
                let calories = line.parse::<u64>().map_err(ParseError)?;
                rations.push(Ration { calories });
            }
        }

        Ok(Self { elves })
    }
}

struct Elf {
    rations: Vec<Ration>,
}

struct Ration {
    calories: u64,
}

#[derive(Debug)]
struct ParseError(ParseIntError);

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected an integer; only digits and newlines are valid input"
        )
    }
}
