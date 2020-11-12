#![cfg_attr(not(feature = "std"), no_std)]
#![feature(unsized_locals)]

pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
#[macro_use]
mod tests;

pub mod keys;

use codec::{Decode, Encode};
use core::convert::TryInto;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
    weights::Pays,
};
use frame_system::{
    self as system, ensure_signed,
    offchain::{
        AppCrypto, CreateSignedTransaction, SendSignedTransaction, SignedPayload, Signer,
        SigningTypes,
    },
};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use rand::distributions::{Distribution, Uniform};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use sp_runtime::{offchain as rt_offchain, RuntimeDebug};
use sp_std::{collections::vec_deque::VecDeque, if_std, prelude::*, str};
use types::{Ballot, PublicKey as SubstratePK};

/// the type to sign and send transactions.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct Payload<Public> {
    number: u64,
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
    trait Store for Module<T: Trait> as Example {
        /// A vector of recently submitted numbers (at most 10).
        Numbers get(fn numbers): VecDeque<u64>;

        /// The system's public key
        PublicKey get(fn public_key): SubstratePK;

        /// A vector containing all submitted votes
        Ballots get(fn ballots): Vec<Ballot>;

        /// A vector containing the IDs of voters that have submitted their ballots
        Voters get(fn voters): Vec<T::AccountId>;
    }
}

decl_event!(
    /// Events generated by the module.
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        /// Event generated when a new number is accepted to contribute to the average.
        NewNumber(Option<AccountId>, u64),

        /// ballot submission event -> [from/who, encrypted ballot]
        VoteSubmitted(AccountId, Ballot),

        /// public key stored event -> [from/who, public key]
        PublicKeyStored(AccountId, SubstratePK),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        // Error returned when not sure which off-chain worker function to executed
        UnknownOffchainMux,

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
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = (10000, Pays::No)]
        pub fn store_public_key(origin, pk: SubstratePK) -> DispatchResult {
            // check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;
            let address_bytes = who.encode();
            debug::info!("Voter {:?} (encoded: {:?}).", &who, address_bytes);

            if_std! {
                // This code is only being compiled and executed when the `std` feature is enabled.
                println!("Voter {:?} (encoded: {:?}).", &who, address_bytes);
            }

            // store the public key
            PublicKey::put(pk.clone());

            // notify that the public key has been successfully stored
            Self::deposit_event(RawEvent::PublicKeyStored(who, pk));

            // Return a successful DispatchResult
            Ok(())
        }

        #[weight = 10000]
        pub fn submit_number_signed(origin, number: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            debug::info!("submit_number_signed: ({}, {:?})", number, who);

            Numbers::mutate(|numbers| {
                if numbers.len() == 10 {
                    let _ = numbers.pop_front();
                }
                numbers.push_back(number);
                debug::info!("Number vector: {:?}", numbers);
            });

            Self::deposit_event(RawEvent::NewNumber(Some(who), number));
            Ok(())
        }

        #[weight = (10000, Pays::No)]
        pub fn cast_ballot(origin, ballot: Ballot) -> DispatchResult {
            // check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;
            let address_bytes = who.encode();
            debug::info!("Voter {:?} (encoded: {:?}) cast a ballot.", &who, address_bytes);

            if_std! {
                // This code is only being compiled and executed when the `std` feature is enabled.
                println!("Voter {:?} (encoded: {:?}) cast a ballot.", &who, address_bytes);
            }

            // store the ballot
            Self::store_ballot(who.clone(), ballot.clone());

            // notify that the ballot has been submitted and successfully stored
            Self::deposit_event(RawEvent::VoteSubmitted(who, ballot));

            // Return a successful DispatchResult
            Ok(())
        }

        fn offchain_worker(block_number: T::BlockNumber) {
            debug::info!("off-chain worker: entering...");

            let result = Self::offchain_signed_tx(block_number);
            match result {
                Ok(_) => debug::info!("off-chain worker: successfully submitted signed_tx {:?}", block_number),
                Err(e) => debug::error!("off-chain worker - error: {:?}", e),
            }

            let number: BigUint = BigUint::parse_bytes(b"10981023801283012983912312", 10).unwrap();
            let random = Self::get_random_less_than(&number);
            match random {
                Ok(value) => debug::info!("off-chain worker: random value: {:?} less than: {:?}", value, number),
                Err(error) => debug::error!("off-chain worker - error: {:?}", error),
            }

            let lower: BigUint = BigUint::one();
            let value = Self::get_random_bigunint_range(&lower, &number);
            match value {
                Ok(val) => debug::info!("off-chain worker: random bigunit value in range. lower: {:?}, upper: {:?}, value: {:?}", lower, number, val),
                Err(error) => debug::error!("off-chain worker - error: {:?}", error),
            }

            let value = Self::get_random_range(5, 12312356);
            match value {
                Ok(val) => debug::info!("off-chain worker: random value in range. lower: {:?}, upper: {:?}, value: {:?}", 5, 12312356, val),
                Err(error) => debug::error!("off-chain worker - error: {:?}", error),
            }

            let value = Self::generate_permutation(10);
            match value {
                Ok(val) => debug::info!("off-chain worker: permutation: {:?}", val),
                Err(error) => debug::error!("off-chain worker - error: {:?}", error),
            }
            debug::info!("off-chain worker: done...");
        }
    }
}

