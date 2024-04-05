# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.1 (2024-04-05)

### New Features

 - <csr-id-77f3e5afb9e276d1d6c643574ebac75ed0003939/> simplify lifetime bounds for processes and clusters
   feat(hydroflow_plus): simplify lifetime bounds for processes and
   clusters
   
   This allows `extract` to move the flow builder, which is a prerequisite
   for having developers run the optimizer during deployment as well in
   case it changes the network topology.

### Bug Fixes

 - <csr-id-6cc5a7f78fc8bd28f32dad58857ccd1c46bc93cd/> handle multiple references to copy-able free variables
 - <csr-id-54b4ee3fc847db14a0b825e04eae6e1288d44af3/> ignore unnecessary lifetime lint on entrypoints

### Style

 - <csr-id-7958fb0d900be8fe7359326abfa11dcb8fb35e8a/> qualified path cleanups for clippy

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 31 calendar days.
 - 67 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 4 unique issues were worked on: [#1090](https://github.com/hydro-project/hydroflow/issues/1090), [#1100](https://github.com/hydro-project/hydroflow/issues/1100), [#1117](https://github.com/hydro-project/hydroflow/issues/1117), [#1124](https://github.com/hydro-project/hydroflow/issues/1124)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1090](https://github.com/hydro-project/hydroflow/issues/1090)**
    - Qualified path cleanups for clippy ([`7958fb0`](https://github.com/hydro-project/hydroflow/commit/7958fb0d900be8fe7359326abfa11dcb8fb35e8a))
 * **[#1100](https://github.com/hydro-project/hydroflow/issues/1100)**
    - Simplify lifetime bounds for processes and clusters ([`77f3e5a`](https://github.com/hydro-project/hydroflow/commit/77f3e5afb9e276d1d6c643574ebac75ed0003939))
 * **[#1117](https://github.com/hydro-project/hydroflow/issues/1117)**
    - Ignore unnecessary lifetime lint on entrypoints ([`54b4ee3`](https://github.com/hydro-project/hydroflow/commit/54b4ee3fc847db14a0b825e04eae6e1288d44af3))
 * **[#1124](https://github.com/hydro-project/hydroflow/issues/1124)**
    - Handle multiple references to copy-able free variables ([`6cc5a7f`](https://github.com/hydro-project/hydroflow/commit/6cc5a7f78fc8bd28f32dad58857ccd1c46bc93cd))
</details>

## v0.1.0 (2024-01-29)

### Documentation

 - <csr-id-3b36020d16792f26da4df3c5b09652a4ab47ec4f/> actually committing empty CHANGELOG.md is required

### New Features

 - <csr-id-af6e3be60fdb69ceec1613347910f4dd49980d34/> push down persists and implement Pi example
   Also fixes type inference issues with reduce the same way as we did for fold.
 - <csr-id-71083233afc01e0132d7186f4af8c0b4a6323ec7/> support crates that have no entrypoints
   Also includes various bugfixes needed for Hydroflow+.
 - <csr-id-e5bdd12e32d6ea72fd91a55c12e09f07a0edaa5c/> add initial test using Hydro CLI from Hydroflow+
   This also required a change to Hydroflow core to make it possible to run the dataflow itself on a single thread (using a LocalSet), even if the surrounding runtime is not single-threaded (required to work around deadlocks because we can't use async APIs inside Hydroflow+). This requires us to spawn any Hydroflow tasks (only for `dest_sink` at the moment) right next to when we run the dataflow rather than when the Hydroflow graph is initialized. From a conceptual perspective, this seems _more right_, since now creating a Hydroflow program will not result in any actual tasks running.
   
   In the third PR of this series, I aim to add a new Hydroflow+ operator that will automate the setup of a `dest_sink`/`source_stream` pair that span nodes.
 - <csr-id-8b635683e5ac3c4ed2d896ae88e2953db1c6312c/> add a functional surface syntax using staging

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 76 calendar days.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 4 unique issues were worked on: [#1021](https://github.com/hydro-project/hydroflow/issues/1021), [#899](https://github.com/hydro-project/hydroflow/issues/899), [#978](https://github.com/hydro-project/hydroflow/issues/978), [#983](https://github.com/hydro-project/hydroflow/issues/983)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1021](https://github.com/hydro-project/hydroflow/issues/1021)**
    - Push down persists and implement Pi example ([`af6e3be`](https://github.com/hydro-project/hydroflow/commit/af6e3be60fdb69ceec1613347910f4dd49980d34))
 * **[#899](https://github.com/hydro-project/hydroflow/issues/899)**
    - Add a functional surface syntax using staging ([`8b63568`](https://github.com/hydro-project/hydroflow/commit/8b635683e5ac3c4ed2d896ae88e2953db1c6312c))
 * **[#978](https://github.com/hydro-project/hydroflow/issues/978)**
    - Add initial test using Hydro CLI from Hydroflow+ ([`e5bdd12`](https://github.com/hydro-project/hydroflow/commit/e5bdd12e32d6ea72fd91a55c12e09f07a0edaa5c))
 * **[#983](https://github.com/hydro-project/hydroflow/issues/983)**
    - Support crates that have no entrypoints ([`7108323`](https://github.com/hydro-project/hydroflow/commit/71083233afc01e0132d7186f4af8c0b4a6323ec7))
 * **Uncategorized**
    - Release stageleft_macro v0.1.0, stageleft v0.1.0, hydroflow_plus v0.5.1 ([`1a48db5`](https://github.com/hydro-project/hydroflow/commit/1a48db5a1ba058a718ac777367bf6eba3a236b7c))
    - Actually committing empty CHANGELOG.md is required ([`3b36020`](https://github.com/hydro-project/hydroflow/commit/3b36020d16792f26da4df3c5b09652a4ab47ec4f))
</details>

