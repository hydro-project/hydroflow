# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.5.0 (2023-10-11)

### Chore

 - <csr-id-e788989737fbd501173bc99c6f9f5f5ba514ec9c/> Fix `clippy::implied_bounds_in_impls` from latest nightlies

### Documentation

 - <csr-id-6b82126347e2ae3c11cc10fea4f3fbcb463734e6/> fix lattice math link

### New Features

 - <csr-id-488d6dd448e10e2bf217693dd2a29973488c838a/> Add serde derives to collections
 - <csr-id-35c2606f2df16a428a5c163d5582923ecd5998c4/> Add `UnionFind` lattice

### Bug Fixes (BREAKING)

 - <csr-id-18e9cfaa8b1415d72d67a69d7b0fecc997b5670a/> fix some types and semantics for atomization

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 40 calendar days.
 - 56 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#915](https://github.com/hydro-project/hydroflow/issues/915), [#922](https://github.com/hydro-project/hydroflow/issues/922)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#915](https://github.com/hydro-project/hydroflow/issues/915)**
    - Add `UnionFind` lattice ([`35c2606`](https://github.com/hydro-project/hydroflow/commit/35c2606f2df16a428a5c163d5582923ecd5998c4))
    - Fix some types and semantics for atomization ([`18e9cfa`](https://github.com/hydro-project/hydroflow/commit/18e9cfaa8b1415d72d67a69d7b0fecc997b5670a))
 * **[#922](https://github.com/hydro-project/hydroflow/issues/922)**
    - Add serde derives to collections ([`488d6dd`](https://github.com/hydro-project/hydroflow/commit/488d6dd448e10e2bf217693dd2a29973488c838a))
 * **Uncategorized**
    - Fix lattice math link ([`6b82126`](https://github.com/hydro-project/hydroflow/commit/6b82126347e2ae3c11cc10fea4f3fbcb463734e6))
    - Fix `clippy::implied_bounds_in_impls` from latest nightlies ([`e788989`](https://github.com/hydro-project/hydroflow/commit/e788989737fbd501173bc99c6f9f5f5ba514ec9c))
</details>

## 0.4.0 (2023-08-15)

<csr-id-f60053f70da3071c54de4a0eabb059a143aa2ccc/>
<csr-id-6a2ad6b770c2ccf470548320d8753025b3a66c0a/>
<csr-id-262166e7cecf8ffb5a2c7bc989e8cf66c4524a68/>
<csr-id-7b0485b20939ec86ed8e74ecc9c75ac1b5d01072/>

### Chore

 - <csr-id-f60053f70da3071c54de4a0eabb059a143aa2ccc/> fix lint, format errors for latest nightly version (without updated pinned)
   For nightly version (d9c13cd45 2023-07-05)

### Documentation

 - <csr-id-a8b0d2d10eef3e45669f77a1f2460cd31a95d15b/> Improve `Atomize` docs

### New Features

 - <csr-id-7282457e383407eabbeb1f931c130edb095c33ca/> formalize `Default::default()` as returning bottom for lattice types
   Not a breaking change since changed names were introduced only since last release
 - <csr-id-b2406994a703f028724cc30065fec60f7f8a7247/> Implement `SimpleKeyedRef` for map types
 - <csr-id-8ec75c6d8998b7d7e5a0ae24ee53b0cdb6932683/> Add atomize trait, impls, tests

### Refactor

 - <csr-id-6a2ad6b770c2ccf470548320d8753025b3a66c0a/> fix new clippy lints on latest nightly 1.73.0-nightly (db7ff98a7 2023-07-31)
 - <csr-id-262166e7cecf8ffb5a2c7bc989e8cf66c4524a68/> Change `Atomize` to require returning empty iff lattice is bottom
   Previously was the opposite, `Atomize` always had to return non-empty.
   
   Not breaking since `Atomize` has not yet been published.

### New Features (BREAKING)

 - <csr-id-7b752f743cbedc632b127dddf3f9a84e839eb47a/> Add bottom (+top) collapsing, implement `IsBot`/`IsTop` for all lattice types
   * `WithBot(Some(BOTTOM))` and `WithBot(None)` are now considered to both be bottom, equal. Also, `MapUnion({})` and `MapUnion({key: BOTTOM})` are considered to both be bottom, equal.
* `WithTop(Some(TOP))` and `WithTop(None)` are now considered to both be top, equal.
* `check_lattice_bot/top` now check that `is_bot` and `is_top` must be consistent among all equal elements

### Refactor (BREAKING)

 - <csr-id-7b0485b20939ec86ed8e74ecc9c75ac1b5d01072/> Rename `Seq` -> `VecUnion`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 39 calendar days.
 - 42 days passed between releases.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 8 unique issues were worked on: [#822](https://github.com/hydro-project/hydroflow/issues/822), [#849](https://github.com/hydro-project/hydroflow/issues/849), [#854](https://github.com/hydro-project/hydroflow/issues/854), [#860](https://github.com/hydro-project/hydroflow/issues/860), [#865](https://github.com/hydro-project/hydroflow/issues/865), [#866](https://github.com/hydro-project/hydroflow/issues/866), [#867](https://github.com/hydro-project/hydroflow/issues/867), [#879](https://github.com/hydro-project/hydroflow/issues/879)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#822](https://github.com/hydro-project/hydroflow/issues/822)**
    - Fix lint, format errors for latest nightly version (without updated pinned) ([`f60053f`](https://github.com/hydro-project/hydroflow/commit/f60053f70da3071c54de4a0eabb059a143aa2ccc))
 * **[#849](https://github.com/hydro-project/hydroflow/issues/849)**
    - Rename `Seq` -> `VecUnion` ([`7b0485b`](https://github.com/hydro-project/hydroflow/commit/7b0485b20939ec86ed8e74ecc9c75ac1b5d01072))
 * **[#854](https://github.com/hydro-project/hydroflow/issues/854)**
    - Add atomize trait, impls, tests ([`8ec75c6`](https://github.com/hydro-project/hydroflow/commit/8ec75c6d8998b7d7e5a0ae24ee53b0cdb6932683))
 * **[#860](https://github.com/hydro-project/hydroflow/issues/860)**
    - Improve `Atomize` docs ([`a8b0d2d`](https://github.com/hydro-project/hydroflow/commit/a8b0d2d10eef3e45669f77a1f2460cd31a95d15b))
 * **[#865](https://github.com/hydro-project/hydroflow/issues/865)**
    - Add bottom (+top) collapsing, implement `IsBot`/`IsTop` for all lattice types ([`7b752f7`](https://github.com/hydro-project/hydroflow/commit/7b752f743cbedc632b127dddf3f9a84e839eb47a))
 * **[#866](https://github.com/hydro-project/hydroflow/issues/866)**
    - Implement `SimpleKeyedRef` for map types ([`b240699`](https://github.com/hydro-project/hydroflow/commit/b2406994a703f028724cc30065fec60f7f8a7247))
 * **[#867](https://github.com/hydro-project/hydroflow/issues/867)**
    - Change `Atomize` to require returning empty iff lattice is bottom ([`262166e`](https://github.com/hydro-project/hydroflow/commit/262166e7cecf8ffb5a2c7bc989e8cf66c4524a68))
 * **[#879](https://github.com/hydro-project/hydroflow/issues/879)**
    - Formalize `Default::default()` as returning bottom for lattice types ([`7282457`](https://github.com/hydro-project/hydroflow/commit/7282457e383407eabbeb1f931c130edb095c33ca))
 * **Uncategorized**
    - Release hydroflow_lang v0.4.0, hydroflow_datalog_core v0.4.0, hydroflow_datalog v0.4.0, hydroflow_macro v0.4.0, lattices v0.4.0, pusherator v0.0.3, hydroflow v0.4.0, hydro_cli v0.4.0, safety bump 4 crates ([`cb313f0`](https://github.com/hydro-project/hydroflow/commit/cb313f0635214460a8308d05cbef4bf7f4bfaa15))
    - Fix new clippy lints on latest nightly 1.73.0-nightly (db7ff98a7 2023-07-31) ([`6a2ad6b`](https://github.com/hydro-project/hydroflow/commit/6a2ad6b770c2ccf470548320d8753025b3a66c0a))
</details>

## 0.3.0 (2023-07-04)

<csr-id-0cbbaeaec5e192e2539771bb247926271c2dc4a3/>
<csr-id-70c88a51c4c83a4dc2fc67a0cd344786a4ff26f7/>
<csr-id-4a727ecf1232e0f03f5300547282bfbe73342cfa/>
<csr-id-5c7e4d3aea1dfb61d51bcb0291740281824e3090/>
<csr-id-1bdadb82b25941d11f3fa24eaac35109927c852f/>

### Documentation

 - <csr-id-ac4fd827ccede0ad53dfc59079cdb7df5928e491/> List `WithTop` in README 4/4

### New Features

 - <csr-id-016abeea3ecd390a976dd8dbec371b08fe744655/> make unit `()` a point lattice
 - <csr-id-dc99c021640a47b704905d087eadcbc477f033f0/> impl `IsTop`, `IsBot` for `Min`, `Max` over numeric types
 - <csr-id-f5e0d19e8531c250bc4492b61b9731c947916daf/> Add `Conflict<T>` lattice
 - <csr-id-fc4dcbdfa703d79a0c183a2eb3f5dbb42260b67a/> add top lattice, opposite of bottom
 - <csr-id-153cbabd462d776eae395e371470abb4662642cd/> Add `Seq` lattice.

### Bug Fixes

 - <csr-id-9bb5528d99e83fdae5aeca9456802379131c2f90/> removed unused nightly features `impl_trait_in_assoc_type`, `type_alias_impl_trait`
 - <csr-id-3c4eb16833160f8813b812487a1297c023400138/> fix ConvertFrom for bottom to actually convert the type
   * fix: fix type inference with doubly-nested bottom types
* fix: address comments

### Refactor

 - <csr-id-0cbbaeaec5e192e2539771bb247926271c2dc4a3/> Rename `bottom.rs` -> `with_bot.rs`, `top.rs` -> `with_top.rs` 1/4

### Style

 - <csr-id-70c88a51c4c83a4dc2fc67a0cd344786a4ff26f7/> `warn` missing docs (instead of `deny`) to allow code before docs

### New Features (BREAKING)

<csr-id-deb26af6bcd547f91bf339367387d36e5e59565a/>

 - <csr-id-931d93887c238025596cb22226e16d43e16a7425/> Add `reveal` methods, make fields private
 - <csr-id-7aec1ac884e01a560770dfab7e0ba64d520415f6/> Add `Provenance` generic param token to `Point`.
   - Use `()` provenance for `kvs_bench` example.

### Bug Fixes (BREAKING)

 - <csr-id-5cfd2a0f48f11f6185070cab932f50b630e1f800/> Remove `Default` impl for `WithTop` 3/4
   Is confusing, probably not what users want.

### Refactor (BREAKING)

 - <csr-id-4a727ecf1232e0f03f5300547282bfbe73342cfa/> Rename `ConvertFrom::from` -> `LatticeFrom::lattice_from`
 - <csr-id-5c7e4d3aea1dfb61d51bcb0291740281824e3090/> Rename `Bottom` -> `WithBot`, `Top` -> `WithTop`, constructors now take `Option`s 2/4
 - <csr-id-1bdadb82b25941d11f3fa24eaac35109927c852f/> Rename `Immut` -> `Point` lattice.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 18 commits contributed to the release over the course of 31 calendar days.
 - 33 days passed between releases.
 - 17 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 12 unique issues were worked on: [#742](https://github.com/hydro-project/hydroflow/issues/742), [#744](https://github.com/hydro-project/hydroflow/issues/744), [#761](https://github.com/hydro-project/hydroflow/issues/761), [#763](https://github.com/hydro-project/hydroflow/issues/763), [#765](https://github.com/hydro-project/hydroflow/issues/765), [#766](https://github.com/hydro-project/hydroflow/issues/766), [#767](https://github.com/hydro-project/hydroflow/issues/767), [#772](https://github.com/hydro-project/hydroflow/issues/772), [#773](https://github.com/hydro-project/hydroflow/issues/773), [#780](https://github.com/hydro-project/hydroflow/issues/780), [#789](https://github.com/hydro-project/hydroflow/issues/789), [#793](https://github.com/hydro-project/hydroflow/issues/793)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#742](https://github.com/hydro-project/hydroflow/issues/742)**
    - Fix ConvertFrom for bottom to actually convert the type ([`3c4eb16`](https://github.com/hydro-project/hydroflow/commit/3c4eb16833160f8813b812487a1297c023400138))
 * **[#744](https://github.com/hydro-project/hydroflow/issues/744)**
    - Add top lattice, opposite of bottom ([`fc4dcbd`](https://github.com/hydro-project/hydroflow/commit/fc4dcbdfa703d79a0c183a2eb3f5dbb42260b67a))
 * **[#761](https://github.com/hydro-project/hydroflow/issues/761)**
    - Rename `Immut` -> `Point` lattice. ([`1bdadb8`](https://github.com/hydro-project/hydroflow/commit/1bdadb82b25941d11f3fa24eaac35109927c852f))
 * **[#763](https://github.com/hydro-project/hydroflow/issues/763)**
    - List `WithTop` in README 4/4 ([`ac4fd82`](https://github.com/hydro-project/hydroflow/commit/ac4fd827ccede0ad53dfc59079cdb7df5928e491))
    - Remove `Default` impl for `WithTop` 3/4 ([`5cfd2a0`](https://github.com/hydro-project/hydroflow/commit/5cfd2a0f48f11f6185070cab932f50b630e1f800))
    - Rename `Bottom` -> `WithBot`, `Top` -> `WithTop`, constructors now take `Option`s 2/4 ([`5c7e4d3`](https://github.com/hydro-project/hydroflow/commit/5c7e4d3aea1dfb61d51bcb0291740281824e3090))
    - Rename `bottom.rs` -> `with_bot.rs`, `top.rs` -> `with_top.rs` 1/4 ([`0cbbaea`](https://github.com/hydro-project/hydroflow/commit/0cbbaeaec5e192e2539771bb247926271c2dc4a3))
 * **[#765](https://github.com/hydro-project/hydroflow/issues/765)**
    - Rename `ConvertFrom::from` -> `LatticeFrom::lattice_from` ([`4a727ec`](https://github.com/hydro-project/hydroflow/commit/4a727ecf1232e0f03f5300547282bfbe73342cfa))
 * **[#766](https://github.com/hydro-project/hydroflow/issues/766)**
    - Add `IsBot::is_bot` and `IsTop::is_top` traits ([`deb26af`](https://github.com/hydro-project/hydroflow/commit/deb26af6bcd547f91bf339367387d36e5e59565a))
 * **[#767](https://github.com/hydro-project/hydroflow/issues/767)**
    - Add `Conflict<T>` lattice ([`f5e0d19`](https://github.com/hydro-project/hydroflow/commit/f5e0d19e8531c250bc4492b61b9731c947916daf))
 * **[#772](https://github.com/hydro-project/hydroflow/issues/772)**
    - Add `Provenance` generic param token to `Point`. ([`7aec1ac`](https://github.com/hydro-project/hydroflow/commit/7aec1ac884e01a560770dfab7e0ba64d520415f6))
 * **[#773](https://github.com/hydro-project/hydroflow/issues/773)**
    - `warn` missing docs (instead of `deny`) to allow code before docs ([`70c88a5`](https://github.com/hydro-project/hydroflow/commit/70c88a51c4c83a4dc2fc67a0cd344786a4ff26f7))
 * **[#780](https://github.com/hydro-project/hydroflow/issues/780)**
    - Removed unused nightly features `impl_trait_in_assoc_type`, `type_alias_impl_trait` ([`9bb5528`](https://github.com/hydro-project/hydroflow/commit/9bb5528d99e83fdae5aeca9456802379131c2f90))
 * **[#789](https://github.com/hydro-project/hydroflow/issues/789)**
    - Add `reveal` methods, make fields private ([`931d938`](https://github.com/hydro-project/hydroflow/commit/931d93887c238025596cb22226e16d43e16a7425))
 * **[#793](https://github.com/hydro-project/hydroflow/issues/793)**
    - Make unit `()` a point lattice ([`016abee`](https://github.com/hydro-project/hydroflow/commit/016abeea3ecd390a976dd8dbec371b08fe744655))
    - Impl `IsTop`, `IsBot` for `Min`, `Max` over numeric types ([`dc99c02`](https://github.com/hydro-project/hydroflow/commit/dc99c021640a47b704905d087eadcbc477f033f0))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.3.0, hydroflow_lang v0.3.0, hydroflow_datalog_core v0.3.0, hydroflow_datalog v0.3.0, hydroflow_macro v0.3.0, lattices v0.3.0, pusherator v0.0.2, hydroflow v0.3.0, hydro_cli v0.3.0, safety bump 5 crates ([`ec9633e`](https://github.com/hydro-project/hydroflow/commit/ec9633e2e393c2bf106223abeb0b680200fbdf84))
    - Add `Seq` lattice. ([`153cbab`](https://github.com/hydro-project/hydroflow/commit/153cbabd462d776eae395e371470abb4662642cd))
</details>

## 0.2.0 (2023-05-31)

<csr-id-fd896fbe925fbd8ef1d16be7206ac20ba585081a/>
<csr-id-10b308532245db8f4480ce53b67aea050ae1918d/>

### Chore

 - <csr-id-fd896fbe925fbd8ef1d16be7206ac20ba585081a/> manually bump versions for v0.2.0 release

### Refactor (BREAKING)

 - <csr-id-10b308532245db8f4480ce53b67aea050ae1918d/> rename `Fake` -> `Immut`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 day passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release hydroflow_lang v0.2.0, hydroflow_datalog_core v0.2.0, hydroflow_datalog v0.2.0, hydroflow_macro v0.2.0, lattices v0.2.0, hydroflow v0.2.0, hydro_cli v0.2.0 ([`ca464c3`](https://github.com/hydro-project/hydroflow/commit/ca464c32322a7ad39eb53e1794777c849aa548a0))
    - Manually bump versions for v0.2.0 release ([`fd896fb`](https://github.com/hydro-project/hydroflow/commit/fd896fbe925fbd8ef1d16be7206ac20ba585081a))
    - Rename `Fake` -> `Immut` ([`10b3085`](https://github.com/hydro-project/hydroflow/commit/10b308532245db8f4480ce53b67aea050ae1918d))
</details>

## 0.1.2 (2023-05-30)

### New Features

 - <csr-id-ecff609a0153446efc1809230ae100964bb9f89b/> print out items when lattice identity tests fail

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 5 calendar days.
 - 6 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#691](https://github.com/hydro-project/hydroflow/issues/691)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#691](https://github.com/hydro-project/hydroflow/issues/691)**
    - Print out items when lattice identity tests fail ([`ecff609`](https://github.com/hydro-project/hydroflow/commit/ecff609a0153446efc1809230ae100964bb9f89b))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.1.1, hydroflow_lang v0.1.1, hydroflow_datalog_core v0.1.1, hydroflow_macro v0.1.1, lattices v0.1.2, hydroflow v0.1.1, hydro_cli v0.1.0 ([`d9fa8b3`](https://github.com/hydro-project/hydroflow/commit/d9fa8b387e303b33d9614dbde80abf1af08bd8eb))
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

