use alloc::vec::Vec;
use core::ops::{Div, Mul, Sub};
use num_bigint::BigUint;
use num_traits::One;
use num_traits::Zero;

pub trait ModuloOperations {
    fn modmul(&self, rhs: &Self, modulus: &Self) -> Self;
}

impl ModuloOperations for BigUint {
    fn modmul(&self, rhs: &Self, modulus: &Self) -> Self {
        assert!(
            !modulus.is_zero(),
            "attempt to calculate with zero modulus!"
        );
        self.mul(rhs) % modulus
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
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
        (self.p.clone().sub(1 as u32)).div(2 as u32)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PublicKey {
    // system parameters (p, g)
    pub params: ElGamalParams,

    // public key: h = g^s mod p
    // - g: generator
    // - s: private key
    pub h: BigUint,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PrivateKey {
    // system parameters (p, g)
    pub params: ElGamalParams,

    // private key: s
    // - s: a random value (s ∈ Zq)
    pub s: BigUint,
}

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

pub struct Helper;

impl Helper {
    pub fn generate_key_pair(params: &ElGamalParams, r: &BigUint) -> (PublicKey, PrivateKey) {
        let sk = PrivateKey {
            params: params.clone(),
            s: r.clone(),
        };
        let h = params.g.clone().modpow(&sk.s, &params.p);
        let pk = PublicKey {
            params: params.clone(),
            h,
        };
        (pk, sk)
    }

    pub fn is_p_valid(p: &BigUint) -> bool {
        // check if p is prime
        unimplemented!()
    }

    pub fn is_generator(params: &ElGamalParams) -> bool {
        let p = params.p.clone();
        let g = params.g.clone();
        let q = params.q();

        // g is a generator (valid) if:
        // 1. g != 1
        // 2. q != q
        // 3. g^q mod p == 1
        g != q && g != BigUint::one() && (g.modpow(&q, &p) == BigUint::one())
    }

    pub fn get_generator_candidates(p: &BigUint) -> Vec<BigUint> {
        // 1. step: find q for the given p
        // 2. step: get all primitive roots for q
        // 3. step: check that g is a valid generator
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::{ElGamalParams, Helper, PrivateKey, PublicKey};
    use num_bigint::BigUint;

    #[cfg(test)]
    mod modulo_operations {
        use crate::elgamal::system::ModuloOperations;
        use num_bigint::BigUint;
        use num_traits::Zero;

        #[test]
        fn is_modulo_multiplication() {
            let three = BigUint::from(3 as u32);
            let six = BigUint::from(6 as u32);
            let ten = BigUint::from(10 as u32);

            let eight = six.modmul(&three, &ten);
            assert_eq!(eight, BigUint::from(8 as u32));
        }

        #[test]
        #[should_panic]
        fn it_should_not_use_modulus_zero() {
            let three = BigUint::from(3 as u32);
            let six = BigUint::from(6 as u32);
            let zero = BigUint::zero();

            // should panic since modulus is zero
            six.modmul(&three, &zero);
        }
    }

    #[test]
    fn check_that_q_is_correctly_computed() {
        let test_params = ElGamalParams {
            p: BigUint::from(7 as u32),
            g: BigUint::from(2 as u32),
        };

        let expected_q = BigUint::from(3 as u32);
        let q = test_params.q();
        assert_eq!(expected_q, q);
    }

    #[test]
    fn it_should_create_a_public_key() {
        let params = ElGamalParams {
            p: BigUint::from(7 as u32),
            // and, therefore, q -> 3
            g: BigUint::from(2 as u32),
        };

        // random value must be: r ∈ Zq = r ∈ {0,1,2}
        let r = BigUint::from(2 as u32);

        // h = g^r mod p
        // h = 2^2 mod 7 = 4
        let h = params.g.clone().modpow(&r, &params.p);
        let pk = PublicKey {
            params: params.clone(),
            h,
        };

        assert_eq!(pk.h, BigUint::from(4 as u32));
        assert_eq!(pk.params.g, BigUint::from(2 as u32));
        assert_eq!(pk.params.p, BigUint::from(7 as u32));
    }

    #[test]
    fn it_should_create_a_private_key() {
        let params = ElGamalParams {
            p: BigUint::from(7 as u32),
            // and, therefore, q -> 3
            g: BigUint::from(2 as u32),
        };

        // random value must be: r ∈ Zq = r ∈ {0,1,2}
        let r = BigUint::from(2 as u32);

        let sk = PrivateKey {
            params: params.clone(),
            // s: a random value
            s: r.clone(),
        };

        assert_eq!(sk.s, BigUint::from(2 as u32));
        assert_eq!(sk.params.g, BigUint::from(2 as u32));
        assert_eq!(sk.params.p, BigUint::from(7 as u32));
    }
    #[test]
    fn it_should_create_a_key_pair() {
        let params = ElGamalParams {
            p: BigUint::from(7 as u32),
            // and, therefore, q -> 3
            g: BigUint::from(2 as u32),
        };

        // random value must be: r ∈ Zq = r ∈ {0,1,2}
        let r = BigUint::from(2 as u32);

        // create public/private key pair
        let (pk, sk) = Helper::generate_key_pair(&params, &r);

        assert_eq!(pk.params.p, BigUint::from(7 as u32));
        assert_eq!(pk.params.g, BigUint::from(2 as u32));
        assert_eq!(pk.params.q(), BigUint::from(3 as u32));

        assert_eq!(sk.params.p, BigUint::from(7 as u32));
        assert_eq!(sk.params.g, BigUint::from(2 as u32));
        assert_eq!(sk.s, BigUint::from(2 as u32));

        // verify that h == g^s mod p
        assert_eq!(pk.h, sk.params.g.modpow(&sk.s, &sk.params.p));
    }

    #[test]
    fn check_if_generator_success() {
        let test_params = ElGamalParams {
            p: BigUint::from(7 as u32),
            g: BigUint::from(2 as u32),
        };

        let g_is_a_generator = Helper::is_generator(&test_params);
        assert!(g_is_a_generator);
    }

    #[test]
    fn check_if_generator_failure() {
        let test_params = ElGamalParams {
            p: BigUint::from(7 as u32),
            g: BigUint::from(4 as u32),
        };

        let g_is_not_a_generator = Helper::is_generator(&test_params);
        assert!(g_is_not_a_generator);
    }
}
