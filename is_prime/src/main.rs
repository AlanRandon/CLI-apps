#![warn(clippy::pedantic, clippy::nursery)]

use clap::Parser;
use num_bigint::BigUint;

mod primality;

/// A simple program to test the primality of a number
#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    /// The number to test the primality of
    number: u128,

    // The primality test to use
    #[arg(long, short, value_enum, default_value_t = PrimalityTest::TrialDivision)]
    test: PrimalityTest,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum PrimalityTest {
    TrialDivision,
    MillerRabin,
}

impl PrimalityTest {
    fn get_test(&self) -> fn(n: u128) -> bool {
        match self {
            Self::TrialDivision => primality::trial_division,
            Self::MillerRabin => |n| primality::miller_rabin(&BigUint::from(n), 8),
        }
    }
}

fn main() {
    let Args { number, test } = Args::parse();

    println!(
        "{} {}",
        number,
        if test.get_test()(number) {
            "is prime"
        } else {
            "is not prime"
        }
    );
}
