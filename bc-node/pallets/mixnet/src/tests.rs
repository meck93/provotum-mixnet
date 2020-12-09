use crate::mock::*;
use crate::types::{Ballot, Cipher, PublicKey as SubstratePK, ShuffleProof as Proof, Wrapper};
use crate::*;
use codec::Decode;
use crypto::{
    encryption::ElGamal,
    helper::Helper,
    types::{Cipher as BigCipher, PublicKey as ElGamalPK},
};
use frame_support::assert_ok;
use frame_system as system;
use hex_literal::hex;
use num_traits::Zero;
use sp_std::vec;

fn setup_public_key(pk: SubstratePK) {
    // create the submitter
    let account: <TestRuntime as system::Trait>::AccountId = Default::default();
    let who = Origin::signed(account);

    // store created public key and public parameters
    let public_key_storage = OffchainModule::store_public_key(who, pk);
    assert_ok!(public_key_storage);
}

fn setup_vote(params: PublicParameters) -> (Vec<u8>, Vec<u8>) {
    // create the submitter (i.e. the voting_authority)
    // use Alice as VotingAuthority
    let account_id: [u8; 32] =
        hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").into();
    let account = <TestRuntime as system::Trait>::AccountId::decode(&mut &account_id[..]).unwrap();
    let who = Origin::signed(account);

    // create the vote
    let vote_id = "20201212".as_bytes().to_vec();
    let vote_title = "Popular Vote of 12.12.2020".as_bytes().to_vec();

    let topic_id = "20201212-01".as_bytes().to_vec();
    let topic_question = "Moritz for President?".as_bytes().to_vec();
    let topic: Topic = (topic_id.clone(), topic_question);
    let topics = vec![topic];

    let vote_created =
        OffchainModule::create_vote(who, vote_id.clone(), vote_title, params, topics);
    assert_ok!(vote_created);
    (vote_id, topic_id)
}

#[test]
fn test_setup_public_key_work() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let (_, _, pk) = Helper::setup_sm_system();
        setup_public_key(pk.into());
    });
}

#[test]
fn test_initialization_works() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // Fetch Voting Authority
        let voting_authorities = OffchainModule::voting_authorities();
        assert!(voting_authorities.len() == 1);

        // Fetch Sealers
        let sealers = OffchainModule::sealers();
        assert!(sealers.len() == 3);
    });
}

#[test]
fn test_store_public_key() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // create the submitter (i.e. the public key submitter)
        let account: <TestRuntime as system::Trait>::AccountId = Default::default();
        let who = Origin::signed(account);

        // create the public key
        let (_, _, pk) = Helper::setup_md_system();

        // store created public key and public parameters
        let public_key_storage = OffchainModule::store_public_key(who, pk.clone().into());
        assert_ok!(public_key_storage);

        // fetch the public key from the chain
        let pk_from_chain: ElGamalPK = OffchainModule::public_key().unwrap().into();
        assert_eq!(pk_from_chain, pk);
    });
}

#[test]
fn test_fetch_public_key_does_not_exist() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // fetch the public key from the chain which doesn't exist
        let pk_from_chain: Option<SubstratePK> = OffchainModule::public_key();
        assert_eq!(pk_from_chain, None);
    });
}
#[test]
fn test_create_vote_not_a_voting_authority() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // TODO
    });
}
#[test]
fn test_create_vote_works() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let vote_id = "20201212".as_bytes().to_vec();
        let topic_id = "20201212-01".as_bytes().to_vec();
        let (params, _, _) = Helper::setup_sm_system();
        let (vote_id_, topic_id_) = setup_vote(params.into());
        assert_eq!(vote_id, vote_id_);
        assert_eq!(topic_id, topic_id_);
    });
}

#[test]
fn test_cast_ballot_no_vote_exists() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // TODO
    });
}

