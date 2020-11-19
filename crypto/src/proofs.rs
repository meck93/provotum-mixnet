use crate::types::{ElGamalParams, ModuloOperations};
use crate::{
    helper::Helper,
    types::{Cipher, PermutationCommitment, PublicKey},
};
use alloc::{vec, vec::Vec};
use num_bigint::BigUint;
use num_traits::One;

pub struct ShuffleProof;

impl ShuffleProof {
    /// The offline part of the shuffle proof
    /// i.e. generate the permutation matrix commitment
    pub fn offline(
        id: usize,
        encryptions: Vec<Cipher>,
        shuffled_encryptions: Vec<Cipher>,
        re_encryption_randoms: Vec<BigUint>,
        permutation: &[usize],
        randoms: Vec<BigUint>,
        pk: &PublicKey,
    ) {
        // input checks
        assert!(
            encryptions.len() == shuffled_encryptions.len(),
            "encryptions and shuffled_encryptions need to have the same length!"
        );
        assert!(
            encryptions.len() == re_encryption_randoms.len(),
            "encryptions and re_encryption_randoms need to have the same length!"
        );
        assert!(
            encryptions.len() == permutation.len(),
            "encryptions and permutation need to have the same length!"
        );
        assert!(
            encryptions.len() == randoms.len(),
            "encryptions and randoms need to have the same length!"
        );
        assert!(!encryptions.is_empty(), "vectors cannot be empty!");

        let size = encryptions.len();
        let params = &pk.params;
        let q = &pk.params.q();

        // get {size} independent generators: h
        let generators = Helper::get_generators(id, q, size);

        // commit to the given permutation: (c, r)
        let permutation_commitment =
            Self::generate_permutation_commitment(params, permutation, randoms, generators);
        let commitments = permutation_commitment.commitments;

        // get {size} challenges: u = get_challenges(size, hash(e,e',c,pk))
        let mut challenges =
            Self::get_challenges(size, encryptions, shuffled_encryptions, commitments, pk);
        let mut temp_ = Vec::new();

        // permute the challenges -> same order as randoms + permuation
        for i in 0..challenges.len() {
            let j_i = permutation[i];
            let u_j_i = challenges[j_i].clone();
            temp_.push(u_j_i);
        }
        assert_eq!(challenges.len(), temp_.len());

        // reassign the permuted challenges
        challenges = temp_;

        // generate commitment chain: (c', r')
        // let commitment_chain = Self::generate_commitment_chain(challenges);
    }

    /// Generates a commitment to a permutation by committing to the columns of the corresponding permutation matrix.
    ///
    /// Inputs:
    /// - params ElGamalParams
    /// - permutation \[usize\]
    /// - randoms Vec<BigUint>, BigUint ∈ G_q
    /// - (independent) generators Vec<BigUint>, BigUint ∈ (G_q \ {1})
    pub fn generate_permutation_commitment(
        params: &ElGamalParams,
        permutation: &[usize],
        randoms: Vec<BigUint>,
        generators: Vec<BigUint>,
    ) -> PermutationCommitment {
        assert!(
            permutation.len() == randoms.len(),
            "permutation and randoms need to have the same length!"
        );
        assert!(
            permutation.len() == generators.len(),
            "permutation and generators need to have the same length!"
        );
        assert!(!permutation.is_empty(), "vectors cannot be empty!");

        let p = &params.p;
        let g = &params.g;
        let one = BigUint::one();
        let too_large = p.clone() + one;

        // initialize a vector of length: random.len() and default value p+1
        let mut commitments: Vec<BigUint> = vec![too_large.clone(); randoms.len()];
        assert!(commitments.len() == randoms.len());

        for i in 0..permutation.len() {
            // get the random value r at position j_i
            let j_i = permutation[i];
            let r_j_i = &randoms[j_i];

            // a random independent generator ∈ G_q
            let h = &generators[i];

            // create commitment
            // g_pow_r_j_i = g^(r_j_i) mod p
            let g_pow_r_j_i = g.modpow(r_j_i, p);

            // c_j_i = (g^(r_j_i) * h_i) mod p
            let c_j_i = g_pow_r_j_i.modmul(h, p);

            // insert c_j_i at position j_i in commitments vector
            let removed = commitments.remove(j_i);
            assert_eq!(removed, too_large);
            commitments.insert(j_i, c_j_i);
        }
        // make sure that none of the commitments are still a p+1 value
        // which is technically not possible since all chosen values are mod p
        // only if a value has not been replaced it can still be p+1
        assert!(commitments.iter().all(|value| value != &too_large));
        assert!(commitments.len() == randoms.len());
        PermutationCommitment {
            commitments,
            randoms,
        }
    }

