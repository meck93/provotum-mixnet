use alloc::vec::Vec;
use core::ops::{Add, Div, Mul, Sub};
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct ElGamalParams {
    // modulus: p
    pub p: BigUint,

    // 1. public generator g
    pub g: BigUint,

    // 2. public generator h
    pub h: BigUint,
}

impl ElGamalParams {
    // q:
    // q is valid if it is prime
    pub fn q(&self) -> BigUint {
        (self.p.clone().sub(BigUint::one())).div(BigUint::from(2u32))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PublicKey {
    // system parameters (p, g)
    pub params: ElGamalParams,

    // public key: h = g^x mod p
    // - g: generator
    // - x: private key
    pub h: BigUint,
}

impl PublicKey {
    pub fn combine_public_keys_bigunits(self, others: &[BigUint]) -> Self {
        assert!(!others.is_empty(), "there must be at least another key!");
        let mut h: BigUint = self.h.clone();
        others.iter().for_each(|pk| h *= pk);
        h %= self.params.p.clone();
        PublicKey {
            h,
            params: self.params,
        }
    }

    pub fn combine_public_keys(self, others: &[PublicKey]) -> Self {
        assert!(!others.is_empty(), "there must be at least another key!");
        let mut h: BigUint = self.h.clone();
        others.iter().for_each(|pk| h *= pk.h.clone());
        h %= self.params.p.clone();
        PublicKey {
            h,
            params: self.params,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct PrivateKey {
    // system parameters (p, g)
    pub params: ElGamalParams,

    // private key: x
    // - x: a random value (x ∈ Zq)
    pub x: BigUint,
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct Cipher {
    // a = g^r mod p
    // - g: generator
    // - r: random value (r ∈ Zq)
    pub a: BigUint,

    // b = h^r*g^m mod p
    // - h: public key
    // - m: message
    pub b: BigUint,
}
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct PermutationCommitment {
    pub commitments: Vec<BigUint>,
    pub randoms: Vec<BigUint>,
}

/// Algorithm 8.47: The public value Y
pub type BigY<'a> = (
    Vec<Cipher>,  // e
    Vec<Cipher>,  // e_tilde
    Vec<BigUint>, // vec_c
    Vec<BigUint>, // vec_c_hat
    &'a BigUint,  // public key: the value h of pk
);

/// Algorithm 8.47: The public commitment t
pub type BigT = (
    BigUint,      // t1
    BigUint,      // t2
    BigUint,      // t3
    BigUint,      // t4_1
    BigUint,      // t4_2
    Vec<BigUint>, // vec_t_hat
);

pub trait ModuloOperations {
    /// Calculates the modular multiplicative of a BigUint: result = self * rhs % modulus.
    fn modmul(&self, rhs: &Self, modulus: &Self) -> Self;

    /// Calculates the modular division of two BigUints: result = self / divisor % modulus.
    fn moddiv(&self, divisor: &Self, modulus: &Self) -> Option<BigUint>;

    /// Calculates the modular addition of two BigUints: result = (self + other) % modulus.
    fn modadd(&self, other: &Self, modulus: &Self) -> Self;

    /// Calculates the modular subtraction of two BigUints: result = ((self + modulus) - other) % modulus.
    fn modsub(&self, other: &Self, modulus: &Self) -> Self;

    /// Calculates the modular multiplicative inverse x of an integer a such that ax ≡ 1 (mod m).
    /// Alternative formulation: a^-1 (mod m)
    fn invmod(&self, modulus: &Self) -> Option<BigUint>;
    // fn extended_gcd(a: &BigUint, b: &BigUint) -> (BigUint, BigUint, BigUint);
}

impl ModuloOperations for BigUint {
    fn modmul(&self, multiplier: &Self, modulus: &Self) -> Self {
        assert!(
            !modulus.is_zero(),
            "attempt to calculate with zero modulus!"
        );
        self.mul(multiplier) % modulus
    }

    fn moddiv(&self, divisor: &Self, modulus: &Self) -> Option<BigUint> {
        assert!(
            !modulus.is_zero(),
            "attempt to calculate with zero modulus!"
        );
        assert!(
            divisor < modulus,
            "modulus must be greater than the divisor!"
        );
        assert!(self < modulus, "modulus must be greater than the dividend!");
        let inverse_divisor = divisor.invmod(modulus);
        match inverse_divisor {
            Some(value) => Some(self.mul(&value) % modulus),
            None => None,
        }
    }

    fn modadd(&self, other: &Self, modulus: &Self) -> Self {
        assert!(
            !modulus.is_zero(),
            "attempt to calculate with zero modulus!"
        );
        self.add(other) % modulus
    }

    fn modsub(&self, other: &Self, modulus: &Self) -> Self {
        assert!(
            !modulus.is_zero(),
            "attempt to calculate with zero modulus!"
        );
        // self + modulus is done to ensure that the value is always >0
        // it's a simple shift by the whole modulus
        self.add(modulus).sub(other) % modulus
    }

    fn invmod(&self, modulus: &Self) -> Option<BigUint> {
        assert!(
            !modulus.is_zero(),
            "attempt to calculate with zero modulus!"
        );
        assert!(
            self < modulus,
            "modulus must be greater or equal to the number!"
        );
        let a = BigInt::from(self.clone());
        let b = BigInt::from(modulus.clone());

        let (g, x, _) = extended_gcd(&a, &b);
        if g != BigInt::one() {
            None
        } else {
            let result = ((x % &b) + &b) % &b;
            result.to_biguint()
        }
    }
}

fn extended_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    assert!(a < b, "a must be smaller than b!");
    if *a == BigInt::zero() {
        (b.clone(), BigInt::zero(), BigInt::one())
    } else {
        let (g, x, y) = extended_gcd(&(b % a), &a);
        (g, y - (b / a) * x.clone(), x)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        helper::Helper,
        types::{ElGamalParams, ModuloOperations, PrivateKey, PublicKey},
    };
    use alloc::vec::Vec;
    use num_bigint::BigUint;
    use num_traits::Zero;

    #[test]
    fn check_that_q_is_correctly_computed() {
        let test_params = ElGamalParams {
            p: BigUint::from(7u32),
            // and, therefore, q -> 3
            g: BigUint::from(2u32),
            h: BigUint::from(3u32),
        };

        let expected_q = BigUint::from(3u32);
        let q = test_params.q();
        assert_eq!(expected_q, q);
    }

    #[test]
    fn it_should_create_a_public_key() {
        let params = ElGamalParams {
            p: BigUint::from(7u32),
            // and, therefore, q -> 3
            g: BigUint::from(2u32),
            h: BigUint::from(3u32),
        };

        // random value must be: r ∈ Zq = r ∈ {0,1,2}
        let r = BigUint::from(2u32);

        // h = g^r mod p
        // h = 2^2 mod 7 = 4
        let h = params.g.clone().modpow(&r, &params.p);
        let pk = PublicKey {
            params: params.clone(),
            h,
        };

        assert_eq!(pk.h, BigUint::from(4u32));
        assert_eq!(pk.params.g, BigUint::from(2u32));
        assert_eq!(pk.params.p, BigUint::from(7u32));
    }

    #[test]
    fn it_should_create_a_private_key() {
        let params = ElGamalParams {
            p: BigUint::from(7u32),
            // and, therefore, q -> 3
            g: BigUint::from(2u32),
            h: BigUint::from(3u32),
        };

        // random value must be: r ∈ Zq = r ∈ {0,1,2}
        let r = BigUint::from(2u32);

        let sk = PrivateKey {
            params: params.clone(),
            // x: a random value
            x: r.clone(),
        };

        assert_eq!(sk.x, BigUint::from(2u32));
        assert_eq!(sk.params.g, BigUint::from(2u32));
        assert_eq!(sk.params.p, BigUint::from(7u32));
    }

    #[test]
    #[should_panic(expected = "there must be at least another key!")]
    fn it_should_combine_public_keys_bigunits_not_same_params() {
        let (_, _, pk) = Helper::setup_tiny_system();
        let others = Vec::new();
        pk.combine_public_keys_bigunits(&others);
    }

    #[test]
    fn it_combine_public_keys_bigunits() {
        let (_, _, pk1) = Helper::setup_tiny_system();
        let (_, _, pk2) = Helper::setup_tiny_system();
        let (_, _, pk3) = Helper::setup_tiny_system();

        let others = vec![pk2.h.clone()];
        let new_pk = pk1.clone().combine_public_keys_bigunits(&others);
        assert_eq!(new_pk.h, BigUint::from(24u32));

        let others = vec![pk2.h, pk3.h];
        let new_pk = pk1.combine_public_keys_bigunits(&others);
        assert_eq!(new_pk.h, BigUint::from(37u32));
    }

    #[test]
    #[should_panic(expected = "there must be at least another key!")]
    fn it_should_combine_public_keys_no_keys() {
        let (_, _, pk) = Helper::setup_tiny_system();
        let others = Vec::new();
        pk.combine_public_keys(&others);
    }

    #[test]
    fn it_should_combine_public_keys() {
        let (_, _, pk1) = Helper::setup_tiny_system();
        let (_, _, pk2) = Helper::setup_tiny_system();
        let (_, _, pk3) = Helper::setup_tiny_system();

        let others = vec![pk2.clone()];
        let new_pk = pk1.clone().combine_public_keys(&others);
        assert_eq!(new_pk.h, BigUint::from(24u32));

        let others = vec![pk2, pk3];
        let new_pk = pk1.combine_public_keys(&others);
        assert_eq!(new_pk.h, BigUint::from(37u32));
    }

    #[test]
    fn is_modulo_multiplication() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let ten = BigUint::from(10u32);

        let eight = six.modmul(&three, &ten);
        assert_eq!(eight, BigUint::from(8u32));
    }

    #[test]
    #[should_panic(expected = "attempt to calculate with zero modulus!")]
    fn it_should_not_use_modulus_zero_multiplication() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let zero = BigUint::zero();

        // should panic since modulus is zero
        six.modmul(&three, &zero);
    }

    #[test]
    fn is_modulo_division() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let ten = BigUint::from(10u32);

        // (6/3) mod 10
        // = 6 * 3^-1 mod 10 = 6 * 3.invmod(10) mod 10
        // = 6 * 7 mod 10 = 42 mod 10 = 2
        let two = six
            .moddiv(&three, &ten)
            .expect("cannot compute mod_inverse in mod_div!");
        assert_eq!(two, BigUint::from(2u32));
    }

    #[test]
    #[should_panic(expected = "attempt to calculate with zero modulus!")]
    fn it_should_not_use_modulus_zero_division() {
        let three = BigUint::from(3u32);
        let four = BigUint::from(4u32);
        let zero = BigUint::zero();

        // should panic since modulus is zero
        four.moddiv(&three, &zero);
    }

    #[test]
    #[should_panic(expected = "modulus must be greater than the divisor!")]
    fn it_should_panic_modulus_is_smaller_than_divisior() {
        let two = BigUint::from(2u32);
        let three = BigUint::from(3u32);
        let four = BigUint::from(4u32);

        // should panic since modulus is zero
        two.moddiv(&four, &three);
    }

    #[test]
    #[should_panic(expected = "modulus must be greater than the dividend!")]
    fn it_should_panic_modulus_is_smaller_than_dividend() {
        let two = BigUint::from(2u32);
        let three = BigUint::from(3u32);
        let four = BigUint::from(4u32);

        // should panic since modulus is zero
        four.moddiv(&two, &three);
    }

    #[test]
    fn is_modulo_addition() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let seven = BigUint::from(7u32);

        let two = six.modadd(&three, &seven);
        assert_eq!(two, BigUint::from(2u32));
    }