#[test]
fn test_cast_ballot_works() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // Setup Public Key
        let (params, _, pk) = Helper::setup_sm_system();
        let q = &params.q();
        setup_public_key(pk.clone().into());

        // Setup Vote
        let (vote_id, topic_id) = setup_vote(params.into());

        // Create the voter
        let acct: <TestRuntime as system::Trait>::AccountId = Default::default();
        println!("acct: {:?}", acct);

        // submit the value 32
        let num: u64 = 32;
        let big: BigUint = BigUint::from(num);
        let r = OffchainModule::get_random_biguint_less_than(q).unwrap();
        let cipher: Cipher = ElGamal::encrypt(&big, &r, &pk).into();
        let answers = vec![(topic_id.clone(), cipher.clone())];
        let ballot: Ballot = Ballot { answers };

        // Test
        // call cast_ballot
        assert_ok!(OffchainModule::cast_ballot(
            Origin::signed(acct),
            vote_id.clone(),
            ballot.clone()
        ));
        let ballot_from_chain = OffchainModule::ballots(vote_id.clone(), acct);
        // A encrypted ballot is inserted to Ballots vec
        assert_eq!(ballot_from_chain, ballot.clone());

        // Cipher is inserted into Ciphers
        assert_eq!(
            OffchainModule::ciphers(topic_id.clone()),
            vec![cipher.clone()]
        );

        // An event is emitted
        assert!(System::events().iter().any(|er| er.event
            == TestEvent::pallet_mixnet(RawEvent::BallotSubmitted(
                acct,
                vote_id.clone(),
                ballot.clone()
            ))));

        // Insert another ballot
        let ballot2 = ballot.clone();
        assert_ok!(OffchainModule::cast_ballot(
            Origin::signed(acct),
            vote_id.clone(),
            ballot.clone()
        ));
        // A encrypted ballot is inserted to Ballots vec
        assert_eq!(OffchainModule::ballots(vote_id, acct), ballot2.clone());

        // Cipher is inserted into Ciphers
        assert_eq!(
            OffchainModule::ciphers(topic_id.clone()),
            vec![cipher.clone(), cipher]
        );
    });
}

#[test]
fn test_offchain_signed_tx() {
    let (mut t, pool_state, _) = ExternalityBuilder::build();

    t.execute_with(|| {
        // Setup
        let (params, _, pk) = Helper::setup_sm_system();
        setup_public_key(pk.clone().into());
        let q = &params.q();

        // Setup Vote
        let (vote_id, topic_id) = setup_vote(params.into());

        let num: u64 = 32;
        let big: BigUint = BigUint::from(num);
        let r = OffchainModule::get_random_biguint_less_than(q).unwrap();
        let cipher: Cipher = ElGamal::encrypt(&big, &r, &pk).into();
        let answers: Vec<(TopicId, Cipher)> = vec![(topic_id.clone(), cipher)];
        let ballot: Ballot = Ballot { answers };

        // Test
        OffchainModule::offchain_signed_tx(num, vote_id.clone(), topic_id).unwrap();

        // Verify
        let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let tx = TestExtrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature.unwrap().0, 0);
        assert_eq!(tx.call, Call::cast_ballot(vote_id, ballot.clone()));
    });
}

#[test]
fn test_get_random_bytes() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let size: usize = 32;
        let random = OffchainModule::get_random_bytes(size).unwrap();
        assert_eq!(random.len(), size);
    });
}

#[test]
fn test_get_random_number_less_than() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let upper_bound: BigUint = BigUint::parse_bytes(b"10981023801283012983912312", 10).unwrap();
        let random = OffchainModule::get_random_biguint_less_than(&upper_bound).unwrap();
        assert!(random < upper_bound);
    });
}

#[test]
fn test_get_random_number_less_than_should_panic_number_is_zero() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let upper_bound: BigUint = BigUint::parse_bytes(b"0", 10).unwrap();
        OffchainModule::get_random_biguint_less_than(&upper_bound).expect_err(
            "The returned value should be: '<Error<T>>::RandomnessUpperBoundZeroError'",
        );
    });
}

#[test]
fn test_get_random_numbers_less_than() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let upper_bound: BigUint = BigUint::parse_bytes(b"10981023801283012983912312", 10).unwrap();
        let randoms: Vec<BigUint> =
            OffchainModule::get_random_biguints_less_than(&upper_bound, 10).unwrap();
        assert_eq!(randoms.len(), 10);
        let zero = BigUint::zero();
        for random in randoms.iter() {
            assert!(random < &upper_bound);
            assert!(random > &zero);
        }
    });
}

#[test]
fn test_get_random_numbers_less_than_should_panic_number_is_zero() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let upper_bound: BigUint = BigUint::parse_bytes(b"10981023801283012983912312", 10).unwrap();
        OffchainModule::get_random_biguints_less_than(&upper_bound, 0).expect_err(
            "The returned value should be: '<Error<T>>::RandomnessUpperBoundZeroError'",
        );
    });
}

