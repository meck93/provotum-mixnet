#![cfg_attr(not(feature = "std"), no_std)]

pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
#[macro_use]
mod tests;

use crate::types::{Ballot, PublicKey as SubstratePK, PublicParameters};
use crypto::elgamal::encryption::ElGamal;
use crypto::elgamal::types::{Cipher, ElGamalParams, PublicKey as ElGamalPK};
use frame_support::{
    codec::Encode, debug, decl_error, decl_event, decl_module, decl_storage, dispatch,
    weights::Pays,
};
use frame_system::{self as system, ensure_signed};
use num_bigint::BigUint;
use sp_std::if_std;
use sp_std::vec::Vec;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// The pallet's runtime storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MixnetModule {
        pub PublicKey get(fn public_key): SubstratePK;
        pub Ballots get(fn ballots): Vec<Ballot>;
        Voters get(fn voters): Vec<T::AccountId>;
    }
}

// Pallets use events to inform users when important changes are made.
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        /// ballot submission event -> [from/who, encrypted ballot]
        VoteSubmitted(AccountId, Ballot),

        /// public key stored event -> [from/who, public key]
        PublicKeyStored(AccountId, SubstratePK),

        /// ballots shuffled event -> [from/who]
        BallotsShuffled(AccountId),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = (10000, Pays::No)]
        pub fn store_public_key(origin, pk: SubstratePK) -> dispatch::DispatchResult {
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

        #[weight = (10000, Pays::No)]
        pub fn cast_ballot(origin, ballot: Ballot) -> dispatch::DispatchResult {
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

        #[weight = (10000, Pays::No)]
        fn trigger_shuffle(origin) -> dispatch::DispatchResult {
            // check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;
            let address_bytes = who.encode();
            debug::info!("Voter {:?} (encoded: {:?}) cast a ballot.", &who, address_bytes);

            if_std! {
                // This code is only being compiled and executed when the `std` feature is enabled.
                println!("Voter {:?} (encoded: {:?}) cast a ballot.", &who, address_bytes);
            }

            debug::info!("Shuffle requested!");

            if_std! {
                // This code is only being compiled and executed when the `std` feature is enabled.
                println!("Shuffle requested!");
            }

            // retrieve the public key
            let pk: SubstratePK = PublicKey::get();

            if_std! {
                // This code is only being compiled and executed when the `std` feature is enabled.
                println!("public key: {:?}", pk);
            }

            // shuffle all ballots
            Self::shuffle_ballots(pk);

            // notify that the ballots have been shuffled successfully
            Self::deposit_event(RawEvent::BallotsShuffled(who));

            // Return a successful DispatchResult
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
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

    fn shuffle_ballots(pk: SubstratePK) {
        // get all encrypted ballots
        let ballots: Vec<Ballot> = Ballots::get();

        // transform all ballots to crypto crate ciphers
        let mut ciphers: Vec<Cipher> = Vec::new();

        for ballot in ballots {
            ciphers.push(ballot.into());
        }

        // fake randoms
        let randoms = [
            BigUint::from(123123u32),
            BigUint::from(1200002u32),
            BigUint::from(91293u32),
        ];

        // fake permutation
        let permutations = [2, 0, 1];

        // shuffle the ballots
        let shuffled = ElGamal::shuffle(&ciphers, &permutations, &randoms, &pk.into());

        // store the shuffled ballots
        let mut shuffled_ballots: Vec<Ballot> = Vec::new();

        for cipher in shuffled {
            shuffled_ballots.push(cipher.into());
        }

        // store the shuffled ballots again
        Ballots::put(shuffled_ballots);
        debug::info!("Ballots have been shuffled!");

        if_std! {
            // This code is only being compiled and executed when the `std` feature is enabled.
            println!("Ballots have been shuffled!");
        }
    }
}