    #[test]
    #[should_panic(expected = "attempt to calculate with zero modulus!")]
    fn it_should_not_use_modulus_zero_addition() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let zero = BigUint::zero();

        // should panic since modulus is zero
        six.modadd(&three, &zero);
    }

    #[test]
    fn is_modulo_substration_a_greater_b() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let seven = BigUint::from(7u32);

        // 6 - 3 mod 7 = 3 mod 7 = 3
        let three = six.modsub(&three, &seven);
        assert_eq!(three, BigUint::from(3u32));
    }

    #[test]
    fn is_modulo_substration_a_smaller_b() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let seven = BigUint::from(7u32);

        // 3 - 6 mod 7 = (3+7) - 6 mod 7 = 10 - 6 mod 7 = 4 mod 7 = 4
        let four = three.modsub(&six, &seven);
        assert_eq!(four, BigUint::from(4u32));
    }

    #[test]
    #[should_panic(expected = "attempt to calculate with zero modulus!")]
    fn it_should_not_use_modulus_zero_substraction() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let zero = BigUint::zero();

        // should panic since modulus is zero
        six.modsub(&three, &zero);
    }

    #[test]
    fn it_should_compute_inverse_modulo_2_invmod_7() {
        let seven = BigUint::from(7u32);
        let two = BigUint::from(2u32);

        let expected_result = BigUint::from(4u32);

        let inverse = two.invmod(&seven).expect("cannot compute mod_inverse!");
        assert_eq!(expected_result, inverse);
    }

    #[test]
    fn it_should_compute_inverse_modulo_17_invmod_23() {
        let seventeen = BigUint::from(17u32);
        let twentythree = BigUint::from(23u32);

        let expected_result = BigUint::from(19u32);

        let inverse = seventeen
            .invmod(&twentythree)
            .expect("cannot compute mod_inverse!");
        assert_eq!(expected_result, inverse);
    }

    #[test]
    #[should_panic(expected = "modulus must be greater or equal to the number!")]
    fn it_should_panic_modulus_is_smaller_than_number_invmod() {
        let six = BigUint::from(6u32);
        let two = BigUint::from(2u32);

        // should panic since two is smaller than six
        six.invmod(&two);
    }

    #[test]
    #[should_panic(expected = "attempt to calculate with zero modulus!")]
    fn it_should_panic_modulus_is_zero() {
        let six = BigUint::from(6u32);
        let zero = BigUint::zero();

        // should panic since modulus is zero
        six.invmod(&zero);
    }
}
