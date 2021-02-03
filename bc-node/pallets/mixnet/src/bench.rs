#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::sp_api_hidden_includes_decl_storage::hidden_include::StorageDoubleMap;
use crate::types::{PublicParameters, ShuffleProof as Proof, Topic, Vote, Wrapper};
use crypto::{
    encryption::ElGamal, helper::Helper, types::Cipher as BigCipher,
    types::PublicKey as ElGamalPK,
};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use hex_literal::hex;
use num_bigint::BigUint;
use num_traits::One;
use sp_std::vec;

use crate::Module as PalletMixnet;

fn get_voting_authority<T: Trait>() -> RawOrigin<T::AccountId> {
    // use Alice as VotingAuthority
    let account_id: [u8; 32] =
        hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").into();

    let account = T::AccountId::decode(&mut &account_id[..]).unwrap();
    RawOrigin::Signed(account.into())
}

fn get_sealer_bob<T: Trait>() -> (RawOrigin<T::AccountId>, [u8; 32]) {
    let account_id: [u8; 32] =
        hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48").into();

    let account = T::AccountId::decode(&mut &account_id[..]).unwrap();
    (RawOrigin::Signed(account.into()), account_id)
}

fn get_sealer_charlie<T: Trait>() -> (RawOrigin<T::AccountId>, [u8; 32]) {
    let account_id: [u8; 32] =
        hex!("90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22").into();

    let account = T::AccountId::decode(&mut &account_id[..]).unwrap();
    (RawOrigin::Signed(account.into()), account_id)
}

fn setup_public_key<T: Trait>(
    vote_id: VoteId,
    pk: SubstratePK,
) -> Result<(), &'static str> {
    // use Alice as VotingAuthority
    let who = get_voting_authority::<T>();

    // store created public key and public parameters
    let _setup_result = PalletMixnet::<T>::store_public_key(who.into(), vote_id, pk)?;
    Ok(())
}

fn setup_vote<T: Trait>(
    params: PublicParameters,
) -> Result<(Vec<u8>, Vec<u8>), &'static str> {
    // use Alice as VotingAuthority
    let who = get_voting_authority::<T>();

    // create the vote
    let vote_id = "20201212".as_bytes().to_vec();
    let vote_title = "Popular Vote of 12.12.2020".as_bytes().to_vec();

    let topic_id = "20201212-01".as_bytes().to_vec();
    let topic_question = "Moritz for President?".as_bytes().to_vec();
    let topic: Topic = (topic_id.clone(), topic_question);
    let topics = vec![topic];

    PalletMixnet::<T>::create_vote(
        who.into(),
        vote_id.clone(),
        vote_title,
        params,
        topics,
    )?;
    Ok((vote_id, topic_id))
}

fn setup_shuffle<T: Trait>(
    size: usize,
    encoded: bool,
) -> Result<(Vec<u8>, Vec<u8>, ElGamalPK), &'static str> {
    // setup
    let (params, _, pk) = Helper::setup_lg_system();
    let (vote_id, topic_id) = setup_vote::<T>(params.into())?;
    setup_public_key::<T>(vote_id.clone(), pk.clone().into())?;

    // create messages and random values
    let q = &pk.params.q();
    let messages = PalletMixnet::<T>::get_random_biguints_less_than(q, size)?;
    let randoms = PalletMixnet::<T>::get_random_biguints_less_than(q, size)?;

    // create the voter (i.e. the transaction signer)
    let account: T::AccountId = whitelisted_caller();
    let voter = RawOrigin::Signed(account.into());

    for index in 0..messages.len() {
        let random = &randoms[index];
        let message = &messages[index];

        // transform the ballot into a from that the blockchain can handle
        // i.e. a Substrate representation { a: Vec<u8>, b: Vec<u8> }
        let cipher: Cipher;
        if encoded {
            cipher = ElGamal::encrypt_encode(message, random, &pk).into();
        } else {
            cipher = ElGamal::encrypt(message, random, &pk).into();
        }
        let answers: Vec<(TopicId, Cipher)> = vec![(topic_id.clone(), cipher)];
        let ballot: Ballot = Ballot { answers };
        PalletMixnet::<T>::cast_ballot(voter.clone().into(), vote_id.clone(), ballot)?;
    }
    Ok((topic_id, vote_id, pk))
}

