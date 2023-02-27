# multiplatform_test

Provides a proc-macro that expands to testing on platforms relevant to
Hydroflow. By default, expands to testing on the host (using the normal
`#[test]` attribute) and wasm (using `#[wasm_bindgen_test]`.

For example, the test

```rust
use multiplatform_test::multiplatform_test;

#[multiplatform_test]
fn my_test() {
  // ...
}
```

Expands to

```rust
#[test]
#[wasm_bindgen_test_macro::wasm_bindgen_test]
fn my_test() {
  // ...
}
```

## Installation

Because the `multiplaform_test` macro expands to `wasm_bindgen_test_macro`, you
will also need to depend on the
[`wasm_bindgen_test_macro`](https://crates.io/crates/wasm-bindgen-test-macro/)
crate.

## Usage

### Specifying platforms

You can test on a subset of platforms by passing in the platforms in parens:

```rust
use multiplatform_test::multiplatform_test;

#[multiplatform_test(default)]  // Only test on the default #[test] platform
fn my_test() {
  // ...
}
```

expands to


```rust
use multiplatform_test::multiplatform_test;

#[test]
fn my_test() {
  // ...
}
```

