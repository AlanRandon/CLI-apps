use integer_sqrt::IntegerSquareRoot;
use num_bigint::{BigUint, RandBigInt};
use rand::thread_rng;
use rayon::prelude::*;

/// Performs trial division to check for prime numbers
pub fn trial_division(n: u128) -> bool {
    match n {
        0 | 1 => false,
        _ => {
            matches!(
                (2..=n.integer_sqrt())
                    .into_par_iter()
                    .find_any(|i| n % i == 0),
                None
            )
        }
    }
}

pub fn miller_rabin(n: u128, passes: usize) -> bool {
    if matches!(n, 0 | 1 | 4 | 6 | 8 | 9) {
        return false;
    }

    if matches!(n, 2 | 3 | 5 | 7) {
        return true;
    }

    let mut s = 0;
    let mut d = n - 1;
    while d % 2 == 0 {
        d >>= 1;
        s += 1;
    }

    let one = BigUint::from(1_u8);
    let two = BigUint::from(2_u8);

    let trial_composite = |a: BigUint| {
        if a.pow(d.try_into().expect("d is too big")) % n == one {
            return false;
        }

        for i in 0..s {
            if a.pow((two.pow(i) * d).try_into().expect("d is too big")) % n == BigUint::from(n - 1)
            {
                return false;
            }
        }

        true
    };

    for _ in 0..passes {
        println!("Gnereating big range");
        let a = thread_rng().gen_biguint_range(&two, &BigUint::from(n));
        if trial_composite(a) {
            return false;
        }
    }

    true
}

#[test]
fn trial_division_test() {
    for (number, is_prime) in [
        (1, false),
        (3, true),
        (10, false),
        (8128, false),
        (9_061_530_241, true),
    ] {
        assert_eq!(
            trial_division(number),
            is_prime,
            "{} {} prime",
            number,
            if is_prime { "is" } else { "isn't" }
        );
    }
}

#[test]
fn miller_rabin_test() {
    for (number, is_prime) in [
        (1, false),
        (3, true),
        (10, false),
        (8128, false),
        (9_061_530_241, true),
    ] {
        assert_eq!(
            miller_rabin(number, 8),
            is_prime,
            "{} {} prime",
            number,
            if is_prime { "is" } else { "isn't" }
        );
    }
}