fn setup_shuffle_proof<T: Trait>(
    size: usize,
    encoded: bool,
) -> Result<
    (
        Vec<u8>,
        Vec<BigCipher>,
        Vec<BigCipher>,
        Vec<BigUint>,
        Vec<usize>,
        ElGamalPK,
    ),
    &'static str,
> {
    let (topic_id, vote_id, pk) = setup_shuffle::<T>(size, encoded)?;

    // get the encrypted votes
    let e: Vec<BigCipher> = Wrapper(PalletMixnet::<T>::ciphers(&topic_id)).into();
    ensure!(e.len() == size, "# of votes on chain is not correct");

    // shuffle the votes
    let result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    let s: (Vec<BigCipher>, Vec<BigUint>, Vec<usize>) = result.unwrap();
    let e_hat = s.0; // the shuffled votes
    let r = s.1; // the re-encryption randoms
    let permutation = s.2;
    Ok((vote_id, e, e_hat, r, permutation, pk))
}

benchmarks! {
    _{ }

    store_public_key {
        let (_, _, pk) = Helper::setup_lg_system();
        let who = get_voting_authority::<T>();
        let vote_id = "20201212".as_bytes().to_vec();
    }: {
        // store created public key and public parameters
        let _result = PalletMixnet::<T>::store_public_key(who.into(), vote_id.clone(), pk.clone().into());
    }
    verify {
        // fetch the public key from the chain
        let pk_from_chain: ElGamalPK = PalletMixnet::<T>::public_key(vote_id).unwrap().into();
        ensure!(pk_from_chain == pk, "fail pk_from_chain != pk");
    }

    store_public_key_share {
        let (params, sk, pk) = Helper::setup_lg_system();
        let (bob, bob_id) = get_sealer_bob::<T>();
        let (vote_id, _) = setup_vote::<T>(params.clone().into())?;

        // create public key share + proof
        let q = &params.clone().q();
        let random = PalletMixnet::<T>::get_random_biguint_less_than(q)?;
        let proof = KeyGenerationProof::generate(&params, &sk.x, &pk.h, &random, &bob_id);
        let pk_share = PublicKeyShare {
            proof: proof.clone().into(),
            pk: pk.h.to_bytes_be(),
        };
    }: {
        // store created public key and public parameters
        let _result = PalletMixnet::<T>::store_public_key_share(bob.into(), vote_id.clone(), pk_share.clone().into());
    }

    combine_public_key_shares {
        let (params, sk, pk) = Helper::setup_lg_system();
        let voting_authority = get_voting_authority::<T>();
        let (vote_id, _) = setup_vote::<T>(params.clone().into())?;
        let q = &params.clone().q();

        // create public key share + proof for bob
        let (bob, bob_id) = get_sealer_bob::<T>();
        let random = PalletMixnet::<T>::get_random_biguint_less_than(q)?;
        let proof_bob = KeyGenerationProof::generate(&params, &sk.x, &pk.h, &random, &bob_id);
        let pk_share_bob = PublicKeyShare {
            proof: proof_bob.clone().into(),
            pk: pk.h.to_bytes_be(),
        };
        // store created public key and public parameters
        let result_ = PalletMixnet::<T>::store_public_key_share(bob.into(), vote_id.clone(), pk_share_bob.clone().into());

        // create public key share + proof for charlie
        let (charlie, charlie_id) = get_sealer_charlie::<T>();
        let random = PalletMixnet::<T>::get_random_biguint_less_than(q)?;
        let proof_charlie = KeyGenerationProof::generate(&params, &sk.x, &pk.h, &random, &charlie_id);
        let pk_share_charlie = PublicKeyShare {
            proof: proof_charlie.clone().into(),
            pk: pk.h.to_bytes_be(),
        };
        // store created public key and public parameters
        let result_ = PalletMixnet::<T>::store_public_key_share(charlie.into(), vote_id.clone(), pk_share_charlie.clone().into());
    }: {
        // combine the public key shares
        let _result = PalletMixnet::<T>::combine_public_key_shares(voting_authority.into(), vote_id.clone());
    }

    create_vote {
        // use Alice as VotingAuthority
        let who = get_voting_authority::<T>();

        // create the vote
        let vote_id = "20201212".as_bytes().to_vec();
        let vote_title = "Popular Vote of 12.12.2020".as_bytes().to_vec();

        let topic_id = "20201212-01".as_bytes().to_vec();
        let topic_question = "Moritz for President?".as_bytes().to_vec();
        let topic: Topic = (topic_id.clone(), topic_question);
        let topics = vec![topic];

        // store created public key
        let (params, _, pk) = Helper::setup_lg_system();
        PalletMixnet::<T>::store_public_key(who.clone().into(), vote_id.clone(), pk.into())?;

    }: {
        let _result = PalletMixnet::<T>::create_vote(who.into(), vote_id.clone(), vote_title.clone(), params.into(), topics)?;
    } verify {
        let vote: Vote<T::AccountId> = PalletMixnet::<T>::votes(vote_id);
        ensure!(vote_title == vote.title, "title are not the same!");
    }

    store_question {
        let (params, _, pk) = Helper::setup_lg_system();
        let (vote_id, topic_id) = setup_vote::<T>(params.into())?;

        // use Alice as VotingAuthority
        let who = get_voting_authority::<T>();

        // create another topic
        let topic_id_2 = "20201212-02".as_bytes().to_vec();
        let topic_question = "Moritz for King?".as_bytes().to_vec();
        let topic: Topic = (topic_id_2.clone(), topic_question.clone());
    }: {
        let _result = PalletMixnet::<T>::store_question(who.into(), vote_id.clone(), topic);
    } verify {
        let topic_: Vec<Topic> = PalletMixnet::<T>::topics(vote_id);
        ensure!(topic_id == topic_[0].0, "topic ids are not the same!");
        ensure!(topic_id_2 == topic_[1].0, "topic ids are not the same!");
        ensure!(topic_question == topic_[1].1, "topic questions are not the same!");
    }

    cast_ballot {
        // setup
        let (params, _, pk) = Helper::setup_lg_system();
        let (vote_id, topic_id) = setup_vote::<T>(params.into())?;

        // create messages and random values
        let q = &pk.params.q();
        let message = BigUint::one();
        let random = PalletMixnet::<T>::get_random_biguint_less_than(q)?;

        // create the voter (i.e. the transaction signer)
        let account: T::AccountId = whitelisted_caller();
        let voter = RawOrigin::Signed(account.clone().into());

        // transform the ballot into a from that the blockchain can handle
        // i.e. a Substrate representation { a: Vec<u8>, b: Vec<u8> }
        let cipher: Cipher = ElGamal::encrypt_encode(&message, &random, &pk).into();
        let answers: Vec<(TopicId, Cipher)> = vec![(topic_id, cipher)];
        let ballot: Ballot = Ballot { answers };
    }: {
        let _result = PalletMixnet::<T>::cast_ballot(voter.clone().into(), vote_id.clone(), ballot.clone())?;
    } verify {
        let ballot_: Ballot = Ballots::<T>::get(vote_id, account);
        ensure!(ballot == ballot_, "ballots are not the same!");
    }

    verify_public_key_share_proof {
        // setup
        let (params, sk, pk) = Helper::setup_lg_system();
        let q = params.q();
        let (vote_id, topic_id) = setup_vote::<T>(params.clone().into())?;

        // create the sealer
        let sealer_id: [u8; 32] =
        hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48").into();
        let sealer_account_id = T::AccountId::decode(&mut &sealer_id[..]).unwrap();
        let sealer = RawOrigin::Signed(sealer_account_id.into());

        // create public key share + proof
        let r = PalletMixnet::<T>::get_random_biguint_less_than(&q)?;
        let proof = KeyGenerationProof::generate(&params, &sk.x, &pk.h, &r, &sealer_id);
        let pk_share = PublicKeyShare {
            proof: proof.clone().into(),
            pk: pk.h.to_bytes_be(),
        };

    }: {
        PalletMixnet::<T>::store_public_key_share(sealer.into(), vote_id, pk_share.clone())?;
    }

    shuffle_ciphers_3 {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(3, false)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_ciphers_10 {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(10, false)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_ciphers_30 {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(30, false)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_ciphers_100 {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(100, false)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_ciphers_1000 {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(1000, false)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_ciphers_3_encoded {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(3, true)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_ciphers_10_encoded {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(10, true)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_ciphers_30_encoded {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(30, true)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_ciphers_100_encoded {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(100, true)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_ciphers_1000_encoded {
        let (topic_id, vote_id, _) = setup_shuffle::<T>(1000, true)?;
    }: {
        let _result = PalletMixnet::<T>::shuffle_ciphers(&vote_id, &topic_id);
    }

    shuffle_proof_3 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(3, false)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    shuffle_proof_10 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(10, false)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    shuffle_proof_30 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(30, false)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    shuffle_proof_100 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(100, false)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    shuffle_proof_1000 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(1000, false)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    shuffle_proof_3_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(3, true)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    shuffle_proof_10_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(10, true)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    shuffle_proof_30_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(30, true)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    shuffle_proof_100_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(100, true)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    shuffle_proof_1000_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(1000, true)?;
    }: {
        let _result = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e, e_hat, r, &permutation, &pk);
    }

    verify_shuffle_proof_3 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(3, false)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    verify_shuffle_proof_10 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(10, false)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    verify_shuffle_proof_30 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(30, false)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    verify_shuffle_proof_100 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(100, false)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    verify_shuffle_proof_1000 {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(1000, false)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    verify_shuffle_proof_3_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(3, true)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    verify_shuffle_proof_10_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(10, true)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    verify_shuffle_proof_30_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(30, true)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    verify_shuffle_proof_100_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(100, true)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    verify_shuffle_proof_1000_encoded {
        let (vote_id, e, e_hat, r, permutation, pk) = setup_shuffle_proof::<T>(1000, true)?;
        let proof: Proof = PalletMixnet::<T>::generate_shuffle_proof(&vote_id, e.clone(), e_hat.clone(), r, &permutation, &pk)?;
    }: {
        let success = PalletMixnet::<T>::verify_shuffle_proof(&vote_id, proof, e, e_hat, &pk)?;
        ensure!(success, "proof did not verify!");
    }

    // TODO: add benchmarks for
    // 1. submit_decrypted_share
    // 2. combine_decrypted_shares
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExternalityBuilder, TestRuntime};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        let (mut t, _, _) = ExternalityBuilder::build();
        t.execute_with(|| {
            assert_ok!(test_benchmark_store_public_key::<TestRuntime>());
            assert_ok!(test_benchmark_store_public_key_share::<TestRuntime>());
            assert_ok!(test_benchmark_combine_public_key_shares::<TestRuntime>());
            assert_ok!(test_benchmark_store_question::<TestRuntime>());
            assert_ok!(test_benchmark_create_vote::<TestRuntime>());
            assert_ok!(test_benchmark_cast_ballot::<TestRuntime>());
            assert_ok!(test_benchmark_shuffle_ciphers_3::<TestRuntime>());
            assert_ok!(test_benchmark_shuffle_ciphers_3_encoded::<TestRuntime>());
            assert_ok!(test_benchmark_shuffle_proof_3::<TestRuntime>());
            assert_ok!(test_benchmark_shuffle_proof_3_encoded::<TestRuntime>());
            assert_ok!(test_benchmark_verify_shuffle_proof_3::<TestRuntime>());
            assert_ok!(test_benchmark_verify_shuffle_proof_3_encoded::<TestRuntime>());
        });
    }
}
