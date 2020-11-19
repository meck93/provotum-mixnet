use alloc::vec::Vec;
use core::ops::{Div, Mul, Sub};
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct ElGamalParams {
    // modulus: p
    pub p: BigUint,

    // generator: g
    pub g: BigUint,
}

impl ElGamalParams {
    // q:
    // q is valid if it is prime
    pub fn q(&self) -> BigUint {
        (self.p.clone().sub(1u32)).div(2u32)
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct PublicKey {
    // system parameters (p, g)
    pub params: ElGamalParams,

    // public key: h = g^x mod p
    // - g: generator
    // - x: private key
    pub h: BigUint,
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

pub trait ModuloOperations {
    /// Calculates the modular multiplicative of a BigUint: result = self * multiplier % modulus.
    fn modmul(&self, rhs: &Self, modulus: &Self) -> Self;

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
    use crate::types::{ElGamalParams, ModuloOperations, PrivateKey, PublicKey};
    use num_bigint::BigUint;
    use num_traits::Zero;

    #[test]
    fn check_that_q_is_correctly_computed() {
        let test_params = ElGamalParams {
            p: BigUint::from(7u32),
            g: BigUint::from(2u32),
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
    fn is_modulo_multiplication() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let ten = BigUint::from(10u32);

        let eight = six.modmul(&three, &ten);
        assert_eq!(eight, BigUint::from(8u32));
    }

    #[test]
    #[should_panic(expected = "attempt to calculate with zero modulus!")]
    fn it_should_not_use_modulus_zero() {
        let three = BigUint::from(3u32);
        let six = BigUint::from(6u32);
        let zero = BigUint::zero();

        // should panic since modulus is zero
        six.modmul(&three, &zero);
    }

    #[test]
    fn it_should_compute_inverse_modulo_2_invmod_7() {
        let seven = BigUint::from(7u32);
        let two = BigUint::from(2u32);

        let expected_result = BigUint::from(4u32);

        let result = two.invmod(&seven);
        let inverse: BigUint = result.unwrap();
        assert_eq!(expected_result, inverse);
    }

    #[test]
    fn it_should_compute_inverse_modulo_17_invmod_23() {
        let seventeen = BigUint::from(17u32);
        let twentythree = BigUint::from(23u32);

        let expected_result = BigUint::from(19u32);

        let result = seventeen.invmod(&twentythree);
        let inverse: BigUint = result.unwrap();
        assert_eq!(expected_result, inverse);
    }

    #[test]
    #[should_panic(expected = "modulus must be greater or equal to the number!")]
    fn it_should_panic_modulus_is_smaller_than_number() {
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