#[test]
fn test_get_random_bigunint_range() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let lower: BigUint = BigUint::parse_bytes(b"0", 10).unwrap();
        let upper: BigUint = BigUint::parse_bytes(b"10981023801283012983912312", 10).unwrap();
        let value = OffchainModule::get_random_bigunint_range(&lower, &upper).unwrap();

        assert!(value < upper);
        assert!(lower < value);
    });
}

#[test]
fn test_get_random_bigunint_range_upper_is_zero() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let lower: BigUint = BigUint::parse_bytes(b"0", 10).unwrap();
        let upper: BigUint = BigUint::parse_bytes(b"0", 10).unwrap();
        OffchainModule::get_random_bigunint_range(&lower, &upper)
            .expect_err("The returned value should be: '<Error<T>>::RandomRangeError'");
    });
}

#[test]
fn test_get_random_bigunint_range_upper_is_not_larger_than_lower() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let lower: BigUint = BigUint::parse_bytes(b"5", 10).unwrap();
        let upper: BigUint = BigUint::parse_bytes(b"5", 10).unwrap();
        OffchainModule::get_random_bigunint_range(&lower, &upper)
            .expect_err("The returned value should be: '<Error<T>>::RandomRangeError'");
    });
}

#[test]
fn test_get_random_range() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let lower: usize = 0;
        let upper: usize = 100;
        let value = OffchainModule::get_random_range(lower, upper).unwrap();

        assert!(value < upper);
        assert!(lower < value);
    });
}

#[test]
fn test_get_random_range_upper_is_zero_error() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let lower: usize = 0;
        let upper: usize = 0;
        OffchainModule::get_random_range(lower, upper)
            .expect_err("The returned value should be: '<Error<T>>::RandomRangeError'");
    });
}

#[test]
fn test_get_random_range_upper_is_not_larger_than_lower_error() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let lower: usize = 5;
        let upper: usize = 5;
        OffchainModule::get_random_range(lower, upper)
            .expect_err("The returned value should be: '<Error<T>>::RandomRangeError'");
    });
}

#[test]
fn test_generate_permutation_size_zero_error() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let size = 0;
        OffchainModule::generate_permutation(size)
            .expect_err("The returned value should be: '<Error<T>>::PermutationSizeZeroError'");
    });
}

#[test]
fn test_should_generate_a_permutation_size_three() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let size = 3;
        let permutation = OffchainModule::generate_permutation(size).unwrap();

        // check that the permutation has the expected size
        assert!(permutation.len() == (size as usize));

        // check that 0, 1, 2 occur at least once each
        assert!(permutation.iter().any(|&value| value == 0));
        assert!(permutation.iter().any(|&value| value == 1));
        assert!(permutation.iter().any(|&value| value == 2));
    });
}

#[test]
fn test_fetch_ballots_size_zero() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let topic_id = "Moritz for President?".as_bytes().to_vec();
        // Read pallet storage (i.e. the submitted ballots)
        // and assert an expected result.
        let ciphers_from_chain: Vec<Cipher> = OffchainModule::ciphers(topic_id);
        assert!(ciphers_from_chain.len() == 0);
    });
}

#[test]
fn store_small_dummy_vote_works() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // Setup Vote
        let (params, sk, pk) = Helper::setup_sm_system();
        let (vote_id, topic_id) = setup_vote(params.into());

        let message = BigUint::from(1u32);
        let random = BigUint::from(7u32);

        // encrypt the message -> encrypted message
        // cipher = the crypto crate version of a ballot { a: BigUint, b: BigUint }
        let big_cipher: BigCipher = ElGamal::encrypt(&message, &random, &pk);

        // transform the ballot into a from that the blockchain can handle
        // i.e. a Substrate representation { a: Vec<u8>, b: Vec<u8> }
        let cipher: Cipher = big_cipher.clone().into();
        let answers: Vec<(TopicId, Cipher)> = vec![(topic_id.clone(), cipher.clone())];
        let ballot: Ballot = Ballot { answers };

        // create the voter (i.e. the transaction signer)
        let account: <TestRuntime as system::Trait>::AccountId = Default::default();
        let voter = Origin::signed(account);

        let vote_submission_result = OffchainModule::cast_ballot(voter, vote_id, ballot);
        assert_ok!(vote_submission_result);

        // fetch the submitted ballot
        let ciphers_from_chain: Vec<Cipher> = OffchainModule::ciphers(topic_id);
        assert!(ciphers_from_chain.len() > 0);

        let cipher_from_chain: Cipher = ciphers_from_chain[0].clone();
        assert_eq!(cipher, cipher_from_chain);

        // transform the Ballot -> BigCipher
        let big_cipher_from_chain: BigCipher = cipher_from_chain.into();
        assert_eq!(big_cipher, big_cipher_from_chain);

        let decrypted_vote = ElGamal::decrypt(&big_cipher_from_chain, &sk);
        assert_eq!(message, decrypted_vote);
    });
}

