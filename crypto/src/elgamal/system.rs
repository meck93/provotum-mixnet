use alloc::vec::Vec;
use core::ops::{Div, Sub};
use num_bigint::BigUint;
use num_traits::One;

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
        (self.p.clone().sub(1 as u32)).div(2 as u32)
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

pub struct Helper;

impl Helper {
    pub fn generate_key_pair(params: &ElGamalParams, r: &BigUint) -> (PublicKey, PrivateKey) {
        let sk = PrivateKey {
            params: params.clone(),
            x: r.clone(),
        };
        let h = params.g.modpow(&sk.x, &params.p);
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
        let q = params.q();

        // g is a generator (valid) if:
        // 1. g != 1
        // 2. q != q
        // 3. g^q mod p == 1
        params.g != q
            && params.g != BigUint::one()
            && (params.g.modpow(&q, &params.p) == BigUint::one())
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
            // x: a random value
            x: r.clone(),
        };

        assert_eq!(sk.x, BigUint::from(2 as u32));
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
        assert_eq!(sk.x, BigUint::from(2 as u32));

        // verify that h == g^x mod p
        assert_eq!(pk.h, sk.params.g.modpow(&sk.x, &sk.params.p));
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
