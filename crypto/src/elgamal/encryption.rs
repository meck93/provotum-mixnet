use crate::elgamal::system::ModuloOperations;
use crate::elgamal::system::{Cipher, PublicKey};
use num_bigint::BigUint;

pub struct Encryption;

impl Encryption {
    /// Returns an ElGamal Encryption of a message
    /// - (c1, c2) = (g^r, h^r*g^m)
    ///
    /// ## Arguments
    ///
    /// * `m` - The message (BigUint)
    /// * `pk` - The public key used to encrypt the vote
    pub fn encrypt(m: &BigUint, r: &BigUint, pk: &PublicKey) -> Cipher {
        let g = &pk.params.g;
        let p = &pk.params.p;
        let h = &pk.h;

        // c1 = g^r
        let c1 = g.modpow(&r, &p);

        // encode the message: g^m (exponential elgamal)
        let enc_m = Encryption::encode_message(&m, &g, &p);

        // c2 = h^r*g^m
        let h_pow_r = h.modpow(&r, &p);
        let c2 = h_pow_r.modmul(&enc_m, p);

        Cipher { a: c1, b: c2 }
    }

    pub fn decrypt() {
        unimplemented!()
    }

    /// Encodes a plain-text message to be used in an explonential ElGamal scheme
    /// - encoded_message = g^m
    ///
    /// ## Arguments
    ///
    /// * `m` - The message  (BigUint)
    /// * `g` - The generator of the cyclic group Z_p
    fn encode_message(m: &BigUint, g: &BigUint, p: &BigUint) -> BigUint {
        g.modpow(m, p)
    }
}

#[cfg(test)]
mod tests {
    use super::Encryption;
    use crate::elgamal::system::{ElGamalParams, Helper};
    use num_bigint::BigUint;

    #[test]
    fn it_should_encode_message() {
        let params = ElGamalParams {
            p: BigUint::from(7 as u32),
            // and, therefore, q -> 3
            g: BigUint::from(2 as u32),
        };
        let message = BigUint::from(2 as u32);
        let encoded_message = Encryption::encode_message(&message, &params.g, &params.p);
        assert_eq!(encoded_message, BigUint::from(4 as u32));
    }

    #[test]
    fn it_should_encrypt() {
        let params = ElGamalParams {
            p: BigUint::from(7 as u32),
            // and, therefore, q -> 3
            g: BigUint::from(2 as u32),
        };

        // generate a public/private key pair
        let r = BigUint::from(2 as u32);
        let (pk, _sk) = Helper::generate_key_pair(&params, &r);

        // the value of the message: 1
        let message = BigUint::from(1 as u32);

        // a new random value for the encryption
        let r_ = BigUint::from(1 as u32);

        // encrypt the message
        let encrypted_message = Encryption::encrypt(&message, &r_, &pk);

        // verify the encryption
        let p = &pk.params.p;
        let g = &pk.params.g;

        // check that a = g^r_ = 2^1 mod 7 = 2
        assert_eq!(encrypted_message.a, BigUint::from(2 as u32));

        // check that b = h^r_*g^m = (g^r)^r_ * g^m
        // b = ((2^2)^1 mod 7 * 2^1 mod 7) mod 7
        // b = (4 mod 7 * 2 mod 7) mod 7 = 1
        assert_eq!(encrypted_message.b, BigUint::from(1 as u32));
    }
}
