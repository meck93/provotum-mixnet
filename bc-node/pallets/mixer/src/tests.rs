use crate::mock::*;
use crate::*;
use codec::Decode;
use crypto::elgamal::{types::Cipher, encryption::ElGamal, helper::Helper};
use frame_support::assert_ok;
use sp_std::if_std;
use pallet_mixnet::types::Ballot;

#[test]
fn test_submit_number_signed_works() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        // call submit_number_signed
        let num = 32;
        let acct: <TestRuntime as system::Trait>::AccountId = Default::default();
        assert_ok!(OffchainModule::submit_number_signed(
            Origin::signed(acct),
            num
        ));
        // A number is inserted to <Numbers> vec
        assert_eq!(<Numbers>::get(), vec![num]);
        // An event is emitted
        assert!(System::events()
            .iter()
            .any(|er| er.event == TestEvent::pallet_mixer(RawEvent::NewNumber(Some(acct), num))));

        // Insert another number
        let num2 = num * 2;
        assert_ok!(OffchainModule::submit_number_signed(
            Origin::signed(acct),
            num2
        ));
        // A number is inserted to <Numbers> vec
        assert_eq!(<Numbers>::get(), vec![num, num2]);
    });
}

#[test]
fn test_offchain_signed_tx() {
    let (mut t, pool_state, _) = ExternalityBuilder::build();

    t.execute_with(|| {
        // Setup
        let num = 32;
        OffchainModule::offchain_signed_tx(num).unwrap();

        // Verify
        let tx = pool_state.write().transactions.pop().unwrap();
        assert!(pool_state.read().transactions.is_empty());
        let tx = TestExtrinsic::decode(&mut &*tx).unwrap();
        assert_eq!(tx.signature.unwrap().0, 0);
        assert_eq!(tx.call, Call::submit_number_signed(num));
    });
}

#[test]
fn test_offchain_signed_tx_random_number() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let acct: <TestRuntime as system::Trait>::AccountId = Default::default();
        let origin = Origin::signed(acct);
        let tx_result = OffchainModule::random(origin);
        assert_ok!(tx_result);
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
        let random = OffchainModule::get_random_less_than(&upper_bound).unwrap();
        assert!(random < upper_bound);
    });
}

#[test]
fn test_get_random_number_less_than_should_panic_number_is_zero() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let upper_bound: BigUint = BigUint::parse_bytes(b"0", 10).unwrap();
        OffchainModule::get_random_less_than(&upper_bound).expect_err(
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
        
        if_std! {
            println!("random value in range. lower: {:?}, upper: {:?}, value: {:?}", lower, upper, value);
        }
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
        
        if_std! {
            println!("random value in range. lower: {:?}, upper: {:?}, value: {:?}", lower, upper, value);
        }
    });
}

#[test]
fn test_get_random_range_upper_is_zero_error() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let lower: usize = 0;
        let upper: usize = 0;
        OffchainModule::get_random_range(lower, upper).expect_err("The returned value should be: '<Error<T>>::RandomRangeError'");
    });
}

#[test]
fn test_get_random_range_upper_is_not_larger_than_lower_error() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let lower: usize = 5;
        let upper: usize = 5;
        OffchainModule::get_random_range(lower, upper).expect_err("The returned value should be: '<Error<T>>::RandomRangeError'");
    });
}

#[test]
fn test_generate_permutation_size_zero_error() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let size = 0;
        OffchainModule::generate_permutation(size).expect_err("The returned value should be: '<Error<T>>::PermutationSizeZeroError'");
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

////////////////////////////////////////////////////
/// Integration Tests -> together with pallet_mixnet

#[test]
fn integration_test_fetch_ballots() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {        
        // Read pallet storage (i.e. the submitted ballots) 
        // and assert an expected result.                
        let votes_from_chain: Vec<Ballot> = MixnetModule::ballots();
        assert!(votes_from_chain.len() == 0);
    });
}

#[test]
fn integration_test_cast_ballot() {
    let (mut t, _, _) = ExternalityBuilder::build();
    t.execute_with(|| {
        let account: <TestRuntime as system::Trait>::AccountId = Default::default();
        let voter = Origin::signed(account);

        let (_, sk, pk) = Helper::setup_system(b"85053461164796801949539541639542805770666392330682673302530819774105141531698707146930307290253537320447270457", 
        b"2", 
        b"1701411834604692317316873037");
        let message = BigUint::from(1u32);
        let random = BigUint::parse_bytes(b"170141183460469231731687303715884", 10).unwrap();

        // encrypt the message -> encrypted message
        // cipher = the crypto crate version of a ballot { a: BigUint, b: BigUint }
        let cipher: Cipher = ElGamal::encrypt(&message, &random, &pk);

        // transform the ballot into a from that the blockchain can handle
        // i.e. a Substrate representation { a: Vec<u8>, b: Vec<u8> }
        let encrypted_vote: Ballot = cipher.clone().into();

        let vote_submission_result = MixnetModule::cast_ballot(voter, encrypted_vote.clone());
        assert_ok!(vote_submission_result);

        // fetch the submitted ballot
        let votes_from_chain: Vec<Ballot> = MixnetModule::ballots();
        assert!(votes_from_chain.len() > 0);

        let vote_from_chain: Ballot = votes_from_chain[0].clone();
        assert_eq!(encrypted_vote, vote_from_chain);
        
        // transform the Ballot -> Cipher
        let cipher_from_chain: Cipher = vote_from_chain.into();
        assert_eq!(cipher, cipher_from_chain);

        let decrypted_vote = ElGamal::decrypt(&cipher_from_chain, &sk);
        assert_eq!(message, decrypted_vote);
    });
}
