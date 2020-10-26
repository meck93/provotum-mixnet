use core::ops::{AddAssign, Sub};
use num_bigint::{BigUint, RandBigInt};
use num_traits::{One, Zero};
use rand;

// generate a random value: 0 < x < number
pub fn random_biguint(number: &BigUint) -> BigUint {
    assert!(*number > BigUint::zero(), "q must be greater than zero!");
    let one = BigUint::one();
    let upper_bound = number.clone().sub(one);
    let bit_size: u64 = upper_bound.bits();

    let mut rng = rand::thread_rng();
    rng.gen_biguint(bit_size) % number
}

pub fn generate_random_prime(bit_size: u64) -> BigUint {
    let mut rng = rand::thread_rng();
    let mut candidate = rng.gen_biguint(bit_size);
    let two = BigUint::from(2 as u32);

    if &candidate % &two == BigUint::zero() {
        candidate.add_assign(BigUint::one())
    }

    while !is_prime(&candidate, 128) {
        candidate.add_assign(two.clone());
    }
    candidate
}

// Miller-Rabin Primality Test
// https://en.wikipedia.org/wiki/Miller-Rabin_primality_test
pub fn is_prime(num: &BigUint, certainty: u32) -> bool {
    let zero: BigUint = BigUint::zero();
    let one: BigUint = BigUint::one();
    let two = one.clone() + one.clone();

    if *num == two {
        return true;
    }

    if num % two.clone() == zero.clone() {
        return false;
    }

    let num_less_one = num - one.clone();

    // write n-1 as 2**s * d
    let mut d = num_less_one.clone();
    let mut s: BigUint = Zero::zero();

    while d.clone() % two.clone() == zero.clone() {
        d = d / two.clone();
        s = s + one.clone();
    }

    let mut k = 0;
    let mut rng = rand::thread_rng();

    // test for probable prime
    while k < certainty {
        let a = rng.gen_biguint_range(&two, num);
        let mut x = a.modpow(&d, num);
        if x != one.clone() && x != num_less_one {
            let mut r = zero.clone();
            loop {
                x = x.modpow(&two, num);
                if x == num_less_one {
                    break;
                } else if x == one.clone() || r == (s.clone() - one.clone()) {
                    return false;
                }
                r = r + one.clone();
            }
        }
        k += 2;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{generate_random_prime, is_prime, random_biguint};
    use num_bigint::BigUint;

    #[test]
    fn it_should_generate_random_number() {
        let number = BigUint::parse_bytes(b"123", 10).unwrap();
        for _ in 0..20 {
            let random = self::random_biguint(&number);
            assert!(random < number);
        }
    }

    #[test]
    fn check_that_2_is_prime() {
        let number = BigUint::parse_bytes(b"2", 10).unwrap();
        let is_prime = self::is_prime(&number, 20);
        assert!(is_prime);
    }

    #[test]
    fn check_that_11_is_prime() {
        let number = BigUint::from(11 as u32);
        let is_prime = self::is_prime(&number, 20);
        assert!(is_prime);
    }

    #[test]
    fn check_that_84532559_is_prime() {
        let number = BigUint::parse_bytes(b"84532559", 10).unwrap();
        let is_prime = self::is_prime(&number, 20);
        assert!(is_prime);
    }

    #[test]
    fn check_that_84532560_is_not_prime() {
        let number = BigUint::parse_bytes(b"84532560", 10).unwrap();
        let is_prime = self::is_prime(&number, 20);
        assert!(!is_prime);
    }

    #[test]
    fn should_generate_random_prime() {
        let bit_size = 256;
        let prime = self::generate_random_prime(bit_size);
        let is_prime = self::is_prime(&prime, 128);
        assert!(is_prime);
    }
}
