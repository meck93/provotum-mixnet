//! This library provides common cryptographic functionality for working within
//! the exponential ElGamal cryptosystem.

// Declaring our library as `no-std` unconditionally lets us be consistent
// in how we `use` items from `std` or `core`
#![no_std]

// We always pull in `std` during tests, because it's just easier
// to write tests when you can assume you're on a capable platform
#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

// crates which this library exposes
pub mod elgamal;