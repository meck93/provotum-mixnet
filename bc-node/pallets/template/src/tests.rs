use crate::types::Vote;
use crate::{mock::*, Error};
use crypto::elgamal::encryption::ElGamal;
use crypto::elgamal::system::{Cipher, ElGamalParams, PrivateKey, PublicKey};
use frame_support::{assert_noop, assert_ok};
use num_bigint::BigUint;

// helper function to setup ElGamal system before a test
fn setup_system(p: &[u8], g: &[u8], x: &[u8]) -> (ElGamalParams, PrivateKey, PublicKey) {
    let params = ElGamalParams {
        p: BigUint::parse_bytes(p, 10).unwrap(),
        g: BigUint::parse_bytes(g, 10).unwrap(),
    };
    let sk = PrivateKey {
        params: params.clone(),
        x: BigUint::parse_bytes(x, 10).unwrap(),
    };
    let pk = PublicKey {
        params: params.clone(),
        h: params.g.modpow(&sk.x, &params.p),
    };
    (params, sk, pk)
}

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
        // Read pallet storage and assert an expected result.
        assert_eq!(TemplateModule::something(), Some(42));
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            TemplateModule::cause_error(Origin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}

#[test]
fn store_small_dummy_vote() {
    new_test_ext().execute_with(|| {
        let (_, _, pk) = setup_system(b"23", b"2", b"7");
        let message = BigUint::from(1u32);
        let random = BigUint::from(7u32);

        // encrypt the message -> encrypted message
        // cipher = the crypto crate version of a vote { a: BigUint, b: BigUint }
        let cipher: Cipher = ElGamal::encrypt(&message, &random, &pk);
        println!("Vote: {:?}", message);
        println!("Encrypted Vote: {:?}", cipher);

        // transform the vote into a from that the blockchain can handle
        // i.e. a Substrate representation { a: Vec<u8>, b: Vec<u8> }
        let encrypted_vote: Vote = cipher.into();
        let voter = Origin::signed(1);

        let vote = TemplateModule::cast_encrypted_ballot(voter, encrypted_vote);
        assert_ok!(vote);
    })
}

#[test]
fn store_real_size_vote() {
    new_test_ext().execute_with(|| {
        let (_, _, pk) = setup_system(b"85053461164796801949539541639542805770666392330682673302530819774105141531698707146930307290253537320447270457", b"2", b"1701411834604692317316873037");
        let message = BigUint::from(1u32);
        let random = BigUint::parse_bytes(b"170141183460469231731687303715884", 10).unwrap();

        // encrypt the message -> encrypted message
        // cipher = the crypto crate version of a vote { a: BigUint, b: BigUint }
        let cipher: Cipher = ElGamal::encrypt(&message, &random, &pk);
        println!("Vote: {:?}", message);
        println!("Encrypted Vote: {:?}", cipher);

        // transform the vote into a from that the blockchain can handle
        // i.e. a Substrate representation { a: Vec<u8>, b: Vec<u8> }
        let encrypted_vote: Vote = cipher.into();
        let voter = Origin::signed(1);

        let vote = TemplateModule::cast_encrypted_ballot(voter, encrypted_vote);
        assert_ok!(vote);
    })
}
