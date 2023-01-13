#![warn(clippy::pedantic, clippy::nursery)]

use clap::Parser;
use num_bigint::BigUint;

mod primality;

/// A simple program to test the primality of a number
#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    /// The number to test the primality of
    number: BigUint,

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
    fn get_test(&self) -> fn(n: &BigUint) -> bool {
        match self {
            Self::TrialDivision => |n| primality::trial_division(n.try_into().unwrap()),
            Self::MillerRabin => |n| primality::miller_rabin(n, 8),
        }
    }
}

fn main() {
    let Args { number, test } = Args::parse();

    println!(
        "{} {}",
        number,
        if test.get_test()(&number) {
            match test {
                PrimalityTest::TrialDivision => "is prime",
                PrimalityTest::MillerRabin => "is probably prime",
            }
        } else {
            "is not prime"
        }
    );
}
