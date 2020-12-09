use criterion::{criterion_group, criterion_main, Criterion};
use crypto::{
    encryption::ElGamal,
    helper::Helper,
    types::{Cipher, PublicKey},
};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::time::Duration;

fn setup_shuffling(nr_of_votes: usize) -> (Vec<Cipher>, Vec<usize>, Vec<BigUint>, PublicKey) {
    let (_, _, pk) = Helper::setup_lg_system();
    let q = pk.params.q();

    // encryption of zero
    let zero = BigUint::zero();
    let r = BigUint::parse_bytes(b"1234", 10).unwrap();
    let enc_zero = ElGamal::encrypt(&zero, &r, &pk);

    // encryption of one
    let one = BigUint::one();
    let r_ = BigUint::parse_bytes(b"4321", 10).unwrap();
    let enc_one = ElGamal::encrypt(&one, &r_, &pk);

    let mut encryptions: Vec<Cipher> = Vec::new();
    let mut randoms: Vec<BigUint> = Vec::new();
    let power = BigUint::parse_bytes(b"ABCDEF123456789ABCDEF123412341241241241124", 16).unwrap();
    let mut permutation: Vec<usize> = Vec::new();

    for i in 0..nr_of_votes {
        permutation.push(i);

        let mut random = BigUint::from(i);
        random *= BigUint::from(i);
        random = random.modpow(&power, &q);
        randoms.push(random);

        if i % 2 == 0 {
            encryptions.push(enc_zero.clone());
        } else {
            encryptions.push(enc_one.clone());
        }
    }

    // create a fake permutation
    permutation.reverse();

    assert!(encryptions.len() == randoms.len());
    assert!(encryptions.len() == permutation.len());
    assert!(encryptions.len() == nr_of_votes);
    (encryptions, permutation, randoms, pk)
}

fn bench_elgamal(c: &mut Criterion) {
    // benchmark config
    let mut group = c.benchmark_group("elgamal");
    group.measurement_time(Duration::new(15, 0));
    group.sample_size(350);

    group.bench_function("encryption", |b| {
        b.iter_with_setup(
            || {
                let (_, _, pk) = Helper::setup_lg_system();
                let message = BigUint::from(1u32);
                let random =
                    BigUint::parse_bytes(b"170141183460469231731687303715884", 10).unwrap();
                (message, random, pk)
            },
            |(m, r, pk)| ElGamal::encrypt(&m, &r, &pk),
        )
    });

    group.bench_function("decryption", |b| {
        b.iter_with_setup(
            || {
                let (_, sk, pk) = Helper::setup_lg_system();
                let message = BigUint::from(1u32);
                let random =
                    BigUint::parse_bytes(b"170141183460469231731687303715884", 10).unwrap();

                // encrypt the message
                let encrypted_message = ElGamal::encrypt(&message, &random, &pk);
                (encrypted_message, sk)
            },
            |(encrypted_message, sk)| ElGamal::decrypt(&encrypted_message, &sk),
        )
    });

    group.bench_function("homomorphic addition", |b| {
        b.iter_with_setup(
            || {
                let (params, _, pk) = Helper::setup_lg_system();
                let one = BigUint::one();

                // encrypt the message
                let r = BigUint::parse_bytes(b"170141183460469231731687303715884", 10).unwrap();
                let enc_one = ElGamal::encrypt(&one, &r, &pk);

                // encrypt the message again
                let r_ = BigUint::parse_bytes(b"170141183460469231731687303712342", 10).unwrap();
                let enc_one_ = ElGamal::encrypt(&one, &r_, &pk);

                (enc_one, enc_one_, params.p)
            },
            |(enc_one, enc_one_, p)| ElGamal::add(&enc_one, &enc_one_, &p),
        )
    });

    group.bench_function("re_encryption", |b| {
        b.iter_with_setup(
            || {
                let (_, _, pk) = Helper::setup_lg_system();
                let one = BigUint::one();

                // encrypt the message
                let r = BigUint::parse_bytes(b"170141183460469231731687303715884", 10).unwrap();
                let encryption = ElGamal::encrypt(&one, &r, &pk);

                // use another random value for the re_encryption
                let r_ = BigUint::parse_bytes(b"170141183460469231731687303712342", 10).unwrap();

                (encryption, r_, pk)
            },
            |(encryption, r_, pk)| ElGamal::re_encrypt(&encryption, &r_, &pk),
        )
    });

    group.bench_function("re_encryption by homomorphic addition zero (g^0)", |b| {
        b.iter_with_setup(
            || {
                let (_, _, pk) = Helper::setup_lg_system();
                let one = BigUint::one();

                // encrypt the message
                let r = BigUint::parse_bytes(b"170141183460469231731687303715884", 10).unwrap();
                let encryption = ElGamal::encrypt(&one, &r, &pk);

                // use another random value for the re_encryption
                let r_ = BigUint::parse_bytes(b"170141183460469231731687303712342", 10).unwrap();

                (encryption, r_, pk)
            },
            |(encryption, r_, pk)| ElGamal::re_encrypt_via_addition(&encryption, &r_, &pk),
        )
    });
}

fn bench_shuffling(c: &mut Criterion) {
    // benchmark config
    let mut group = c.benchmark_group("shuffling");
    group.sample_size(20);

    group.bench_function("3 votes", |b| {
        b.iter_with_setup(
            || setup_shuffling(3),
            |(encryptions, permutation, randoms, pk)| {
                ElGamal::shuffle(&encryptions, &permutation, &randoms, &pk)
            },
        )
    });

    group.bench_function("30 votes", |b| {
        b.iter_with_setup(
            || setup_shuffling(30),
            |(encryptions, permutation, randoms, pk)| {
                ElGamal::shuffle(&encryptions, &permutation, &randoms, &pk)
            },
        )
    });

    group.bench_function("100 votes", |b| {
        b.iter_with_setup(
            || setup_shuffling(100),
            |(encryptions, permutation, randoms, pk)| {
                ElGamal::shuffle(&encryptions, &permutation, &randoms, &pk)
            },
        )
    });

    group.finish();
}

criterion_group!(benches, bench_elgamal, bench_shuffling);
criterion_main!(benches);
