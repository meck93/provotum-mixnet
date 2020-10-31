use crypto::elgamal::system::Cipher;
use frame_support::codec::{Decode, Encode};
use num_bigint::BigUint;
use sp_std::vec::Vec;

// the Cipher from the crypto crate -> different types which the blockchain can handle
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Vote {
    pub a: Vec<u8>,
    pub b: Vec<u8>,
}

impl Into<Vote> for Cipher {
    fn into(self) -> Vote {
        Vote {
            a: self.a.to_bytes_be(),
            b: self.b.to_bytes_be(),
        }
    }
}

impl Into<Cipher> for Vote {
    fn into(self) -> Cipher {
        Cipher {
            a: BigUint::from_bytes_be(&self.a),
            b: BigUint::from_bytes_be(&self.b),
        }
    }
}