    pub fn generate_commitment_chain() {
        // TODO: implement according to Alg 8.49 of ch-vote spec
        unimplemented!()
    }

    /// Computes n challenges 0 <= c_i <= 2^tau for a given of public value.
    ///
    /// Inputs:
    /// - number: usize
    /// - encryptions: Vec<Cipher>
    /// - shuffled_encryptions: Vec<Cipher>
    /// - commitments: Vec<BigUint>
    /// - pk: PublicKey
    pub fn get_challenges(
        number: usize,
        encryptions: Vec<Cipher>,
        shuffled_encryptions: Vec<Cipher>,
        commitments: Vec<BigUint>,
        pk: &PublicKey,
    ) -> Vec<BigUint> {
        assert!(number > 0, "at least one challenge must be generated!");
        assert!(
            encryptions.len() == shuffled_encryptions.len(),
            "encryptions and shuffled_encryptions need to have the same length!"
        );
        assert!(
            encryptions.len() == commitments.len(),
            "encryptions and commitments need to have the same length!"
        );
        assert!(!encryptions.is_empty(), "vectors cannot be empty!");
        let mut challenges: Vec<BigUint> = Vec::new();

        // hash all inputs into a single BigUint
        let h = Helper::hash_challenge_inputs(encryptions, shuffled_encryptions, commitments, pk);

        for i in 0..number {
            let i_ = Helper::hash_vec_usize_to_biguint(&[i].to_vec());
            let mut c_i = Helper::hash_vec_biguints_to_biguint([h.clone(), i_].to_vec());

            // hash(h,i_) mod 2^T
            // Verifiable Re-Encryption Mixnets (Haenni, Locher, Koenig, Dubuis) uses c_i ∈ Z_q
            // therefore, we use mod q
            c_i %= pk.params.q();
            challenges.push(c_i);
        }
        challenges
    }

    /// The online part of the shuffle proof
    ///
    pub fn online() {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::ShuffleProof;
    use crate::{helper::Helper, random::Random, types::Cipher};
    use alloc::{vec, vec::Vec};
    use num_bigint::BigUint;
    use num_traits::{One, Zero};

    #[test]
    #[should_panic(expected = "permutation and randoms need to have the same length!")]
    fn it_should_panic_generate_permutation_commitment_different_size_permutations_randoms() {
        let (params, _, pk) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"1701411834604692317316",
        );
        let q = pk.params.q();
        let vote_id = 123usize;

        let randoms: [BigUint; 0] = [];
        let permutation = [1usize];
        let generators = Helper::get_generators(vote_id, &q, 1usize);

        ShuffleProof::generate_permutation_commitment(
            &params,
            &permutation,
            randoms.to_vec(),
            generators,
        );
    }

    #[test]
    #[should_panic(expected = "permutation and generators need to have the same length!")]
    fn it_should_panic_generate_permutation_commitment_different_size_permutations_generators() {
        let (params, _, _) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"1701411834604692317316",
        );

        let randoms = [BigUint::one()];
        let permutation = [1usize];
        let generators = Vec::new();

