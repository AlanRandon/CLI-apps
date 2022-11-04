use clap::Parser;

mod primality;

/// A simple program to test the primality of a number
#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    /// The number to test the primality of
    number: u128,
}

fn main() {
    let Args { number } = Args::parse();

    println!(
        "{} {}",
        number,
        if primality::trial_division(number) {
            "is prime"
        } else {
            "is not prime"
        }
    );
}
