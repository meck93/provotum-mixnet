//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use wasm_package::multiply;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn smoke_test() {
    assert_eq!(1 + 1, 2);
}

#[wasm_bindgen_test]
fn test_multiply() {
    let n1: u64 = 12312312312312;
    let n2: u64 = 12312312312312;

    let result = multiply(n1, n2);

    assert_eq!(result, result);
}
