

## v0.5.5 (2024-07-23)

### Documentation

 - <csr-id-b4e226f1305a9631083bb6e9c7e5f01cc7c9aa90/> add `#[derive(Lattice)]` docs to README, import into book, fix #1259

### New Features

 - <csr-id-b3d01c20cae2335a3da2c02343debe677f17786b/> add `#[derive(Lattice)]` derive macros, fix #1247
   This adds derive macros to allow user-created macros. Each field must be
   a lattice.
   
   Example usage:
   ```rust
   struct MyLattice<KeySet, Epoch>
   where
       KeySet: Collection,
       Epoch: Ord,
   {
       keys: SetUnion<KeySet>,
       epoch: Max<Epoch>,
   }
   ```
   
   Uses `#[derive(Lattice)]` for the `lattices` library `Pair` lattice.
   Also contains some cleanup in the `lattices` crate.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 49 calendar days.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#1250](https://github.com/hydro-project/hydroflow/issues/1250), [#1267](https://github.com/hydro-project/hydroflow/issues/1267)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1250](https://github.com/hydro-project/hydroflow/issues/1250)**
    - Add `#[derive(Lattice)]` derive macros, fix #1247 ([`b3d01c2`](https://github.com/hydro-project/hydroflow/commit/b3d01c20cae2335a3da2c02343debe677f17786b))
 * **[#1267](https://github.com/hydro-project/hydroflow/issues/1267)**
    - Add `#[derive(Lattice)]` docs to README, import into book, fix #1259 ([`b4e226f`](https://github.com/hydro-project/hydroflow/commit/b4e226f1305a9631083bb6e9c7e5f01cc7c9aa90))
</details>

