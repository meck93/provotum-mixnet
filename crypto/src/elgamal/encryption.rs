use crate::elgamal::types::{Cipher, ModuloOperations, PrivateKey, PublicKey};
use num_bigint::BigUint;
use num_traits::Zero;

pub struct ElGamal;

impl ElGamal {
    /// Returns an ElGamal Encryption of a message
    /// - (a, b) = (g^r, h^r * g^m)
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

        // b = h^r * g^m
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

    /// Homomorphically adds two ElGamal encryptions.
    /// Returns an ElGamal encryption.
    ///
    /// ## Arguments
    ///
    /// * `this`   - a Cipher { a, b } (ElGamal encryption)
    /// * `other`  - a Cipher { a, b } (ElGamal encryption)
    /// * `p` - The group modulus p (BigUint)    
    pub fn add(this: &Cipher, other: &Cipher, p: &BigUint) -> Cipher {
        let (a1, b1) = (this.a.clone(), this.b.clone());
        let (a2, b2) = (other.a.clone(), other.b.clone());
        Cipher {
            a: a1.modmul(&a2, p),
            b: b1.modmul(&b2, p),
        }
    }

    /// Returns an ElGamal re-encryption of a message
    /// - message:      (a, b)      = (g^r, h^r * g^m)
    /// - zero:         (a', b')    = (g^r', h^r' * g^0) = (g^r', h^r')
    /// - reencryption: (a'', b'')  = (a * a', b * b')     = (g^(r * r'), h^(r * r') * g^m)
    ///
    /// Note: The g^0 = 1 and, therefore, can be dropped. Re-encryption -> homomorphic addition with zero.
    ///
    /// ## Arguments
    ///
    /// * `cipher` - An ElGamal Encryption { a: BigUint, b: BigUint }
    /// * `r`      - The random number used to re-encrypt the vote    
    /// * `pk`     - The public key used to re-encrypt the vote
    pub fn re_encrypt(cipher: &Cipher, r: &BigUint, pk: &PublicKey) -> Cipher {
        let zero = Self::encrypt(&BigUint::zero(), &r, &pk);
        ElGamal::add(cipher, &zero, &pk.params.p)
    }
}

#[cfg(test)]
mod tests {
    use crate::elgamal::{encryption::ElGamal, helper::Helper, random::Random};
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

    #[test]
    fn it_should_add_two_zeros() {
        let (params, sk, pk) = Helper::setup_system(b"23", b"2", b"9");
        let zero = BigUint::zero();

        // encryption of zero
        let r_one = BigUint::from(7u32);
        let this = ElGamal::encrypt(&zero, &r_one, &pk);

        // encryption of zero
        let r_two = BigUint::from(5u32);
        let other = ElGamal::encrypt(&zero, &r_two, &pk);

        // add both encryptions: 0 + 0
        let addition = ElGamal::add(&this, &other, &params.p);

        // decrypt result: 0
        let decrypted_addition = ElGamal::decrypt(&addition, &sk);
        assert_eq!(decrypted_addition, zero);
    }

    #[test]
    fn it_should_add_one_and_zero() {
        let (params, sk, pk) = Helper::setup_system(b"23", b"2", b"9");
        let zero = BigUint::zero();
        let one = BigUint::one();

        // encryption of zero
        let r_one = BigUint::from(7u32);
        let this = ElGamal::encrypt(&zero, &r_one, &pk);

        // encryption of one
        let r_two = BigUint::from(5u32);
        let other = ElGamal::encrypt(&one, &r_two, &pk);

        // add both encryptions: 0 + 1
        let addition = ElGamal::add(&this, &other, &params.p);

        // decrypt result: 1
        let decrypted_addition = ElGamal::decrypt(&addition, &sk);
        assert_eq!(decrypted_addition, one);
    }

    #[test]
    fn it_should_add_two_ones() {
        let (params, sk, pk) = Helper::setup_system(b"23", b"2", b"9");
        let one = BigUint::one();
        let expected_result = BigUint::from(2u32);

        // encryption of one
        let r_one = BigUint::from(7u32);
        let this = ElGamal::encrypt(&one, &r_one, &pk);

        // encryption of one
        let r_two = BigUint::from(5u32);
        let other = ElGamal::encrypt(&one, &r_two, &pk);

        // add both encryptions: 1 + 1
        let addition = ElGamal::add(&this, &other, &params.p);

        // decrypt result: 2
        let decrypted_addition = ElGamal::decrypt(&addition, &sk);
        assert_eq!(decrypted_addition, expected_result);
    }

    #[test]
    fn it_should_add_many_and_result_equals_five() {
        let (params, sk, pk) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"2",
            b"1701411834604692317316",
        );

        let q = params.q();
        let zero = BigUint::zero();
        let one = BigUint::one();
        let expected_result = BigUint::from(5u32);

        // start with an encryption of zero
        // use a random number < q
        let r = Random::random_lt_number(&q);
        let mut base = ElGamal::encrypt(&zero, &r, &pk);

        // add five encryptions of one
        for _ in 0..5 {
            let r = Random::random_lt_number(&q);
            let encryption_of_one = ElGamal::encrypt(&one, &r, &pk);
            base = ElGamal::add(&base, &encryption_of_one, &params.p);
        }

        // add five encryptions of zero
        for _ in 0..5 {
            let r = Random::random_lt_number(&q);
            let encryption_of_zero = ElGamal::encrypt(&zero, &r, &pk);
            base = ElGamal::add(&base, &encryption_of_zero, &params.p);
        }

        // decrypt result: 5
        let decrypted_addition = ElGamal::decrypt(&base, &sk);
        assert_eq!(decrypted_addition, expected_result);
    }

    #[test]
    fn it_should_reencrypt_a_message() {
        let (params, sk, pk) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"2",
            b"1701411834604692317316",
        );

        let q = params.q();
        let zero = BigUint::zero();
        let five = BigUint::from(5u32);

        // encryption of zero
        // use a random number < q
        let r = Random::random_lt_number(&q);
        let encrypted_zero = ElGamal::encrypt(&zero, &r, &pk);

        // option 1: homomorphic addition
        // use a random number < q
        let r_ = Random::random_lt_number(&q);
        let encrypted_five = ElGamal::encrypt(&five, &r_, &pk);

        // homomorphic addition with zero: 5 + 0 = 5
        let addition = ElGamal::add(&encrypted_five, &encrypted_zero, &params.p);
        let decrypted_addition = ElGamal::decrypt(&addition, &sk);
        assert_eq!(decrypted_addition, five);

        // option two: re-encryption
        let r__ = Random::random_lt_number(&q);
        let re_encrypted_five = ElGamal::re_encrypt(&encrypted_five, &r__, &pk);
        let decrypted_re_encryption = ElGamal::decrypt(&re_encrypted_five, &sk);
        assert_eq!(decrypted_re_encryption, five);
        assert_eq!(decrypted_addition, decrypted_re_encryption);
    }
}