#[test]
fn store_real_size_vote_works() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // Setup
        let (params, sk, pk) = Helper::setup_md_system();
        let (vote_id, topic_id) = setup_vote(params.into());

        // encrypt the message -> encrypted message
        // cipher = the crypto crate version of a ballot { a: BigUint, b: BigUint }
        let message = BigUint::from(1u32);
        let random = BigUint::parse_bytes(b"170141183460469231731687303715884", 10).unwrap();
        let big_cipher: BigCipher = ElGamal::encrypt(&message, &random, &pk);

        // transform the ballot into a from that the blockchain can handle
        // i.e. a Substrate representation { a: Vec<u8>, b: Vec<u8> }
        let cipher: Cipher = big_cipher.clone().into();
        let answers: Vec<(TopicId, Cipher)> = vec![(topic_id.clone(), cipher.clone())];
        let ballot: Ballot = Ballot { answers };

        // create the voter (i.e. the transaction signer)
        let account: <TestRuntime as system::Trait>::AccountId = Default::default();
        let voter = Origin::signed(account);

        let vote_submission_result = OffchainModule::cast_ballot(voter, vote_id, ballot);
        assert_ok!(vote_submission_result);

        // fetch the submitted ballot
        let ciphers_from_chain: Vec<Cipher> = OffchainModule::ciphers(topic_id);
        assert!(ciphers_from_chain.len() > 0);

        let cipher_from_chain: Cipher = ciphers_from_chain[0].clone();
        assert_eq!(cipher, cipher_from_chain);

        // transform the Ballot -> BigCipher
        let big_cipher_from_chain: BigCipher = cipher_from_chain.into();
        assert_eq!(big_cipher, big_cipher_from_chain);

        let decrypted_vote = ElGamal::decrypt(&big_cipher_from_chain, &sk);
        assert_eq!(message, decrypted_vote);
    });
}

#[test]
fn test_shuffle_ballots() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // Setup
        let (params, sk, pk) = Helper::setup_md_system();
        let (vote_id, topic_id) = setup_vote(params.into());

        // create the submitter (i.e. the public key submitter)
        let account: <TestRuntime as system::Trait>::AccountId = Default::default();
        let who = Origin::signed(account);

        // create the public key
        let messages = [
            BigUint::from(5u32),
            BigUint::from(10u32),
            BigUint::from(15u32),
        ];

        // store created public key and public parameters
        let public_key_storage = OffchainModule::store_public_key(who, pk.clone().into());
        assert_ok!(public_key_storage);

        // encrypt the message -> encrypted message
        // cipher = the crypto crate version of a ballot { a: BigUint, b: BigUint }
        let randoms = [
            b"170141183460469231731687303715884",
            b"170141183460469231731687303700084",
            b"170141183400069231731687303700084",
        ];

        // create the voter (i.e. the transaction signer)
        let account: <TestRuntime as system::Trait>::AccountId = Default::default();
        let voter = Origin::signed(account);

        for index in 0..3 {
            let random = BigUint::parse_bytes(randoms[index], 10).unwrap();

            // transform the ballot into a from that the blockchain can handle
            // i.e. a Substrate representation { a: Vec<u8>, b: Vec<u8> }
            let cipher: Cipher = ElGamal::encrypt(&messages[index], &random, &pk).into();
            let answers: Vec<(TopicId, Cipher)> = vec![(topic_id.clone(), cipher)];
            let ballot: Ballot = Ballot { answers };

            let vote_submission_result =
                OffchainModule::cast_ballot(voter.clone(), vote_id.clone(), ballot);
            assert_ok!(vote_submission_result);
        }

        // shuffle the votes
        let shuffle_result = OffchainModule::shuffle_ciphers(&topic_id);
        let shuffled_big_ciphers: Vec<BigCipher> = shuffle_result.unwrap().0;
        assert!(shuffled_big_ciphers.len() == 3);

        // type conversion: BigCipher (BigUint) to Ballot (Vec<u8>)
        let shuffled_ciphers: Vec<Cipher> = Wrapper(shuffled_big_ciphers).into();

        // transform each ballot into a cipher, decrypt it and finally collect the list of biguints
        let decrypted_votes = shuffled_ciphers
            .iter()
            .map(|b| ElGamal::decrypt(&(b.clone().into()), &sk))
            .collect::<Vec<BigUint>>();

        // check that at least one value is 5, 10, 15
        assert!(decrypted_votes
            .iter()
            .any(|decrypted_vote| *decrypted_vote == messages[0]));
        assert!(decrypted_votes
            .iter()
            .any(|decrypted_vote| *decrypted_vote == messages[1]));
        assert!(decrypted_votes
            .iter()
            .any(|decrypted_vote| *decrypted_vote == messages[2]));
    });
}

