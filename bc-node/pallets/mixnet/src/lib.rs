#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

#[allow(clippy::many_single_char_names)]
mod helpers;

#[allow(clippy::many_single_char_names)]
mod logic;

#[allow(clippy::many_single_char_names)]
mod shuffle;

#[allow(clippy::many_single_char_names)]
pub mod types;

mod bench;

#[cfg(test)]
mod mock;

#[cfg(test)]
#[macro_use]
mod tests;

pub mod keys;

use crate::types::{
    Ballot, Cipher, DecryptedShare, DecryptedShareProof, PublicKey as SubstratePK,
    PublicKeyShare, PublicKeyShareProof, PublicParameters, Title, Topic, TopicId, Vote,
    VoteId, VotePhase, Wrapper,
};
use crate::{
    helpers::{
        assertions::{
            ensure_not_a_voting_authority, ensure_sealer, ensure_vote_exists,
            ensure_vote_phase, ensure_voting_authority,
        },
        ballot::store_ballot,
        keys::{combine_shares, get_public_keyshare, get_public_params},
        phase::set_phase,
    },
    types::{Count, Plaintext},
};
use codec::Encode;
use crypto::types::Cipher as BigCipher;
use crypto::{
    encryption::ElGamal,
    proofs::{decryption::DecryptionProof, keygen::KeyGenerationProof},
};
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
    ensure, weights::Pays,
};
use frame_system::{
    self as system, ensure_signed,
    offchain::{AppCrypto, CreateSignedTransaction},
};
use num_bigint::BigUint;
use num_traits::One;
use sp_runtime::offchain as rt_offchain;
use sp_std::{collections::btree_map::BTreeMap, prelude::*, str, vec::Vec};

/// This is the pallet's configuration trait
pub trait Trait: system::Trait + CreateSignedTransaction<Call<Self>> {
    /// The identifier type for an offchain worker.
    type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
    /// The overarching dispatch call type.
    type Call: From<Call<Self>>;
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OffchainModule {
        pub VotingAuthorities get(fn voting_authorities) config(): Vec<T::AccountId>;
        pub Sealers get(fn sealers) config(): Vec<T::AccountId>;

        /// A vector containing the IDs of voters that have submitted their ballots
        Voters get(fn voters): Vec<T::AccountId>;

        /// Set of all voteIds
        VoteIds get(fn vote_ids): Vec<VoteId>;

        /// Maps a vote (i.e. the voteId) to a due date
        Votes get(fn votes): map hasher(blake2_128_concat) VoteId => Vote<T::AccountId>;

        /// Maps a voteId to a topic (topicId, question)
        Topics get(fn topics): map hasher(blake2_128_concat) VoteId => Vec<Topic>;

        /// Maps an voter and a vote to a ballot. Used to verify if a voter has already voted.
        Ballots get(fn ballots): double_map hasher(blake2_128_concat) VoteId, hasher(blake2_128_concat) T::AccountId => Ballot;

        /// Maps a topicId (question) to a list of Ciphers
        Ciphers get(fn ciphers): map hasher(blake2_128_concat) TopicId => Vec<Cipher>;

        /// Maps a topic to a map of results. [topic_id -> {message/vote: count}]
        Tally get(fn tally): map hasher(blake2_128_concat) TopicId => Option<BTreeMap<Plaintext, Count>>;

        /// Maps a sealer and a topic to a vector of decrypted shares.
        DecryptedShares get(fn decrypted_shares): double_map hasher(blake2_128_concat) TopicId, hasher(blake2_128_concat) T::AccountId  => Vec<Vec<u8>>;

        /// Stores the public key of a sealer together with its Schnorr proof.
        PublicKeyShares get(fn key_shares): map hasher(blake2_128_concat) VoteId => Vec<PublicKeyShare>;

        /// Stores the public key of a sealer, indexed by sealer account
        PublicKeyShareBySealer get(fn key_share_by_sealer): map hasher(blake2_128_concat) (VoteId, T::AccountId) => Option<PublicKeyShare>;

        /// Maps a vote to a public key (the vote's/system's public key) used to encrypt ballots.
        PublicKey get(fn public_key): map hasher(blake2_128_concat) VoteId => Option<SubstratePK>;
    }
}

