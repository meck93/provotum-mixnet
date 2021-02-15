use super::assertions::{ensure_vote_exists, ensure_voting_authority};
use crate::types::{Vote, VoteId, VotePhase};
use crate::{Error, Trait, Votes};
use frame_support::{debug, storage::StorageMap};

/// all functions related to key generation and decrypted share operations
pub fn set_phase<T: Trait>(
    who: &T::AccountId,
    vote_id: &VoteId,
    phase: VotePhase,
) -> Result<(), Error<T>> {
    // only the voting_authority should be able to store the key
    ensure_voting_authority::<T>(who)?;
    // pase can only be changed if the vote exists
    ensure_vote_exists(vote_id)?;

    // set the new phase
    let mut vote: Vote<T::AccountId> = Votes::<T>::get(&vote_id);
    vote.phase = phase.clone();
    Votes::<T>::insert(&vote_id, &vote);
    debug::info!("vote phase updated! new phase: {:?}", phase);
    Ok(())
}