#[test]
fn test_shuffle_ballots_pk_does_not_exist() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let topic_id = "Moritz for President?".as_bytes().to_vec();
        // try to shuffle the ballots -> public key doesn't exist yet
        OffchainModule::shuffle_ciphers(&topic_id)
            .expect_err("The returned value should be: 'Error::<T>::PublicKeyNotExistsError'");
    });
}

#[test]
fn test_shuffle_ballots_no_ballots() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let topic_id = "Moritz for President?".as_bytes().to_vec();

        // create the submitter (i.e. the public key submitter)
        let account: <TestRuntime as system::Trait>::AccountId = Default::default();
        let who = Origin::signed(account);

        // create the public key
        let (_, _, pk) = Helper::setup_sm_system();

        // store created public key and public parameters
        let public_key_storage = OffchainModule::store_public_key(who, pk.clone().into());
        assert_ok!(public_key_storage);

        // try -> to shuffle the ballots (which don't exist)
        OffchainModule::shuffle_ciphers(&topic_id)
            .expect_err("The returned value should be: 'Error::<T>::ShuffleCiphersSizeZeroError'");
    });
}

#[test]
fn test_permute_vector() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let test_vec: Vec<BigUint> = vec![
            BigUint::from(5u32),
            BigUint::from(10u32),
            BigUint::from(15u32),
        ];
        let permutation: Vec<usize> = vec![2, 0, 1];

        let result = OffchainModule::permute_vector(test_vec.clone(), &permutation);
        assert_eq!(result[0], test_vec[2]);
        assert_eq!(result[1], test_vec[0]);
        assert_eq!(result[2], test_vec[1]);
    });
}

#[test]
fn test_shuffle_proof_small_system() {
    // good primes to use for testing
    // p: 202178360940839 -> q: 101089180470419
    // p: 4283 -> q: 2141
    // p: 59 -> q: 29
    // p: 47 -> q: 23
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let (params, _, pk) = Helper::setup_sm_system();
        let (vote_id, topic_id) = setup_vote(params.into());
        let is_p_prime = OffchainModule::is_prime(&pk.params.p, 10).unwrap();
        assert!(is_p_prime);
        let is_q_prime = OffchainModule::is_prime(&pk.params.q(), 10).unwrap();
        assert!(is_q_prime);

        let is_proof_valid = shuffle_proof_test(vote_id, topic_id, pk);
        assert!(is_proof_valid);
    });
}

#[test]
fn test_shuffle_proof_tiny_system() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let (params, _, pk) = Helper::setup_tiny_system();
        let (vote_id, topic_id) = setup_vote(params.into());
        let is_p_prime = OffchainModule::is_prime(&pk.params.p, 10).unwrap();
        assert!(is_p_prime);
        let is_q_prime = OffchainModule::is_prime(&pk.params.q(), 10).unwrap();
        assert!(is_q_prime);

        let is_proof_valid = shuffle_proof_test(vote_id, topic_id, pk);
        assert!(is_proof_valid);
    });
}

#[test]
fn test_shuffle_proof_medium_system() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let (params, _, pk) = Helper::setup_md_system();
        let (vote_id, topic_id) = setup_vote(params.into());
        let is_p_prime = OffchainModule::is_prime(&pk.params.p, 10).unwrap();
        assert!(is_p_prime);
        let is_q_prime = OffchainModule::is_prime(&pk.params.q(), 10).unwrap();
        assert!(is_q_prime);

        let is_proof_valid = shuffle_proof_test(vote_id, topic_id, pk);
        assert!(is_proof_valid);
    });
}

