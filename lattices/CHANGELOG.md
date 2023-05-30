# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.2 (2023-05-30)

### New Features

 - <csr-id-ecff609a0153446efc1809230ae100964bb9f89b/> print out items when lattice identity tests fail

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release over the course of 6 calendar days.
 - 6 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#691](https://github.com/hydro-project/hydroflow/issues/691)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#691](https://github.com/hydro-project/hydroflow/issues/691)**
    - Print out items when lattice identity tests fail ([`ecff609`](https://github.com/hydro-project/hydroflow/commit/ecff609a0153446efc1809230ae100964bb9f89b))
</details>

## 0.1.1 (2023-05-23)

<csr-id-3bee6f858a78d82b7431e124ef9792002c8d77ce/>

### Documentation

 - <csr-id-720744fc90fa05a11e0b79c96baba2eb6fd1c7f3/> simplified explanations, fixed typos, removed dead named links
 - <csr-id-4bc1ac1ea2fa6257219ec7fae94a2b039ec7eb7b/> update links from old to new book

### Refactor

 - <csr-id-3bee6f858a78d82b7431e124ef9792002c8d77ce/> update cc-traits to v2, remove `SimpleKeyedRef` shim

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 2 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#671](https://github.com/hydro-project/hydroflow/issues/671), [#674](https://github.com/hydro-project/hydroflow/issues/674), [#687](https://github.com/hydro-project/hydroflow/issues/687)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#671](https://github.com/hydro-project/hydroflow/issues/671)**
    - Migrate docs to a unified Docusuarus site ([`feed326`](https://github.com/hydro-project/hydroflow/commit/feed3268c0aabeb027b19abd9ed06c565a0462f4))
 * **[#674](https://github.com/hydro-project/hydroflow/issues/674)**
    - Update cc-traits to v2, remove `SimpleKeyedRef` shim ([`3bee6f8`](https://github.com/hydro-project/hydroflow/commit/3bee6f858a78d82b7431e124ef9792002c8d77ce))
 * **[#687](https://github.com/hydro-project/hydroflow/issues/687)**
    - Simplified explanations, fixed typos, removed dead named links ([`720744f`](https://github.com/hydro-project/hydroflow/commit/720744fc90fa05a11e0b79c96baba2eb6fd1c7f3))
    - Update links from old to new book ([`4bc1ac1`](https://github.com/hydro-project/hydroflow/commit/4bc1ac1ea2fa6257219ec7fae94a2b039ec7eb7b))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.1.0, hydroflow_internalmacro v0.1.0, hydroflow_lang v0.1.0, hydroflow_datalog_core v0.1.0, hydroflow_datalog v0.1.0, hydroflow_macro v0.1.0, lattices v0.1.1, hydroflow v0.1.0 ([`7324974`](https://github.com/hydro-project/hydroflow/commit/73249744293c9b89cbaa2d84b23ca3f25b00ae4e))
</details>

## 0.1.0 (2023-05-21)

<csr-id-cd0a86d9271d0e3daab59c46f079925f863424e1/>
<csr-id-20a1b2c0cd04a8b495a02ce345db3d48a99ea0e9/>
<csr-id-1eda91a2ef8794711ef037240f15284e8085d863/>

### Documentation

 - <csr-id-95d23eaf8218002ad0a6a8c4c6e6c76e6b8f785b/> Update docs, add book chapter for `lattices` crate
   - Adds `mdbook-katex` to the book build for latex support.

### New Features

 - <csr-id-15f9688ff4dc816a374ed9068d98bee0a4d51b2c/> Make lattice test helpers public, restructure
   Also impl `LatticeOrd` for `SetUnion`

### Style

 - <csr-id-cd0a86d9271d0e3daab59c46f079925f863424e1/> Warn lint `unused_qualifications`
 - <csr-id-20a1b2c0cd04a8b495a02ce345db3d48a99ea0e9/> rustfmt group imports
 - <csr-id-1eda91a2ef8794711ef037240f15284e8085d863/> rustfmt prescribe flat-module `use` format

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 14 commits contributed to the release over the course of 17 calendar days.
 - 17 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 10 unique issues were worked on: [#625](https://github.com/hydro-project/hydroflow/issues/625), [#637](https://github.com/hydro-project/hydroflow/issues/637), [#638](https://github.com/hydro-project/hydroflow/issues/638), [#642](https://github.com/hydro-project/hydroflow/issues/642), [#644](https://github.com/hydro-project/hydroflow/issues/644), [#645](https://github.com/hydro-project/hydroflow/issues/645), [#658](https://github.com/hydro-project/hydroflow/issues/658), [#660](https://github.com/hydro-project/hydroflow/issues/660), [#664](https://github.com/hydro-project/hydroflow/issues/664), [#667](https://github.com/hydro-project/hydroflow/issues/667)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#625](https://github.com/hydro-project/hydroflow/issues/625)**
    - Use `cc-traits` instead of own `Collection`, remove `tag` indirection ([`10ed00d`](https://github.com/hydro-project/hydroflow/commit/10ed00df8e6f2e86d7db737dd2049f2c5dbfeba0))
 * **[#637](https://github.com/hydro-project/hydroflow/issues/637)**
    - Add bottom and fake lattice types ([`95ce1ed`](https://github.com/hydro-project/hydroflow/commit/95ce1edbecebdc505893b1f527bef905b4247ed8))
 * **[#638](https://github.com/hydro-project/hydroflow/issues/638)**
    - Remove old lattice code ([`f4915fa`](https://github.com/hydro-project/hydroflow/commit/f4915fab98c57652e5345d39076d95ebb0a43fd8))
 * **[#642](https://github.com/hydro-project/hydroflow/issues/642)**
    - Remove zmq, use unsync channels locally, use sync mpsc cross-thread, use cross_join+enumerate instead of broadcast channel,remove Eq requirement from multisetjoin ([`b38f5cf`](https://github.com/hydro-project/hydroflow/commit/b38f5cf198e29a8de2f84eb4cd075818fbeffda6))
 * **[#644](https://github.com/hydro-project/hydroflow/issues/644)**
    - Remove Compare trait, add tests, make all lattice types PartialOrd, Eq, PartialEq ([`698b72f`](https://github.com/hydro-project/hydroflow/commit/698b72f8f013288f211a655bf93f2a3cd6d386e7))
 * **[#645](https://github.com/hydro-project/hydroflow/issues/645)**
    - Fix `Pair` `PartialOrd` implementation, add consistency tests with `NaiveOrd` ([`76e19a7`](https://github.com/hydro-project/hydroflow/commit/76e19a7346cd6005a04c39974bbcf4ed2bd37217))
 * **[#658](https://github.com/hydro-project/hydroflow/issues/658)**
    - Allow fake to merge, compare equal values ([`1a159dc`](https://github.com/hydro-project/hydroflow/commit/1a159dc8a16e40fa40ae3e8433d53c08d82e833c))
 * **[#660](https://github.com/hydro-project/hydroflow/issues/660)**
    - Warn lint `unused_qualifications` ([`cd0a86d`](https://github.com/hydro-project/hydroflow/commit/cd0a86d9271d0e3daab59c46f079925f863424e1))
    - Rustfmt group imports ([`20a1b2c`](https://github.com/hydro-project/hydroflow/commit/20a1b2c0cd04a8b495a02ce345db3d48a99ea0e9))
    - Rustfmt prescribe flat-module `use` format ([`1eda91a`](https://github.com/hydro-project/hydroflow/commit/1eda91a2ef8794711ef037240f15284e8085d863))
 * **[#664](https://github.com/hydro-project/hydroflow/issues/664)**
    - Make lattice test helpers public, restructure ([`15f9688`](https://github.com/hydro-project/hydroflow/commit/15f9688ff4dc816a374ed9068d98bee0a4d51b2c))
 * **[#667](https://github.com/hydro-project/hydroflow/issues/667)**
    - Bump lattices version to `0.1.0` ([`a46ce4a`](https://github.com/hydro-project/hydroflow/commit/a46ce4a522b70661e5acf644f893bfdf56294578))
    - Update docs, add book chapter for `lattices` crate ([`95d23ea`](https://github.com/hydro-project/hydroflow/commit/95d23eaf8218002ad0a6a8c4c6e6c76e6b8f785b))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.0.1, hydroflow_lang v0.0.1, hydroflow_datalog_core v0.0.1, hydroflow_datalog v0.0.1, hydroflow_macro v0.0.1, lattices v0.1.0, variadics v0.0.2, pusherator v0.0.1, hydroflow v0.0.2 ([`809395a`](https://github.com/hydro-project/hydroflow/commit/809395acddb78949d7a2bf036e1a94972f23b1ad))
</details>

<csr-unknown>
Update mdbook-* plugins.Moves most lattice implementations to the top level of the crateto eliminate redundant documentation.<csr-unknown/>

## 0.0.0 (2023-05-03)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#634](https://github.com/hydro-project/hydroflow/issues/634), [#636](https://github.com/hydro-project/hydroflow/issues/636)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#634](https://github.com/hydro-project/hydroflow/issues/634)**
    - Fixup! Move lattice2 into new separate `lattices` crate ([`b08e793`](https://github.com/hydro-project/hydroflow/commit/b08e793edf30e40f21c916ae630a09337034150e))
    - Move lattice2 into new separate `lattices` crate ([`c0006c4`](https://github.com/hydro-project/hydroflow/commit/c0006c4c73e0f3f5c65274e3ad76537ea9fe2643))
 * **[#636](https://github.com/hydro-project/hydroflow/issues/636)**
    - Fixup! Move lattice2 into new separate `lattices` crate ([`b08e793`](https://github.com/hydro-project/hydroflow/commit/b08e793edf30e40f21c916ae630a09337034150e))
</details>

