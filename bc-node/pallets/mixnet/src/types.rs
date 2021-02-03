use codec::{Decode, Encode};
use crypto::proofs::{decryption::DecryptionProof, keygen::KeyGenerationProof};
use crypto::types::{Cipher as BigCipher, ElGamalParams, PublicKey as ElGamalPK};
use frame_system::offchain::{SignedPayload, SigningTypes};
use num_bigint::BigUint;
use num_traits::One;
use sp_runtime::RuntimeDebug;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// the BigCipher from the crypto crate.
/// different types which the blockchain can handle.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Cipher {
    pub a: Vec<u8>,
    pub b: Vec<u8>,
}

impl Into<Cipher> for BigCipher {
    fn into(self) -> Cipher {
        Cipher {
            a: self.a.to_bytes_be(),
            b: self.b.to_bytes_be(),
        }
    }
}

impl Into<BigCipher> for Cipher {
    fn into(self) -> BigCipher {
        BigCipher {
            a: BigUint::from_bytes_be(&self.a),
            b: BigUint::from_bytes_be(&self.b),
        }
    }
}

/// required to perform into() conversion for trait Vec
/// for Vec<Cipher> is not allowed, since trait Vec is not defined here
pub struct Wrapper<T>(pub Vec<T>);

impl Into<Vec<BigCipher>> for Wrapper<Cipher> {
    fn into(self) -> Vec<BigCipher> {
        self.0
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<BigCipher>>()
    }
}

impl Into<Vec<Cipher>> for Wrapper<BigCipher> {
    fn into(self) -> Vec<Cipher> {
        self.0
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<Cipher>>()
    }
}

/// the PublicKey from the crypto crate.
/// different types which the blockchain can handle.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct PublicKey {
    pub params: PublicParameters,
    pub h: Vec<u8>,
}

impl Into<PublicKey> for ElGamalPK {
    fn into(self) -> PublicKey {
        PublicKey {
            params: self.params.into(),
            h: self.h.to_bytes_be(),
        }
    }
}

impl Into<ElGamalPK> for PublicKey {
    fn into(self) -> ElGamalPK {
        ElGamalPK {
            params: self.params.into(),
            h: BigUint::from_bytes_be(&self.h),
        }
    }
}

/// the ElGamalParams from the crypto crate.
/// different types which the blockchain can handle.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct PublicParameters {
    pub p: Vec<u8>,
    // 1. public generator g
    pub g: Vec<u8>,
    // 2. public generator h
    pub h: Vec<u8>,
}

/// defines the function q = (p - 1) / 2 with return type BigUint.
/// implemented for PublicParameters (i.e. ElGamalParams from the crypto crate)
pub trait QAsBigUint {
    fn q(&self) -> BigUint;
}

impl QAsBigUint for PublicParameters {
    fn q(&self) -> BigUint {
        let p: BigUint = BigUint::from_bytes_be(&self.p);
        let q: BigUint = (p - BigUint::one()) / BigUint::from(2u32);
        q
    }
}

/// defines the function q = (p - 1) / 2 with return type Vec<u8>
/// implemented for PublicParameters (i.e. ElGamalParams from the crypto crate)
pub trait QAsVecU8 {
    fn q(&self) -> Vec<u8>;
}

impl QAsVecU8 for PublicParameters {
    fn q(&self) -> Vec<u8> {
        let p: BigUint = BigUint::from_bytes_be(&self.p);
        let q: BigUint = (p - BigUint::one()) / BigUint::from(2u32);
        q.to_bytes_be()
    }
}

impl Into<PublicParameters> for ElGamalParams {
    fn into(self) -> PublicParameters {
        PublicParameters {
            p: self.p.to_bytes_be(),
            g: self.g.to_bytes_be(),
            h: self.h.to_bytes_be(),
        }
    }
}

impl Into<ElGamalParams> for PublicParameters {
    fn into(self) -> ElGamalParams {
        ElGamalParams {
            p: BigUint::from_bytes_be(&self.p),
            g: BigUint::from_bytes_be(&self.g),
            h: BigUint::from_bytes_be(&self.h),
        }
    }
}

/// Algorithm 8.47: The s value of the ShuffleProof
pub type BigS = (
    BigUint,      // s1
    BigUint,      // s2
    BigUint,      // s3
    BigUint,      // s4
    Vec<BigUint>, // vec_s_hat
    Vec<BigUint>, // vec_s_tilde
);

/// Algorithm 8.47: The ShuffleProof
pub type ShuffleProof = (
    BigUint,      // challenge
    BigS,         // S
    Vec<BigUint>, // permutation_commitments
    Vec<BigUint>, // permutation_chain_commitments
);

pub type VoteId = Vec<u8>;
pub type Title = Vec<u8>;

// both types are strings encoded as bytes
pub type TopicId = Vec<u8>;
pub type TopicQuestion = Vec<u8>;

// topicId and question (string as Vec<u8>)
pub type Topic = (TopicId, TopicQuestion);

/// A ballot is composed of all answers of a voter
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Ballot {
    pub answers: Vec<(TopicId, Cipher)>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum VotePhase {
    KeyGeneration,
    Voting,
    Tallying,
}

// Default defines the starting value when VotePhase is created
impl Default for VotePhase {
    fn default() -> Self {
        Self::KeyGeneration
    }
}

/// A vote groups the voting authority, the title of the vote,
/// the phase the vote is currently in and the public parameters
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Vote<AccountId> {
    pub voting_authority: AccountId,
    pub title: Title,
    pub phase: VotePhase,
    pub params: PublicParameters,
}

// the public key generation proof submitted by the sealer -> this prooves knowledge of a secret key that belongs to the submitted public key
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct PublicKeyShareProof {
    pub challenge: Vec<u8>,
    pub response: Vec<u8>,
}

impl Into<PublicKeyShareProof> for KeyGenerationProof {
    fn into(self) -> PublicKeyShareProof {
        PublicKeyShareProof {
            challenge: self.challenge.to_bytes_be(),
            response: self.response.to_bytes_be(),
        }
    }
}

impl Into<KeyGenerationProof> for PublicKeyShareProof {
    fn into(self) -> KeyGenerationProof {
        KeyGenerationProof {
            challenge: BigUint::from_bytes_be(&self.challenge),
            response: BigUint::from_bytes_be(&self.response),
        }
    }
}

// the public key share submitted by each sealer to generated the system's public key
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct PublicKeyShare {
    pub pk: Vec<u8>,
    pub proof: PublicKeyShareProof,
}

pub type DecryptedShare = Vec<u8>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct DecryptedShareProof {
    pub challenge: Vec<u8>,
    pub response: Vec<u8>,
}

impl From<DecryptionProof> for DecryptedShareProof {
    fn from(source: DecryptionProof) -> Self {
        DecryptedShareProof {
            challenge: source.challenge.to_bytes_be(),
            response: source.response.to_bytes_be(),
        }
    }
}

impl From<DecryptedShareProof> for DecryptionProof {
    fn from(source: DecryptedShareProof) -> Self {
        DecryptionProof {
            challenge: BigUint::from_bytes_be(&source.challenge),
            response: BigUint::from_bytes_be(&source.response),
        }
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Tally {
    pub topic_id: TopicId,
    // store a map of result: count
    pub result: BTreeMap<Vec<u8>, Vec<u8>>,
}

/// the type to sign and send transactions.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Payload<Public> {
    ballot: Ballot,
    public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for Payload<T::Public> {
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}
