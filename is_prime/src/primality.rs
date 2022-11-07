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

/// Performs the Miller-Rabin primality test to determine if a number is prime
///
/// If the function returns false, n is not prime.
/// If the function returns true, n is very likely not a prime.
pub fn miller_rabin(n: &BigUint, passes: usize) -> bool {
    match n.try_into() {
        Ok(0 | 1 | 4 | 6 | 8 | 9) => false,
        Ok(2 | 3 | 5 | 7) => true,
        _ => {
            let one_less = n - 1_u8;

            let mut s: u32 = 0;
            let mut d = one_less.clone();
            while &d % 2_u8 == BigUint::from(0_u8) {
                d >>= 1;
                s += 1;
            }

            let one = BigUint::from(1_u8);
            let two = BigUint::from(2_u8);

            let trial_composite = |a: BigUint| {
                if a.modpow(&d, n) == one {
                    return false;
                }

                for i in 0..s {
                    if a.modpow(&(two.pow(i) * &d), n) == one_less {
                        return false;
                    }
                }

                true
            };

            for _ in 0..passes {
                let a = thread_rng().gen_biguint_range(&two, n);
                if trial_composite(a) {
                    return false;
                }
            }

            true
        }
    }
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
        (1_u128, false),
        (3, true),
        (10, false),
        (8128, false),
        (9_061_530_241, true),
        (845_689_124_236_832_528_811_234_994_303, true),
    ] {
        assert_eq!(
            miller_rabin(&BigUint::from(number), 8),
            is_prime,
            "{} {} prime",
            number,
            if is_prime { "is" } else { "isn't" }
        );
    }
}
