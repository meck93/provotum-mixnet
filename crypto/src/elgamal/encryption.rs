use crate::elgamal::types::{Cipher, ModuloOperations, PrivateKey, PublicKey};
use num_bigint::BigUint;
use num_traits::Zero;

pub struct ElGamal;

impl ElGamal {
    /// Returns an ElGamal Encryption of a message
    /// - (a, b) = (g^r, h^r*g^m)
    ///
    /// ## Arguments
    ///
    /// * `m`  - The message (BigUint)
    /// * `r`  - The random number used to encrypt the vote
    /// * `pk` - The public key used to encrypt the vote
    pub fn encrypt(m: &BigUint, r: &BigUint, pk: &PublicKey) -> Cipher {
        let g = &pk.params.g;
        let p = &pk.params.p;
        let h = &pk.h;

        // a = g^r
        let a = g.modpow(r, p);

        // encode the message: g^m (exponential elgamal)
        let enc_m = ElGamal::encode_message(m, g, p);

        // b = h^r*g^m
        let h_pow_r = h.modpow(r, p);
        let b = h_pow_r.modmul(&enc_m, p);

        Cipher { a, b }
    }

    /// Returns the plaintext contained in an ElGamal Encryption
    /// - mh = b * (a^x)^-1
    /// - m = log mh = log g^m
    ///
    /// ## Arguments
    ///
    /// * `cipher` - The ElGamal Encryption (a: BigUint, b: BigUint)
    /// * `sk`     - The private key used to decrypt the vote
    pub fn decrypt(cipher: &Cipher, sk: &PrivateKey) -> BigUint {
        let a = &cipher.a;
        let b = &cipher.b;

        let g = &sk.params.g;
        let p = &sk.params.p;
        let x = &sk.x;

        // a = g^r -> a^x = g^r^x
        let s = a.modpow(x, p);

        // compute multiplicative inverse of s
        let s_1 = s.invmod(p).unwrap();

        // b = g^m*h^r -> mh = b * s^-1
        let mh = b.modmul(&s_1, p);

        // brute force discrete logarithm
        ElGamal::decode_message(&mh, g, p)
    }

    /// Encodes a plain-text message to be used in an explonential ElGamal scheme
    /// Returns encoded_message = g^m.
    ///
    /// ## Arguments
    ///
    /// * `m` - The message  (BigUint)
    /// * `g` - The generator of the cyclic group Z_p (BigUint)
    /// * `p` - The group modulus p (BigUint)
    fn encode_message(m: &BigUint, g: &BigUint, p: &BigUint) -> BigUint {
        g.modpow(m, p)
    }

    /// Decodes an explonential ElGamal scheme encoded message by brute forcing the discrete lograithm.
    /// The goal is to find: encoded_message = g^m by iterating through different values for m.
    ///
    /// ## Arguments
    ///
    /// * `encoded_message` - The encoded message: g^m (BigUint)
    /// * `g` - The generator of the cyclic group Z_p (BigUint)
    /// * `p` - The group modulus p (BigUint)
    fn decode_message(encoded_message: &BigUint, g: &BigUint, p: &BigUint) -> BigUint {
        let one = 1u32;
        let mut message = BigUint::zero();

        // *encoded_message = dereference 'encoded_message' to get the value
        // brute force the discrete logarithm
        while *encoded_message != ElGamal::encode_message(&message, g, p) {
            message += one
        }
        message
    }
}

#[cfg(test)]
mod tests {
    use crate::elgamal::{encryption::ElGamal, helper::Helper};
    use num_bigint::BigUint;
    use num_traits::{One, Zero};

    #[test]
    fn it_should_encode_a_message() {
        let (params, _, _) = Helper::setup_system(b"7", b"2", b"2");
        let message = BigUint::from(2u32);
        let encoded_message = ElGamal::encode_message(&message, &params.g, &params.p);
        assert_eq!(encoded_message, BigUint::from(4u32));
    }

    #[test]
    fn it_should_decode_0() {
        let (params, _, _) = Helper::setup_system(b"7", b"2", b"2");
        let zero = BigUint::zero();
        let message = zero.clone();
        let encoded_message = ElGamal::encode_message(&message, &params.g, &params.p);
        let decoded_message = ElGamal::decode_message(&encoded_message, &params.g, &params.p);
        assert_eq!(zero, decoded_message);
    }

    #[test]
    fn it_should_decode_1() {
        let (params, _, _) = Helper::setup_system(b"7", b"2", b"2");
        let one = BigUint::one();
        let message = one.clone();
        let encoded_message = ElGamal::encode_message(&message, &params.g, &params.p);
        let decoded_message = ElGamal::decode_message(&encoded_message, &params.g, &params.p);
        assert_eq!(one, decoded_message);
    }

    #[test]
    fn it_should_decode_25() {
        let (params, _, _) = Helper::setup_system(b"23", b"2", b"9");

        // choose a message m > 1 && m < q
        let nine = BigUint::from(9u32);
        let message = nine.clone();
        let encoded_message = ElGamal::encode_message(&message, &params.g, &params.p);
        let decoded_message = ElGamal::decode_message(&encoded_message, &params.g, &params.p);
        assert_eq!(nine, decoded_message);
    }

    #[test]
    fn it_should_encrypt() {
        let (_, _, pk) = Helper::setup_system(b"7", b"2", b"2");

        // the value of the message: 1
        let message = BigUint::from(1u32);

        // a new random value for the encryption
        let r_ = BigUint::from(1u32);

        // encrypt the message
        let encrypted_message = ElGamal::encrypt(&message, &r_, &pk);

        // check that a = g^r_ = 2^1 mod 7 = 2
        assert_eq!(encrypted_message.a, BigUint::from(2u32));

        // check that b = h^r_*g^m = (g^r)^r_ * g^m
        // b = ((2^2)^1 mod 7 * 2^1 mod 7) mod 7
        // b = (4 mod 7 * 2 mod 7) mod 7 = 1
        assert_eq!(encrypted_message.b, BigUint::from(1u32));
    }

    #[test]
    fn it_should_encrypt_decrypt_2() {
        let (_, sk, pk) = Helper::setup_system(b"23", b"2", b"9");

        // the value of the message: 2
        let message = BigUint::from(2u32);

        // a new random value for the encryption
        let r_ = BigUint::from(5u32);

        // encrypt the message
        let encrypted_message = ElGamal::encrypt(&message, &r_, &pk);

        // decrypt the encrypted_message & check that the messages are equal
        let decrypted_message = ElGamal::decrypt(&encrypted_message, &sk);
        assert_eq!(decrypted_message, message);
    }
}