impl<T: Trait> Module<T> {
    fn get_rng() -> ChaChaRng {
        // 32 byte array as random seed
        let seed: [u8; 32] = sp_io::offchain::random_seed();
        ChaChaRng::from_seed(seed)
    }

    /// secure random number generation using OS randomness
    fn get_random_bytes(size: usize) -> Result<Vec<u8>, Error<T>> {
        // use chacha20 to produce random vector [u8] of size: size
        let mut rng = Self::get_rng();
        let mut bytes = vec![0; size];

        // try to fill the byte array with random values
        let random_value_generation = rng.try_fill_bytes(&mut bytes);

        match random_value_generation {
            // if successful, returns the random bytes.
            Ok(_) => Ok(bytes),
            // else, that the randomness generation failed.
            Err(error) => {
                debug::error!("randomness generation error: {:?}", error);
                Err(<Error<T>>::RandomnessGenerationError)
            }
        }
    }

    // generate a random value: 0 < random < number
    fn get_random_less_than(number: &BigUint) -> Result<BigUint, Error<T>> {
        if *number <= BigUint::zero() {
            return Err(<Error<T>>::RandomnessUpperBoundZeroError);
        }

        // determine the upper bound for the random value
        let upper_bound: BigUint = number.clone() - BigUint::one();

        // the upper bound but in terms of bytes
        let size: usize = upper_bound.to_bytes_be().len();

        // fill an array of size: <size> with random bytes
        let random_bytes = Self::get_random_bytes(size);

        match random_bytes {
            Ok(bytes) => {
                // try to transform the byte array into a biguint
                let random = BigUint::from_bytes_be(&bytes);
                // ensure: random < number
                Ok(random % number)
            }
            Err(err) => Err(err),
        }
    }

    fn get_random_bigunint_range(lower: &BigUint, upper: &BigUint) -> Result<BigUint, Error<T>> {
        let mut rng = Self::get_rng();
        Self::random_bigunint_range(&mut rng, lower, upper)
    }

    fn random_bigunint_range(
        rng: &mut ChaChaRng,
        lower: &BigUint,
        upper: &BigUint,
    ) -> Result<BigUint, Error<T>> {
        if *lower < BigUint::zero() {
            return Err(<Error<T>>::RandomRangeError);
        }
        if *upper <= BigUint::zero() {
            return Err(<Error<T>>::RandomRangeError);
        }
        if *lower >= *upper {
            return Err(<Error<T>>::RandomRangeError);
        }
        let uniform = Uniform::new(lower, upper);
        let value: BigUint = uniform.sample(rng);
        Ok(value)
    }

