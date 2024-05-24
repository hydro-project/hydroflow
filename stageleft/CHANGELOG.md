# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.3.0 (2024-05-24)

### New Features

 - <csr-id-93fd05e5ff256e2e0a3b513695ff869c32344447/> re-compile staged sources for the macro at the top level

### Bug Fixes

 - <csr-id-658f6483587042d9c6df2936bc58749d30b72997/> fix missing `syn` `visit-mut` feature
 - <csr-id-0cafbdb74a665412a83aa900b4eb10c00e2498dd/> handle send_bincode with local structs
   fix(hydroflow_plus): handle send_bincode with local structs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 43 calendar days.
 - 48 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#1104](https://github.com/hydro-project/hydroflow/issues/1104), [#1151](https://github.com/hydro-project/hydroflow/issues/1151), [#1225](https://github.com/hydro-project/hydroflow/issues/1225)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1104](https://github.com/hydro-project/hydroflow/issues/1104)**
    - Re-compile staged sources for the macro at the top level ([`93fd05e`](https://github.com/hydro-project/hydroflow/commit/93fd05e5ff256e2e0a3b513695ff869c32344447))
 * **[#1151](https://github.com/hydro-project/hydroflow/issues/1151)**
    - Handle send_bincode with local structs ([`0cafbdb`](https://github.com/hydro-project/hydroflow/commit/0cafbdb74a665412a83aa900b4eb10c00e2498dd))
 * **[#1225](https://github.com/hydro-project/hydroflow/issues/1225)**
    - Fix missing `syn` `visit-mut` feature ([`658f648`](https://github.com/hydro-project/hydroflow/commit/658f6483587042d9c6df2936bc58749d30b72997))
</details>

## v0.2.1 (2024-04-05)

<csr-id-7958fb0d900be8fe7359326abfa11dcb8fb35e8a/>

### New Features

 - <csr-id-77f3e5afb9e276d1d6c643574ebac75ed0003939/> simplify lifetime bounds for processes and clusters
   feat(hydroflow_plus): simplify lifetime bounds for processes and
   clusters
   
   This allows `extract` to move the flow builder, which is a prerequisite
   for having developers run the optimizer during deployment as well in
   case it changes the network topology.

### Style

 - <csr-id-7958fb0d900be8fe7359326abfa11dcb8fb35e8a/> qualified path cleanups for clippy

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 30 calendar days.
 - 34 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#1090](https://github.com/hydro-project/hydroflow/issues/1090), [#1100](https://github.com/hydro-project/hydroflow/issues/1100)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1090](https://github.com/hydro-project/hydroflow/issues/1090)**
    - Qualified path cleanups for clippy ([`7958fb0`](https://github.com/hydro-project/hydroflow/commit/7958fb0d900be8fe7359326abfa11dcb8fb35e8a))
 * **[#1100](https://github.com/hydro-project/hydroflow/issues/1100)**
    - Simplify lifetime bounds for processes and clusters ([`77f3e5a`](https://github.com/hydro-project/hydroflow/commit/77f3e5afb9e276d1d6c643574ebac75ed0003939))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.5.2, hydroflow_lang v0.6.1, hydroflow_datalog_core v0.6.1, lattices v0.5.4, hydroflow v0.6.1, stageleft_macro v0.1.1, stageleft v0.2.1, hydroflow_plus v0.6.1, hydro_deploy v0.6.1, hydro_cli v0.6.1, hydroflow_plus_cli_integration v0.6.1, stageleft_tool v0.1.1 ([`cd63f22`](https://github.com/hydro-project/hydroflow/commit/cd63f2258c961a40f0e5dbef20ac329a2d570ad0))
</details>

## v0.2.0 (2024-03-02)

### New Features

 - <csr-id-eb34ccd13f56e1d07cbae35ead79daeb3b9bad20/> use an IR before lowering to Hydroflow
   Makes it possible to write custom optimization passes.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 2 calendar days.
 - 31 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#1070](https://github.com/hydro-project/hydroflow/issues/1070)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1070](https://github.com/hydro-project/hydroflow/issues/1070)**
    - Use an IR before lowering to Hydroflow ([`eb34ccd`](https://github.com/hydro-project/hydroflow/commit/eb34ccd13f56e1d07cbae35ead79daeb3b9bad20))
 * **Uncategorized**
    - Release hydroflow_lang v0.6.0, hydroflow_datalog_core v0.6.0, hydroflow_datalog v0.6.0, hydroflow_macro v0.6.0, lattices v0.5.3, variadics v0.0.4, pusherator v0.0.5, hydroflow v0.6.0, stageleft v0.2.0, hydroflow_plus v0.6.0, hydro_deploy v0.6.0, hydro_cli v0.6.0, hydroflow_plus_cli_integration v0.6.0, safety bump 7 crates ([`09ea65f`](https://github.com/hydro-project/hydroflow/commit/09ea65fe9cd45c357c43bffca30e60243fa45cc8))
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