        ShuffleProof::generate_permutation_commitment(
            &params,
            &permutation,
            randoms.to_vec(),
            generators,
        );
    }

    #[test]
    #[should_panic(expected = "vectors cannot be empty!")]
    fn it_should_panic_generate_permutation_commitment_empty_inputs() {
        let (params, _, _) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"1701411834604692317316",
        );

        let randoms = [];
        let permutation = [];
        let generators = Vec::new();

        ShuffleProof::generate_permutation_commitment(
            &params,
            &permutation,
            randoms.to_vec(),
            generators,
        );
    }

    #[test]
    fn it_should_generate_permutation_commitment() {
        let (params, _, _) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"1701411834604692317316",
        );
        let q = params.q();
        let vote_id = 123usize;

        // create a list of permutation
        let size = 3usize;
        let permutation = Random::generate_permutation(&size);

        // create three random values < q
        let randoms = [
            Random::get_random_less_than(&q),
            Random::get_random_less_than(&q),
            Random::get_random_less_than(&q),
        ];

        // get random generators ∈ G_q
        let generators = Helper::get_generators(vote_id, &q, size);

        // generate commitment
        let permutation_commitment = ShuffleProof::generate_permutation_commitment(
            &params,
            &permutation,
            randoms.to_vec(),
            generators,
        );

        // check that all commitments are: commitment < p
        assert!(permutation_commitment
            .commitments
            .iter()
            .all(|c| c < &params.p));
        // check that both have same number of elements: |commitments| == |randoms|
        assert_eq!(
            permutation_commitment.commitments.len(),
            permutation_commitment.randoms.len()
        );
    }

    #[test]
    #[should_panic(expected = "at least one challenge must be generated!")]
    fn it_should_panic_get_challenges_zero_challenges() {
        // SETUP
        let (_, _, pk) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"1701411834604692317316",
        );

        // fake values
        let size = 0usize;
        let encryptions = Vec::new();
        let shuffled_encryptions = Vec::new();
        let commitments = Vec::new();

        // TEST
        ShuffleProof::get_challenges(size, encryptions, shuffled_encryptions, commitments, &pk);
    }

    #[test]
    #[should_panic(expected = "encryptions and shuffled_encryptions need to have the same length!")]
    fn it_should_panic_get_challenges_different_sizes_encryptions_shuffled_encryptions() {
        // SETUP
        let (_, _, pk) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"1701411834604692317316",
        );

        // fake values
        let size = 1usize;
        let encryptions = vec![Cipher {
            a: BigUint::one(),
            b: BigUint::one(),
        }];
        let shuffled_encryptions = Vec::new();
        let commitments = Vec::new();

        // TEST
        ShuffleProof::get_challenges(size, encryptions, shuffled_encryptions, commitments, &pk);
    }

    #[test]
    #[should_panic(expected = "encryptions and commitments need to have the same length!")]
    fn it_should_panic_get_challenges_different_sizes_encryptions_randoms() {
        // SETUP
        let (_, _, pk) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"1701411834604692317316",
        );

        // fake values
        let size = 1usize;
        let encryptions = vec![Cipher {
            a: BigUint::one(),
            b: BigUint::one(),
        }];
        let shuffled_encryptions = vec![Cipher {
            a: BigUint::zero(),
            b: BigUint::zero(),
        }];
        let commitments = Vec::new();

        // TEST
        ShuffleProof::get_challenges(size, encryptions, shuffled_encryptions, commitments, &pk);
    }

    #[test]
    #[should_panic(expected = "vectors cannot be empty!")]
    fn it_should_panic_get_challenges_empty_inputs() {
        // SETUP
        let (_, _, pk) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"1701411834604692317316",
        );

        // fake values
        let size = 1usize;
        let encryptions = Vec::new();
        let shuffled_encryptions = Vec::new();
        let re_encryption_randoms = Vec::new();

        // TEST
        ShuffleProof::get_challenges(
            size,
            encryptions,
            shuffled_encryptions,
            re_encryption_randoms,
            &pk,
        );
    }

    #[test]
    fn it_should_get_challenges() {
        // SETUP
        let (_, _, pk) = Helper::setup_system(
            b"170141183460469231731687303715884105727",
            b"1701411834604692317316",
        );

        let vote_id = 123usize;
        let size = 3usize;
        let q = &pk.params.q();
        let params = &pk.params;

        // generates a shuffle of three random encryptions of values: zero, one, two
        let encryptions = Random::generate_random_encryptions(&pk, &pk.params.q()).to_vec();
        let shuffle = Random::generate_shuffle(&pk, &pk.params.q(), encryptions.clone());

        // get the shuffled_encryptions & permutation from the shuffle
        let shuffled_encryptions = shuffle
            .iter()
            .map(|item| item.0.clone())
            .collect::<Vec<Cipher>>();
        assert!(shuffled_encryptions.len() == size);
        let permutation = shuffle.iter().map(|item| item.2).collect::<Vec<usize>>();
        assert!(permutation.len() == size);

        // generate {size} random values
        let mut randoms: Vec<BigUint> = Vec::new();
        for _ in 0..size {
            randoms.push(Random::get_random_less_than(q));
        }

        // get {size} independent generators
        let generators = Helper::get_generators(vote_id, q, size);

        // get the permutation commitents
        let permutation_commitment = ShuffleProof::generate_permutation_commitment(
            params,
            &permutation,
            randoms,
            generators,
        );
        let commitments = permutation_commitment.commitments;

        // TEST: challenge value generation
        let challenges =
            ShuffleProof::get_challenges(size, encryptions, shuffled_encryptions, commitments, &pk);

        // check that:
        // 1. three challenges are generated
        // 2. all challenge values are < q
        assert_eq!(challenges.len(), 3);
        assert!(challenges.iter().all(|value| value < &pk.params.q()));
    }
}