    fn get_random_range(lower: usize, upper: usize) -> Result<usize, Error<T>> {
        let mut rng = Self::get_rng();
        Self::random_range(&mut rng, lower, upper)
    }

    fn random_range(rng: &mut ChaChaRng, lower: usize, upper: usize) -> Result<usize, Error<T>> {
        if upper == 0 {
            return Err(<Error<T>>::RandomRangeError);
        }
        if lower >= upper {
            return Err(<Error<T>>::RandomRangeError);
        }
        let uniform = Uniform::new(lower, upper);
        let value: usize = uniform.sample(rng);
        Ok(value)
    }

    fn generate_permutation(size: usize) -> Result<Vec<usize>, Error<T>> {
        if size == 0 {
            return Err(<Error<T>>::PermutationSizeZeroError);
        }

        // vector containing the range of values from 0 up to the size of the vector - 1
        let mut permutation: Vec<usize> = Vec::new();
        let mut range: Vec<usize> = (0..size).collect();
        let mut rng = Self::get_rng();

        for index in 0..size {
            // get random integer
            let random: usize = Self::random_range(&mut rng, index, size).unwrap();

            // get the element in the range at the random position
            let value = range.get(random).unwrap();

            // store the value of the element at the random position
            permutation.push(*value);

            // swap positions
            range[random] = range[index];
        }
        Ok(permutation)
    }

    fn store_ballot(from: T::AccountId, ballot: Ballot) {
        // store the encrypted ballot
        let mut ballots: Vec<Ballot> = Ballots::get();
        ballots.push(ballot.clone());
        Ballots::put(ballots);
        debug::info!("Encrypted Ballot: {:?} has been stored.", ballot);

        if_std! {
            // This code is only being compiled and executed when the `std` feature is enabled.
            println!("Encrypted Ballot: {:?} has been stored.", ballot);
        }

        // update the list of voters
        let mut voters: Vec<T::AccountId> = Voters::<T>::get();
        voters.push(from.clone());
        Voters::<T>::put(voters);
        debug::info!("Voter {:?} has been stored.", from);

        if_std! {
            // This code is only being compiled and executed when the `std` feature is enabled.
            println!("Voter {:?} has been stored.", from);
        }
    }

    fn shuffle_ballots() {
        let pk: SubstratePK = PublicKey::get();
    }

    fn offchain_signed_tx(block_number: T::BlockNumber) -> Result<(), Error<T>> {
        // We retrieve a signer and check if it is valid.
        //   ref: https://substrate.dev/rustdocs/v2.0.0/frame_system/offchain/struct.Signer.html
        let signer = Signer::<T, T::AuthorityId>::any_account();

        // Translating the current block number to number and submit it on-chain
        let number: u64 = block_number.try_into().unwrap_or(0) as u64;

        // `result` is in the type of `Option<(Account<T>, Result<(), ()>)>`. It is:
        //   - `None`: no account is available for sending transaction
        //   - `Some((account, Ok(())))`: transaction is successfully sent
        //   - `Some((account, Err(())))`: error occured when sending the transaction
        let result = signer.send_signed_transaction(|_acct|
			// This is the on-chain function
            Call::submit_number_signed(number));

        // Display error if the signed tx fails.
        if let Some((acc, res)) = result {
            if res.is_err() {
                debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
                return Err(<Error<T>>::OffchainSignedTxError);
            }
            // Transaction is sent successfully
            return Ok(());
        }

        // The case of `None`: no account is available for sending
        debug::error!("No local account available");
        Err(<Error<T>>::NoLocalAcctForSigning)
    }
}

impl<T: Trait> rt_offchain::storage_lock::BlockNumberProvider for Module<T> {
    type BlockNumber = T::BlockNumber;
    fn current_block_number() -> Self::BlockNumber {
        <frame_system::Module<T>>::block_number()
    }
}
