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

use crate::helpers::{
    assertions::{
        ensure_not_a_voting_authority, ensure_sealer, ensure_vote_exists, ensure_voting_authority,
    },
    ballot::store_ballot,
    keys::{combine_shares, get_public_params},
    phase::set_phase,
};
use crate::types::{
    Ballot, Cipher, DecryptedShareProof, IdpPublicKey, PublicKey as SubstratePK, PublicKeyShare,
    PublicKeyShareProof, PublicParameters, Tally, Title, Topic, TopicId, Vote, VoteId, VotePhase,
};
use codec::{Decode, Encode};
use crypto::proofs::keygen::KeyGenerationProof;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    weights::Pays,
};
use frame_system::{
    self as system, ensure_signed,
    offchain::{AppCrypto, CreateSignedTransaction, SignedPayload, SigningTypes},
};
use num_bigint::BigUint;
use sp_runtime::{offchain as rt_offchain, RuntimeDebug};
use sp_std::{prelude::*, str, vec::Vec};

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

        /// Maps a vote to a list of results. [topic, yes, no, total]
        Tallies get(fn tallies): map hasher(blake2_128_concat) VoteId => Vec<Tally>;

        /// The decrypted shares -> TODO: check types
        DecryptedShares get(fn decrypted_shares): map hasher(blake2_128_concat) TopicId => Vec<Vec<u8>>;

        /// Stores the Identity provider's public key for blind signatures
        pub IdentityProviderPublicKey get(fn idp_public_key): Option<IdpPublicKey>;

        /// Stores the public key of a sealer together with its Schnorr proof.
        PublicKeyShares get(fn key_shares): map hasher(blake2_128_concat) VoteId => Vec<PublicKeyShare>;

        /// Stores the public key of a sealer, indexed by sealer account
        PublicKeyShareBySealer get(fn key_share_by_sealer): map hasher(blake2_128_concat) (VoteId, T::AccountId) => PublicKeyShare;

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

        /// The Identity Provider public key was set.
        IdentityProviderPublicKeySet(Vec<u8>, Vec<u8>),

        /// A public key share was submitted. [public key with its proof]
        PublicKeyShareSubmitted(PublicKeyShare),

        /// A system public key has been created. [vote_id, public_key]
        PublicKeyCreated(VoteId, SubstratePK),
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

        /// Allow the identity provider to store its public key.
        #[weight = (10_000, Pays::No)]
        fn store_idp_public_key(origin) -> DispatchResult {
            // only the voting_authority should be able to store the key
            let who: T::AccountId = ensure_signed(origin)?;
            ensure_voting_authority::<T>(&who)?;

            // TODO: implement logic
            // IdentityProviderPublicKey::put(public_key);
            // Self::deposit_event(RawEvent::IdentityProviderPublicKeySet());

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

        /// Register a voter.
        #[weight = (10_000, Pays::No)]
        fn register_voter(origin, signature: Vec<u8>) -> DispatchResult {
            let who: T::AccountId = ensure_signed(origin)?;

            // TODO: implement

            // ensure!(IdentityProviderPublicKey::exists(), Error::<T>::NoneValue);

            // debug::info!("IdP public key is set, verifying signature");

            // let idp_public_key: RSAPublicComponent = IdentityProviderPublicKey::get().unwrap().into();

            // let verified = verify_signature(address_bytes.to_vec(), signature.clone(), idp_public_key);

            // ensure!(verified, Error::<T>::VoterAddressNotVerified);

            // Voters::<T>::insert(&who, &signature);

            // Self::deposit_event(RawEvent::VoterRegistered(who, signature));
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

        /// Store a decrypted share.
        #[weight = (10_000, Pays::No)]
        fn submit_decrypted_share(origin, vote_id: VoteId, topic_id: TopicId, share: Vec<u8>, proof: DecryptedShareProof) -> DispatchResult {
            // only the sealers should be able to store their decrypted shares
            let who: T::AccountId = ensure_signed(origin)?;
            ensure_vote_exists::<T>(&vote_id)?;
            ensure_sealer::<T>(&who)?;

            // TODO: implement

            // Self::verify_and_store_decrypted_share(who.clone(), vote_id, subject_id.clone(), share.clone(), proof)?;

            // Self::deposit_event(RawEvent::DecryptedShareSubmitted(who, subject_id, share));
            Ok(())
        }

        /// Combine decrypted shares into a final plain text tally.
        #[weight = (10_000, Pays::No)]
        fn combine_decrypted_shares(origin, vote_id: VoteId) -> DispatchResult {
            // only the voting_authority should be able to create the final tally
            let who: T::AccountId = ensure_signed(origin)?;
            ensure_voting_authority::<T>(&who)?;
            ensure_vote_exists::<T>(&vote_id)?;

            // TODO: compute tally
            // Self::combine_decrypted_shares(&vote_id);
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
