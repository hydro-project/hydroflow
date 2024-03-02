# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.2.0 (2024-03-02)

### New Features

 - <csr-id-eb34ccd13f56e1d07cbae35ead79daeb3b9bad20/> use an IR before lowering to Hydroflow
   Makes it possible to write custom optimization passes.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release over the course of 2 calendar days.
 - 32 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#1070](https://github.com/hydro-project/hydroflow/issues/1070)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1070](https://github.com/hydro-project/hydroflow/issues/1070)**
    - Use an IR before lowering to Hydroflow ([`eb34ccd`](https://github.com/hydro-project/hydroflow/commit/eb34ccd13f56e1d07cbae35ead79daeb3b9bad20))
</details>

## v0.1.0 (2024-01-29)

### Documentation

 - <csr-id-3b36020d16792f26da4df3c5b09652a4ab47ec4f/> actually committing empty CHANGELOG.md is required

### New Features

 - <csr-id-af6e3be60fdb69ceec1613347910f4dd49980d34/> push down persists and implement Pi example
   Also fixes type inference issues with reduce the same way as we did for fold.
 - <csr-id-174607d12277d7544d0f42890c9a5da2ff184df4/> support building graphs for symmetric clusters in Hydroflow+
 - <csr-id-71083233afc01e0132d7186f4af8c0b4a6323ec7/> support crates that have no entrypoints
   Also includes various bugfixes needed for Hydroflow+.
 - <csr-id-8b635683e5ac3c4ed2d896ae88e2953db1c6312c/> add a functional surface syntax using staging

### Bug Fixes

 - <csr-id-8df66f8c24127d8818d64d1534bb1ab4a616597f/> fix `include!` path separators on windows

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 76 calendar days.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 5 unique issues were worked on: [#1010](https://github.com/hydro-project/hydroflow/issues/1010), [#1021](https://github.com/hydro-project/hydroflow/issues/1021), [#899](https://github.com/hydro-project/hydroflow/issues/899), [#983](https://github.com/hydro-project/hydroflow/issues/983), [#984](https://github.com/hydro-project/hydroflow/issues/984)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1010](https://github.com/hydro-project/hydroflow/issues/1010)**
    - Fix `include!` path separators on windows ([`8df66f8`](https://github.com/hydro-project/hydroflow/commit/8df66f8c24127d8818d64d1534bb1ab4a616597f))
 * **[#1021](https://github.com/hydro-project/hydroflow/issues/1021)**
    - Push down persists and implement Pi example ([`af6e3be`](https://github.com/hydro-project/hydroflow/commit/af6e3be60fdb69ceec1613347910f4dd49980d34))
 * **[#899](https://github.com/hydro-project/hydroflow/issues/899)**
    - Add a functional surface syntax using staging ([`8b63568`](https://github.com/hydro-project/hydroflow/commit/8b635683e5ac3c4ed2d896ae88e2953db1c6312c))
 * **[#983](https://github.com/hydro-project/hydroflow/issues/983)**
    - Support crates that have no entrypoints ([`7108323`](https://github.com/hydro-project/hydroflow/commit/71083233afc01e0132d7186f4af8c0b4a6323ec7))
 * **[#984](https://github.com/hydro-project/hydroflow/issues/984)**
    - Support building graphs for symmetric clusters in Hydroflow+ ([`174607d`](https://github.com/hydro-project/hydroflow/commit/174607d12277d7544d0f42890c9a5da2ff184df4))
 * **Uncategorized**
    - Release stageleft_macro v0.1.0, stageleft v0.1.0, hydroflow_plus v0.5.1 ([`1a48db5`](https://github.com/hydro-project/hydroflow/commit/1a48db5a1ba058a718ac777367bf6eba3a236b7c))
    - Actually committing empty CHANGELOG.md is required ([`3b36020`](https://github.com/hydro-project/hydroflow/commit/3b36020d16792f26da4df3c5b09652a4ab47ec4f))
</details>