#[test]
#[ignore = "will take over 30s to complete, run only when necessary"]
fn test_shuffle_proof_large_system() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let (params, _, pk) = Helper::setup_lg_system();
        let (vote_id, topic_id) = setup_vote(params.into());
        let is_p_prime = OffchainModule::is_prime(&pk.params.p, 10).unwrap();
        assert!(is_p_prime);
        let is_q_prime = OffchainModule::is_prime(&pk.params.q(), 10).unwrap();
        assert!(is_q_prime);

        let is_proof_valid = shuffle_proof_test(vote_id, topic_id, pk);
        assert!(is_proof_valid);
    });
}

#[test]
#[ignore = "will take over 60s to complete, run only when necessary"]
fn test_shuffle_proof_xl_system() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let (params, _, pk) = Helper::setup_xl_system();
        let (vote_id, topic_id) = setup_vote(params.into());
        let is_p_prime = OffchainModule::is_prime(&pk.params.p, 10).unwrap();
        assert!(is_p_prime);
        let is_q_prime = OffchainModule::is_prime(&pk.params.q(), 10).unwrap();
        assert!(is_q_prime);

        let is_proof_valid = shuffle_proof_test(vote_id, topic_id, pk);
        assert!(is_proof_valid);
    });
}

fn shuffle_proof_test(vote_id: Vec<u8>, topic_id: Vec<u8>, pk: ElGamalPK) -> bool {
    let messages = vec![
        BigUint::from(0u32),
        BigUint::from(1u32),
        BigUint::from(2u32),
        BigUint::from(0u32),
        BigUint::from(1u32),
        BigUint::from(2u32),
    ];

    // create the submitter (i.e. the public key submitter)
    let account: <TestRuntime as system::Trait>::AccountId = Default::default();
    let who = Origin::signed(account);

    // store created public key and public parameters
    let public_key_storage = OffchainModule::store_public_key(who, pk.clone().into());
    assert_ok!(public_key_storage);

    // encrypt the message -> encrypted message
    // cipher = the crypto crate version of a ballot { a: BigUint, b: BigUint }
    let randoms = vec![b"08", b"17", b"01", b"16", b"11", b"00"];
    assert_eq!(messages.len(), randoms.len());

    // create the voter (i.e. the transaction signer)
    let account: <TestRuntime as system::Trait>::AccountId = Default::default();
    let voter = Origin::signed(account);

    for index in 0..messages.len() {
        let random = BigUint::parse_bytes(randoms[index], 10).unwrap();

        // transform the ballot into a from that the blockchain can handle
        // i.e. a Substrate representation { a: Vec<u8>, b: Vec<u8> }
        let cipher: Cipher = ElGamal::encrypt(&messages[index], &random, &pk).into();
        let answers: Vec<(TopicId, Cipher)> = vec![(topic_id.clone(), cipher)];
        let ballot: Ballot = Ballot { answers };

        let vote_submission_result =
            OffchainModule::cast_ballot(voter.clone(), vote_id.clone(), ballot);
        assert_ok!(vote_submission_result);
    }

    // get the encrypted votes
    let big_ciphers_from_chain: Vec<BigCipher> = Wrapper(OffchainModule::ciphers(&topic_id)).into();
    assert!(big_ciphers_from_chain.len() > 0);

    // shuffle the votes
    let shuffle_result = OffchainModule::shuffle_ciphers(&topic_id);
    let shuffled: (Vec<BigCipher>, Vec<BigUint>, Vec<usize>) = shuffle_result.unwrap();
    let shuffled_ciphers = shuffled.0;
    let re_encryption_randoms = shuffled.1;
    let permutation = &shuffled.2;

    // TEST
    // GENERATE PROOF
    let result = OffchainModule::generate_shuffle_proof(
        &vote_id,
        big_ciphers_from_chain.clone(),
        shuffled_ciphers.clone(),
        re_encryption_randoms,
        permutation,
        &pk,
    );
    let proof: Proof = result.unwrap();

    // VERIFY PROOF
    let verification = OffchainModule::verify_shuffle_proof(
        &vote_id,
        proof,
        big_ciphers_from_chain,
        shuffled_ciphers,
        &pk,
    );
    let is_proof_valid = verification.unwrap();
    is_proof_valid
}