decl_event!(
    /// Events generated by the module.
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        /// ballot submission event -> [from/who, ballot]
        BallotSubmitted(AccountId, VoteId, Ballot),

        /// public key stored event -> [from/who, public key]
        PublicKeyStored(AccountId, VoteId, SubstratePK),

        /// A voting authority set the vote's public parameters. [vote, who, params]
        VoteCreatedWithPublicParameters(VoteId, AccountId, PublicParameters),

        /// A voting authority set the question of a topic of a vote [vote, (topic_id, question)]
        VoteTopicQuestionStored(VoteId, Topic),

        /// A voting authority changed the vote phase [vote_id, newPhase]
        VotePhaseChanged(VoteId, VotePhase),

        /// A public key share was submitted. [public key with its proof]
        PublicKeyShareSubmitted(PublicKeyShare),

        /// A system public key has been created. [vote_id, public_key]
        PublicKeyCreated(VoteId, SubstratePK),

        /// A decrypted share was submitted for a vote. [paritial decryptions with its proof]
        DecryptedShareSubmitted(VoteId, AccountId),

        /// A decrypted share was submitted for a vote. [paritial decryptions with its proof]
        TopicTallied(TopicId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        // Error returned when not sure which off-chain worker function to executed
        UnknownOffchainMux,

        // Error returned when Vec<u8> cannot be parsed into BigUint
        ParseError,

        // Error returned when requester is not a voting authority
        NotAVotingAuthority,

        // Error returned when requester is a voting authority
        IsVotingAuthority,

        // Error returned when requester is not a sealer
        NotASealer,

        // Error returned when making signed transactions in off-chain worker
        NoLocalAcctForSigning,
        OffchainSignedTxError,

        // Error returned when failing to get randomness
        RandomnessGenerationError,

        // Error returned when upper bound is zero
        RandomnessUpperBoundZeroError,

        // Error returned when error occurs in gen_random_range
        RandomRangeError,

        // Error returned when permutation size is zero
        PermutationSizeZeroError,

        // Error returned when ballots are empty when trying to shuffle them
        ShuffleCiphersSizeZeroError,

        // Error returned when public key doesn't exist
        PublicKeyNotExistsError,

        // Error returned when public key share doesn't exist
        PublicKeyShareNotExistsError,

        // Error returned when the public key share proof doesn't verify
        PublicKeyShareProofError,

        // Error returned when there are less than two public key shares
        NotEnoughPublicKeyShares,

        // Error returned when inverse modulo operation fails
        InvModError,

        // Error returned when division modulo operation fails
        DivModError,

        // Error returned when vote_id does not exist yet
        VoteDoesNotExist,

        // Error returned when vote is in wrong phase
        WrongVotePhase,

        // Error returned when the decrypted share proof doesn't verify
        DecryptedShareProofError,

        // Error returned when not all sealers have submitted their decrypted shares yet
        NotEnoughDecryptedShares,

        // Error returned when a topic has already been tallied and a second attempt to tally the votes is made
        TopicHasAlreadyBeenTallied,

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        /// Set a vote phase.
        #[weight = (10_000, Pays::No)]
        fn set_vote_phase(origin, vote_id: VoteId, phase: VotePhase) -> DispatchResult {
            // only the voting_authority should be able to store the key
            let who: T::AccountId = ensure_signed(origin)?;
            helpers::assertions::ensure_voting_authority::<T>(&who)?;

            // check that the vote_id exists
            ensure!(Votes::<T>::contains_key(&vote_id), Error::<T>::VoteDoesNotExist);

            // set the new phase
            let mut vote: Vote<T::AccountId> = Votes::<T>::get(&vote_id);
            vote.phase = phase.clone();
            Votes::<T>::insert(&vote_id, &vote);
            set_phase::<T>(&who, &vote_id, phase.clone())?;

            // notify that the vote phase has been changed
            Self::deposit_event(RawEvent::VotePhaseChanged(vote_id, phase));
            Ok(())
        }

        // DEV ONLY
        #[weight = (10000, Pays::No)]
        pub fn store_public_key(origin, vote_id: VoteId, pk: SubstratePK) -> DispatchResult {
            // only the voting_authority should be able to store the key
            let who: T::AccountId = ensure_signed(origin)?;
            ensure_voting_authority::<T>(&who)?;

            // store the public key
            PublicKey::insert(vote_id.clone(), pk.clone());

            // notify that the public key has been successfully stored
            Self::deposit_event(RawEvent::PublicKeyStored(who, vote_id, pk));
            Ok(())
        }

        /// Store a public key and its proof.
        /// Can only be called from a sealer.
        #[weight = (10_000, Pays::No)]
        fn store_public_key_share(origin, vote_id: VoteId, pk_share: PublicKeyShare) -> DispatchResult {
            // only sealers can store their public key shares
            let who: T::AccountId = ensure_signed(origin)?;
            ensure_not_a_voting_authority::<T>(&who)?;
            ensure_sealer::<T>(&who)?;

            // get the public parameters
            let params: PublicParameters = get_public_params::<T>(&vote_id)?;

            // verify the public key share proof
            let sealer_id = who.encode();
            let proof: PublicKeyShareProof = pk_share.proof.clone();
            let pk: BigUint = BigUint::from_bytes_be(&pk_share.pk);
            let proof_valid = KeyGenerationProof::verify(&params.into(), &pk, &proof.into(), &sealer_id);
            ensure!(proof_valid, Error::<T>::PublicKeyShareProofError);

            // store the public key share
            let mut shares: Vec<PublicKeyShare> = PublicKeyShares::get(&vote_id);
            shares.push(pk_share.clone());
            PublicKeyShares::insert(&vote_id, shares);
            PublicKeyShareBySealer::<T>::insert((&vote_id, &who), pk_share.clone());
            debug::info!("public_key_share successfully submitted and proof verified!");

            Self::deposit_event(RawEvent::PublicKeyShareSubmitted(pk_share));
            Ok(())
        }

        /// Combine public key shares into a single public key.
        #[weight = (10_000, Pays::No)]
        fn combine_public_key_shares(origin, vote_id: VoteId) -> DispatchResult {
            // only the voting_authority should be able to combine the public key shares
            let who: T::AccountId = ensure_signed(origin)?;
            ensure_voting_authority::<T>(&who)?;
            ensure_vote_exists::<T>(&vote_id)?;

            // create the system's public key
            let pk: SubstratePK = combine_shares::<T>(&vote_id)?;
            PublicKey::insert(vote_id.clone(), pk.clone());
            debug::info!("public_key successfully generated!");

            // advance the voting phase to the next stage
            set_phase::<T>(&who, &vote_id, VotePhase::Voting)?;

            Self::deposit_event(RawEvent::PublicKeyCreated(vote_id, pk));
            Ok(())
        }

        /// Create a vote and store public crypto parameters.
        /// Can only be called from a voting authority.
        #[weight = (10000, Pays::No)]
        fn create_vote(origin, vote_id: VoteId, title: Title, params: PublicParameters, topics: Vec<Topic>) -> DispatchResult {
            let who: T::AccountId = ensure_signed(origin)?;
            ensure_voting_authority::<T>(&who)?;

            let vote = Vote::<T::AccountId> {
                voting_authority: who.clone(),
                title,
                phase: VotePhase::default(),
                params: params.clone()
            };

            // store the vote_id, vote + topic information
            let mut vote_ids: Vec<VoteId> = VoteIds::get();
            vote_ids.push(vote_id.clone());
            VoteIds::put(vote_ids);

            Votes::<T>::insert(&vote_id, vote);
            Topics::insert(&vote_id, topics);

            Self::deposit_event(RawEvent::VoteCreatedWithPublicParameters(vote_id, who, params));
            Ok(())
        }

        /// Add a question to the vote.
        /// Can only be called from a voting authority.
        #[weight = (10000, Pays::No)]
        fn store_question(origin, vote_id: VoteId, topic: Topic) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure_voting_authority::<T>(&who)?;
            ensure_vote_exists::<T>(&vote_id)?;

            let mut topics: Vec<Topic> = Topics::get(&vote_id);
            topics.push(topic.clone());
            Topics::insert(&vote_id, topics);

            Self::deposit_event(RawEvent::VoteTopicQuestionStored(vote_id, topic));

            // Return a successful DispatchResult
            Ok(())
        }

        #[weight = (10000, Pays::No)]
        pub fn cast_ballot(origin, vote_id: VoteId, ballot: Ballot) -> DispatchResult {
          let who = ensure_signed(origin)?;
          ensure_vote_exists::<T>(&vote_id)?;

          // TODO: ensure that it is a legit voter

          // store the ballot
          store_ballot::<T>(&who, &vote_id, ballot.clone());

          // notify that the ballot has been submitted and successfully stored
          Self::deposit_event(RawEvent::BallotSubmitted(who, vote_id, ballot));

          // Return a successful DispatchResult
          Ok(())
        }

        /// Store a decrypted shares.
        #[weight = (10_000, Pays::No)]
        fn submit_decrypted_shares(origin, vote_id: VoteId, topic_id: TopicId, shares: Vec<DecryptedShare>, proof: DecryptedShareProof) -> DispatchResult {
            // only sealers should be able to store their decrypted shares
            let who: T::AccountId = ensure_signed(origin)?;
            ensure_vote_exists::<T>(&vote_id)?;
            ensure_vote_phase::<T>(&vote_id, VotePhase::Tallying)?;
            ensure_sealer::<T>(&who)?;

            // get the public parameters and the public key share of the sealer
            let sealer_id = who.encode();
            let params: PublicParameters = get_public_params::<T>(&vote_id)?;
            let sealer_pk_share: PublicKeyShare = get_public_keyshare::<T>(&vote_id, &who)?;
            let sealer_pk: BigUint = BigUint::from_bytes_be(&sealer_pk_share.pk);

            // type conversion: Vec<Cipher> (Vec<Vec<u8>>) to Vec<BigCipher> (Vec<BigUint>)
            let ciphers: Vec<Cipher> = Ciphers::get(&topic_id);
            let big_ciphers: Vec<BigCipher> = Wrapper(ciphers).into();

            // type conversion: DecryptedShare (Vec<u8>) to BigUint
            let decrypted_shares: Vec<BigUint> = shares.iter().map(|s| BigUint::from_bytes_be(s)).collect::<Vec<BigUint>>();

            // verify the proof using the sealer's public key share
            let proof: DecryptionProof = proof.into();
            let is_valid: bool = DecryptionProof::verify(&params.into(), &sealer_pk, &proof.into(), big_ciphers, decrypted_shares, &sealer_id);
            ensure!(is_valid, Error::<T>::DecryptedShareProofError);

            // store the decrypted shares
            let mut stored: Vec<DecryptedShare> = DecryptedShares::<T>::get::<&TopicId, &T::AccountId>(&topic_id, &who);

            // check if the share has been already submitted. if not, store it.
            for share in shares.iter() {
                if !stored.contains(share) {
                    stored.push(share.clone());
                }
            }

            // store the decrypted shares per topic and sealer
            DecryptedShares::<T>::insert(&topic_id, &who, stored);

            // notify that the decrypted share has been:
            // submitted, the proof verified and successfully stored
            Self::deposit_event(RawEvent::DecryptedShareSubmitted(topic_id, who));
            Ok(())
        }

        /// Combine decrypted shares into a final plain text tally.
        #[weight = (10_000, Pays::No)]
        fn combine_decrypted_shares(origin, vote_id: VoteId, topic_id: TopicId, encoded: bool) -> DispatchResult {
            // only the voting_authority should be able to create the final tally
            let who: T::AccountId = ensure_signed(origin)?;
            ensure_vote_exists::<T>(&vote_id)?;
            ensure_vote_phase::<T>(&vote_id, VotePhase::Tallying)?;
            ensure_voting_authority::<T>(&who)?;

            // get the public parameters and the system public key
            let params: PublicParameters = get_public_params::<T>(&vote_id)?;
            let big_p: BigUint = BigUint::from_bytes_be(&params.p);
            let big_g: BigUint = BigUint::from_bytes_be(&params.g);

            // get all encrypted votes (ciphers) for the topic with id: topic_id
            let ciphers: Vec<Cipher> = Ciphers::get(&topic_id);

            // type conversion: Vec<Cipher> (Vec<Vec<u8>>) to Vec<BigCipher> (Vec<BigUint>)
            let big_ciphers: Vec<BigCipher> = Wrapper(ciphers).into();

            // retrieve the decrypted shares of all sealers
            let sealers: Vec<T::AccountId> = Sealers::<T>::get();
            let mut partial_decryptions: Vec<Vec<BigUint>> = Vec::with_capacity(sealers.len());

            for sealer in sealers.iter() {
                // get the partial decryptions of each sealer
                let shares: Vec<DecryptedShare> = DecryptedShares::<T>::get::<&TopicId, &T::AccountId>(&topic_id, &sealer);

                // make sure that each sealer has submitted his decrypted shares
                ensure!(shares.len() > 0, Error::<T>::NotEnoughDecryptedShares);

                // type conversion: DecryptedShare (Vec<u8>) to BigUint
                let big_shares: Vec<BigUint> = shares.iter().map(|s| BigUint::from_bytes_be(s)).collect::<Vec<BigUint>>();
                partial_decryptions.push(big_shares);
            }

            // combine all partial decryptions by all sealers
            let combined_partial_decryptions = ElGamal::combine_partial_decrypted_as(
                partial_decryptions,
                &big_p,
            );

            // retrieve the plaintext votes
            // by combining the decrypted components a with their decrypted components b
            let iterator = big_ciphers.iter().zip(combined_partial_decryptions.iter());
            let mut plaintexts = iterator
                .map(|(cipher, decrypted_a)| {
                    ElGamal::partial_decrypt_b(&cipher.b, decrypted_a, &big_p)
                })
                .collect::<Vec<BigUint>>();

            // if the votes were encoded, we need to decoded them (brute force dlog)
            if encoded {
                plaintexts = plaintexts
                .iter()
                .map(|encoded| ElGamal::decode_message(encoded, &big_g, &big_p))
                .collect::<Vec<BigUint>>();
            }

            // get the tally for the vote with topic id: topic_id
            let tally: Option<BTreeMap<Plaintext, Count>> = Tally::get::<&TopicId>(&topic_id);

            // check that topic has not been tallied yet
            ensure!(tally.is_none(), Error::<T>::TopicHasAlreadyBeenTallied);

            // count the number of votes per voting option
            // store result as a map -> key: voting option, value: count
            let one = BigUint::one();
            let mut big_results: BTreeMap<BigUint, BigUint> = BTreeMap::new();
            plaintexts.into_iter().for_each(|item| *big_results.entry(item).or_default() += &one);

            // type conversion: BTreeMap<BigUint, BigUint> to BTreeMap<Vec<u8>, Vec<u8>>
            // to be able to store the results on chain
            let mut results: BTreeMap<Plaintext, Count> = BTreeMap::new();
            for (key, value) in big_results.iter() {
                results.insert(key.to_bytes_be(), value.to_bytes_be());
            }

            // store the results on chain
            Tally::insert::<&TopicId, BTreeMap<Plaintext, Count>>(&topic_id, results);

            // notify that the decrypted shares have been successfully combined
            // and that the result has been tallied!
            Self::deposit_event(RawEvent::TopicTallied(topic_id));
            Ok(())
        }

        fn offchain_worker(block_number: T::BlockNumber) {
            debug::info!("off-chain worker: entering...");

            if sp_io::offchain::is_validator() {
                debug::info!("hi there i'm a validator");
            }

            debug::info!("off-chain worker: done...");
        }
    }
}

impl<T: Trait> rt_offchain::storage_lock::BlockNumberProvider for Module<T> {
    type BlockNumber = T::BlockNumber;
    fn current_block_number() -> Self::BlockNumber {
        <frame_system::Module<T>>::block_number()
    }
}
