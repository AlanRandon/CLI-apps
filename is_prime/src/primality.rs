use integer_sqrt::IntegerSquareRoot;
use rayon::prelude::*;

pub fn trial_division(n: u128) -> bool {
    match n {
        0 | 1 => false,
        _ => {
            matches!(
                (2..n.integer_sqrt() + 1)
                    .into_par_iter()
                    .find_any(|i| n % i == 0),
                None
            )
        }
    }
}
