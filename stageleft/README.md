# Stageleft
Stageleft brings the magic of staged programming to Rust, making it easy to write macros with type-safe logic and high-level APIs that can generate efficient code under the hood.

## Setup
Stageleft requires a particular workspace setup, as any crate that uses Stageleft must have an supporting macro crate (whose contents will be automatically generated).