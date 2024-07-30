# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.0.5 (2024-07-23)

### New Features

 - <csr-id-20080cb7ceb5b5d3ba349dfd822a37288e40add6/> add traits for dealing with variadics of references
   Renames some traits, but not a breaking change since there hasn't been a
   release that includes those traits.
 - <csr-id-b92dfc7460c985db6935e79d612f42b9b87e746f/> add `iter_any_ref` and `iter_any_mut` to `VariadicsExt`
   Depends on #1241
   
   This isn't needed for the current GHT implementation, but is useful in
   general
 - <csr-id-1a6228f2db081af68890e2e64b3a91f15dd9214f/> add traits for referencing variadics
   This adds a way to convert a reference to a variadic into a variadic of
   references. I.e. `&var_expr!(a, b, c) -> var_expr!(&a, &b, &c)`

### Bug Fixes

 - <csr-id-bbef0705d509831415d3bb5ce003116af06b6ffb/> `EitherRefVariadic` is `Variadic`
 - <csr-id-c70114d836e5bc36e2104188867e548e90ab38f4/> fix `HomogenousVariadic` `get` and `get_mut` only returning `None`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 53 calendar days.
 - 143 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 5 unique issues were worked on: [#1241](https://github.com/hydro-project/hydroflow/issues/1241), [#1245](https://github.com/hydro-project/hydroflow/issues/1245), [#1324](https://github.com/hydro-project/hydroflow/issues/1324), [#1325](https://github.com/hydro-project/hydroflow/issues/1325), [#1352](https://github.com/hydro-project/hydroflow/issues/1352)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1241](https://github.com/hydro-project/hydroflow/issues/1241)**
    - Add traits for referencing variadics ([`1a6228f`](https://github.com/hydro-project/hydroflow/commit/1a6228f2db081af68890e2e64b3a91f15dd9214f))
 * **[#1245](https://github.com/hydro-project/hydroflow/issues/1245)**
    - Add `iter_any_ref` and `iter_any_mut` to `VariadicsExt` ([`b92dfc7`](https://github.com/hydro-project/hydroflow/commit/b92dfc7460c985db6935e79d612f42b9b87e746f))
 * **[#1324](https://github.com/hydro-project/hydroflow/issues/1324)**
    - Add traits for dealing with variadics of references ([`20080cb`](https://github.com/hydro-project/hydroflow/commit/20080cb7ceb5b5d3ba349dfd822a37288e40add6))
 * **[#1325](https://github.com/hydro-project/hydroflow/issues/1325)**
    - Fix `HomogenousVariadic` `get` and `get_mut` only returning `None` ([`c70114d`](https://github.com/hydro-project/hydroflow/commit/c70114d836e5bc36e2104188867e548e90ab38f4))
 * **[#1352](https://github.com/hydro-project/hydroflow/issues/1352)**
    - `EitherRefVariadic` is `Variadic` ([`bbef070`](https://github.com/hydro-project/hydroflow/commit/bbef0705d509831415d3bb5ce003116af06b6ffb))
</details>

## 0.0.4 (2024-03-02)

<csr-id-5a451ac4ae75024153a06416fc81d834d1fdae6f/>
<csr-id-7103e77d0da1d73f1c93fcdb260b6a4c9a18ff66/>
<csr-id-b4683450a273d510a11338f07920a5558033b31f/>

### Chore

 - <csr-id-5a451ac4ae75024153a06416fc81d834d1fdae6f/> prep for 0.0.4 release
 - <csr-id-7103e77d0da1d73f1c93fcdb260b6a4c9a18ff66/> update pinned rust to 2024-04-24

### Style

 - <csr-id-b4683450a273d510a11338f07920a5558033b31f/> fix dead code lint

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 3 calendar days.
 - 31 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release hydroflow_lang v0.6.0, hydroflow_datalog_core v0.6.0, hydroflow_datalog v0.6.0, hydroflow_macro v0.6.0, lattices v0.5.3, variadics v0.0.4, pusherator v0.0.5, hydroflow v0.6.0, stageleft v0.2.0, hydroflow_plus v0.6.0, hydro_deploy v0.6.0, hydro_cli v0.6.0, hydroflow_plus_cli_integration v0.6.0, safety bump 7 crates ([`09ea65f`](https://github.com/hydro-project/hydroflow/commit/09ea65fe9cd45c357c43bffca30e60243fa45cc8))
    - Prep for 0.0.4 release ([`5a451ac`](https://github.com/hydro-project/hydroflow/commit/5a451ac4ae75024153a06416fc81d834d1fdae6f))
    - Fix dead code lint ([`b468345`](https://github.com/hydro-project/hydroflow/commit/b4683450a273d510a11338f07920a5558033b31f))
    - Update pinned rust to 2024-04-24 ([`7103e77`](https://github.com/hydro-project/hydroflow/commit/7103e77d0da1d73f1c93fcdb260b6a4c9a18ff66))
</details>

## 0.0.3 (2024-01-29)

<csr-id-1b555e57c8c812bed4d6495d2960cbf77fb0b3ef/>
<csr-id-7e65a08711775656e435e854777c5f089dd31a05/>

### Chore

 - <csr-id-1b555e57c8c812bed4d6495d2960cbf77fb0b3ef/> manually set lockstep-versioned crates (and `lattices`) to version `0.5.1`
   Setting manually since
   https://github.com/frewsxcv/rust-crates-index/issues/159 is messing with
   smart-release

### Refactor

 - <csr-id-7e65a08711775656e435e854777c5f089dd31a05/> Improvements prepping for release
   - Adds the "spread"/"splat" `...` syntax to the three variadics macros.
   - Adds `#[sealed]` traits.
   - Adds testing of error messages.
   - Improves docs: `README.md` and Rust docs.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 37 calendar days.
 - 253 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#974](https://github.com/hydro-project/hydroflow/issues/974)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#974](https://github.com/hydro-project/hydroflow/issues/974)**
    - Improvements prepping for release ([`7e65a08`](https://github.com/hydro-project/hydroflow/commit/7e65a08711775656e435e854777c5f089dd31a05))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.5.1, hydroflow_lang v0.5.1, hydroflow_datalog_core v0.5.1, hydroflow_datalog v0.5.1, hydroflow_macro v0.5.1, lattices v0.5.1, variadics v0.0.3, pusherator v0.0.4, hydroflow v0.5.1, stageleft_macro v0.1.0, stageleft v0.1.0, hydroflow_plus v0.5.1, hydro_deploy v0.5.1, hydro_cli v0.5.1 ([`478aebc`](https://github.com/hydro-project/hydroflow/commit/478aebc8fee2aa78eab86bd386322db1c70bde6a))
    - Manually set lockstep-versioned crates (and `lattices`) to version `0.5.1` ([`1b555e5`](https://github.com/hydro-project/hydroflow/commit/1b555e57c8c812bed4d6495d2960cbf77fb0b3ef))
</details>

## 0.0.2 (2023-05-21)

<csr-id-5a3c2949653685de1e33cf7412057a70880283df/>

### Style

 - <csr-id-5a3c2949653685de1e33cf7412057a70880283df/> rustfmt format code comments

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 24 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#660](https://github.com/hydro-project/hydroflow/issues/660)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#660](https://github.com/hydro-project/hydroflow/issues/660)**
    - Rustfmt format code comments ([`5a3c294`](https://github.com/hydro-project/hydroflow/commit/5a3c2949653685de1e33cf7412057a70880283df))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.0.1, hydroflow_lang v0.0.1, hydroflow_datalog_core v0.0.1, hydroflow_datalog v0.0.1, hydroflow_macro v0.0.1, lattices v0.1.0, variadics v0.0.2, pusherator v0.0.1, hydroflow v0.0.2 ([`809395a`](https://github.com/hydro-project/hydroflow/commit/809395acddb78949d7a2bf036e1a94972f23b1ad))
</details>

## 0.0.1 (2023-04-26)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 133 calendar days.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#617](https://github.com/hydro-project/hydroflow/issues/617)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#617](https://github.com/hydro-project/hydroflow/issues/617)**
    - Update `Cargo.toml`s for publishing ([`a78ff9a`](https://github.com/hydro-project/hydroflow/commit/a78ff9aace6771787c2b72aad83be6ad8d49a828))
 * **Uncategorized**
    - Setup release workflow ([`108d0e9`](https://github.com/hydro-project/hydroflow/commit/108d0e933a08b183c4dadf8c3499e4946696e263))
    - Rename variadics/tuple_list macros ([`91d37b0`](https://github.com/hydro-project/hydroflow/commit/91d37b022b1cd0ed590765c40ef43244027c8035))
    - Rename pkg `type_list` -> `variadics` ([`50e7361`](https://github.com/hydro-project/hydroflow/commit/50e7361709cd34fd0e1cbf0c9a9f79343ee9c2e2))
</details>

