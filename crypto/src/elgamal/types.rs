use core::ops::Mul;
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};

pub trait ModuloOperations {
    /// Calculates the modular multiplicative of a BigUint: result = self * multiplier % modulus.
    fn modmul(&self, rhs: &Self, modulus: &Self) -> Self;

    /// Calculates the modular multiplicative inverse x of an integer a such that ax ≡ 1 (mod m).
    /// Alternative formulation: a^-1 (mod m)
    fn invmod(&self, modulus: &Self) -> Option<BigUint>;
    // fn extended_gcd(a: &BigUint, b: &BigUint) -> (BigUint, BigUint, BigUint);
}

impl ModuloOperations for BigUint {
    fn modmul(&self, multiplier: &Self, modulus: &Self) -> Self {
        assert!(
            !modulus.is_zero(),
            "attempt to calculate with zero modulus!"
        );
        self.mul(multiplier) % modulus
    }

    fn invmod(&self, modulus: &Self) -> Option<BigUint> {
        assert!(
            !modulus.is_zero(),
            "attempt to calculate with zero modulus!"
        );
        assert!(
            self < modulus,
            "modulus must be greater or equal to the number!"
        );
        let a = BigInt::from(self.clone());
        let b = BigInt::from(modulus.clone());

        let (g, x, _) = extended_gcd(&a, &b);
        if g != BigInt::one() {
            None
        } else {
            let result = ((x % &b) + &b) % &b;
            result.to_biguint()
        }
    }
}

fn extended_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    assert!(a < b, "a must be smaller than b!");
    if *a == BigInt::zero() {
        (b.clone(), BigInt::zero(), BigInt::one())
    } else {
        let (g, x, y) = extended_gcd(&(b % a), &a);
        (g, y - (b / a) * x.clone(), x)
    }
}

#[cfg(test)]
mod tests {
    use crate::elgamal::types::ModuloOperations;
    use num_bigint::BigUint;
    use num_traits::Zero;

    #[test]
    fn is_modulo_multiplication() {
        let three = BigUint::from(3 as u32);
        let six = BigUint::from(6 as u32);
        let ten = BigUint::from(10 as u32);

        let eight = six.modmul(&three, &ten);
        assert_eq!(eight, BigUint::from(8 as u32));
    }

    #[test]
    #[should_panic(expected = "attempt to calculate with zero modulus!")]
    fn it_should_not_use_modulus_zero() {
        let three = BigUint::from(3 as u32);
        let six = BigUint::from(6 as u32);
        let zero = BigUint::zero();

        // should panic since modulus is zero
        six.modmul(&three, &zero);
    }

    #[test]
    fn it_should_compute_inverse_modulo_2_invmod_7() {
        let seven = BigUint::from(7 as u32);
        let two = BigUint::from(2 as u32);

        let expected_result = BigUint::from(4 as u32);

        let result = two.invmod(&seven);
        let inverse: BigUint = result.unwrap();
        assert_eq!(expected_result, inverse);
    }

    #[test]
    fn it_should_compute_inverse_modulo_17_invmod_23() {
        let seventeen = BigUint::from(17 as u32);
        let twentythree = BigUint::from(23 as u32);

        let expected_result = BigUint::from(19 as u32);

        let result = seventeen.invmod(&twentythree);
        let inverse: BigUint = result.unwrap();
        assert_eq!(expected_result, inverse);
    }

    #[test]
    #[should_panic(expected = "modulus must be greater or equal to the number!")]
    fn it_should_panic_modulus_is_smaller_than_number() {
        let six = BigUint::from(6 as u32);
        let two = BigUint::from(2 as u32);

        // should panic since two is smaller than six
        six.invmod(&two);
    }

    #[test]
    #[should_panic(expected = "attempt to calculate with zero modulus!")]
    fn it_should_panic_modulus_is_zero() {
        let six = BigUint::from(6 as u32);
        let zero = BigUint::zero();

        // should panic since modulus is zero
        six.invmod(&zero);
    }
}
