use num_bigint::BigUint;
use num_traits::One;

pub fn return_big_uint_1() -> BigUint {
    One::one()
}

pub fn add_big_unint(a: &BigUint, b: &BigUint) -> BigUint {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_return_big_uint_one() {
        let one = return_big_uint_1();
        assert_eq!(BigUint::one(), one);
    }

    #[test]
    fn it_should_add_two_biguints() {
        // uses little-endian format
        let a_digits = vec![5, 0];
        let a = BigUint::new(a_digits);

        let b_digits = vec![3, 0];
        let b = BigUint::new(b_digits);

        // pass references (borrows) of a & b
        let computed_result = add_big_unint(&a, &b);

        // computed result
        let result = a + b;

        assert_eq!(result, computed_result);
    }
}
