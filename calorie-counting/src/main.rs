use std::error::Error;
use std::fmt::Display;
use std::num::ParseIntError;

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
            let elves = Elves::try_from(&input)?;
            let max_calories = elves.max_calories();
            let max_calorie_sum = elves.max_calorie_sum(3);

            println!("{max_calories}");
            println!("{max_calorie_sum}");

            Ok(())
        },
    );
}

struct Elves {
    elves: Vec<Elf>,
}

impl Elves {
    fn try_from(calories: &str) -> Result<Self, ParseError> {
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

    fn max_calories(&self) -> u64 {
        self.elves
            .iter()
            .map(|elf| elf.rations.iter().map(|ration| ration.calories).sum())
            .max()
            .unwrap()
    }

    fn max_calorie_sum(&self, top: usize) -> u64 {
        self.elves
            .iter()
            .map(|elf| elf.rations.iter().map(|ration| ration.calories).sum())
            .fold(Vec::with_capacity(top), |mut tops, calories: u64| {
                if tops.len() < top {
                    tops.push(calories);
                } else {
                    tops.sort_unstable();
                    let Some(bottom) = tops.first_mut() else { return tops };
                    *bottom = calories.max(*bottom);
                }
                tops
            })
            .iter()
            .sum()
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
