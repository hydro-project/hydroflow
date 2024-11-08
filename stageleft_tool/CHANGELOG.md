

## v0.4.0 (2024-11-08)

### Chore

 - <csr-id-d5677604e93c07a5392f4229af94a0b736eca382/> update pinned rust version, clippy lints, remove some dead code

### New Features

 - <csr-id-afe78c343658472513b34d28658634b253148aee/> add ability to have staged flows inside unit tests
   Whenever a Hydroflow+ program is compiled, it depends on a generated
   `__staged` module, which contains the entire contents of the crate but
   with every type / function made `pub` and exported, so that the compiled
   UDFs can resolve local references appropriately.
   
   Previously, we would not do this for `#[cfg(test)]` modules, since they
   may use `dev-dependencies` and therefore the generated module may fail
   to compile when not in test mode. To solve this, when running a unit
   test (marked with `hydroflow_plus::deploy::init_test()`) that uses
   trybuild, we emit a version of the `__staged` module with `#[cfg(test)]`
   modules included _into the generated trybuild sources_ because we can
   guarantee via trybuild that the appropriate `dev-dependencies` are
   available.
   
   This by itself allows crates depending on `hydroflow_plus` to have local
   unit tests with Hydroflow+ logic inside them. But we also want to use
   this support for unit tests inside `hydroflow_plus` itself. To enable
   that, we eliminate the `hydroflow_plus_deploy` crate and move its
   contents directly to `hydroflow_plus` itself so that we can access the
   trybuild machinery without incurring a circular dependency.
   
   Also fixes #1408

### Bug Fixes

 - <csr-id-2faffdbf2cc886da22e496df64f46aefa380766c/> properly handle `crate::` imports

### Refactor

 - <csr-id-8b7b1c60fd33b78f9a4b0873bbbd150260ae2ad5/> complete split into leader election and sequencing phases

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 69 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 4 unique issues were worked on: [#1444](https://github.com/hydro-project/hydroflow/issues/1444), [#1450](https://github.com/hydro-project/hydroflow/issues/1450), [#1486](https://github.com/hydro-project/hydroflow/issues/1486), [#1527](https://github.com/hydro-project/hydroflow/issues/1527)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1444](https://github.com/hydro-project/hydroflow/issues/1444)**
    - Update pinned rust version, clippy lints, remove some dead code ([`d567760`](https://github.com/hydro-project/hydroflow/commit/d5677604e93c07a5392f4229af94a0b736eca382))
 * **[#1450](https://github.com/hydro-project/hydroflow/issues/1450)**
    - Add ability to have staged flows inside unit tests ([`afe78c3`](https://github.com/hydro-project/hydroflow/commit/afe78c343658472513b34d28658634b253148aee))
 * **[#1486](https://github.com/hydro-project/hydroflow/issues/1486)**
    - Complete split into leader election and sequencing phases ([`8b7b1c6`](https://github.com/hydro-project/hydroflow/commit/8b7b1c60fd33b78f9a4b0873bbbd150260ae2ad5))
 * **[#1527](https://github.com/hydro-project/hydroflow/issues/1527)**
    - Properly handle `crate::` imports ([`2faffdb`](https://github.com/hydro-project/hydroflow/commit/2faffdbf2cc886da22e496df64f46aefa380766c))
</details>

## v0.3.0 (2024-08-30)

<csr-id-11af32828bab6e4a4264d2635ff71a12bb0bb778/>

### Chore

 - <csr-id-11af32828bab6e4a4264d2635ff71a12bb0bb778/> lower min dependency versions where possible, update `Cargo.lock`
   Moved from #1418
   
   ---------

### New Features

 - <csr-id-46a8a2cb08732bb21096e824bc4542d208c68fb2/> use trybuild to compile subgraph binaries

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 97 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#1398](https://github.com/hydro-project/hydroflow/issues/1398), [#1423](https://github.com/hydro-project/hydroflow/issues/1423)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1398](https://github.com/hydro-project/hydroflow/issues/1398)**
    - Use trybuild to compile subgraph binaries ([`46a8a2c`](https://github.com/hydro-project/hydroflow/commit/46a8a2cb08732bb21096e824bc4542d208c68fb2))
 * **[#1423](https://github.com/hydro-project/hydroflow/issues/1423)**
    - Lower min dependency versions where possible, update `Cargo.lock` ([`11af328`](https://github.com/hydro-project/hydroflow/commit/11af32828bab6e4a4264d2635ff71a12bb0bb778))
 * **Uncategorized**
    - Release hydroflow_lang v0.9.0, hydroflow_datalog_core v0.9.0, hydroflow_datalog v0.9.0, hydroflow_deploy_integration v0.9.0, hydroflow_macro v0.9.0, lattices_macro v0.5.6, lattices v0.5.7, multiplatform_test v0.2.0, variadics v0.0.6, pusherator v0.0.8, hydroflow v0.9.0, stageleft_macro v0.3.0, stageleft v0.4.0, stageleft_tool v0.3.0, hydroflow_plus v0.9.0, hydro_deploy v0.9.0, hydro_cli v0.9.0, hydroflow_plus_deploy v0.9.0, safety bump 8 crates ([`0750117`](https://github.com/hydro-project/hydroflow/commit/0750117de7088c01a439b102adeb4c832889f171))
</details>

## v0.2.0 (2024-05-24)

<csr-id-b86f11aad344fef6ad9cdd1db0b45bb738c48bd6/>

### Chore

 - <csr-id-b86f11aad344fef6ad9cdd1db0b45bb738c48bd6/> expect custom config names to prevent warnings
   See
   https://doc.rust-lang.org/nightly/cargo/reference/build-scripts.html#rustc-check-cfg

### New Features

 - <csr-id-93fd05e5ff256e2e0a3b513695ff869c32344447/> re-compile staged sources for the macro at the top level

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 44 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#1104](https://github.com/hydro-project/hydroflow/issues/1104), [#1192](https://github.com/hydro-project/hydroflow/issues/1192)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1104](https://github.com/hydro-project/hydroflow/issues/1104)**
    - Re-compile staged sources for the macro at the top level ([`93fd05e`](https://github.com/hydro-project/hydroflow/commit/93fd05e5ff256e2e0a3b513695ff869c32344447))
 * **[#1192](https://github.com/hydro-project/hydroflow/issues/1192)**
    - Expect custom config names to prevent warnings ([`b86f11a`](https://github.com/hydro-project/hydroflow/commit/b86f11aad344fef6ad9cdd1db0b45bb738c48bd6))
 * **Uncategorized**
    - Release hydroflow_lang v0.7.0, hydroflow_datalog_core v0.7.0, hydroflow_datalog v0.7.0, hydroflow_macro v0.7.0, lattices v0.5.5, multiplatform_test v0.1.0, pusherator v0.0.6, hydroflow v0.7.0, stageleft_macro v0.2.0, stageleft v0.3.0, stageleft_tool v0.2.0, hydroflow_plus v0.7.0, hydro_deploy v0.7.0, hydro_cli v0.7.0, hydroflow_plus_cli_integration v0.7.0, safety bump 8 crates ([`2852147`](https://github.com/hydro-project/hydroflow/commit/285214740627685e911781793e05d234ab2ad2bd))
</details>

## v0.1.1 (2024-04-09)

<csr-id-fc447ffdf8fd1b2189545a991f08588238182f00/>
<csr-id-7958fb0d900be8fe7359326abfa11dcb8fb35e8a/>

### Chore

 - <csr-id-fc447ffdf8fd1b2189545a991f08588238182f00/> appease latest nightly clippy
   Also updates `surface_keyed_fold.rs` `test_fold_keyed_infer_basic` test.

### New Features

 - <csr-id-5b6562662ce3a0dd172ddc1103a591c1c6037e95/> move persist manipulation into a proper optimization
   feat(hydroflow_plus): move persist manipulation into a proper
   optimization
 - <csr-id-cfb3029a6fb0836789db04a7d0d4a1e8b812b629/> add APIs for running optimization passes
   feat(hydroflow_plus): add APIs for running optimization passes

### Style

 - <csr-id-7958fb0d900be8fe7359326abfa11dcb8fb35e8a/> qualified path cleanups for clippy

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 71 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 4 unique issues were worked on: [#1083](https://github.com/hydro-project/hydroflow/issues/1083), [#1090](https://github.com/hydro-project/hydroflow/issues/1090), [#1098](https://github.com/hydro-project/hydroflow/issues/1098), [#1140](https://github.com/hydro-project/hydroflow/issues/1140)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1083](https://github.com/hydro-project/hydroflow/issues/1083)**
    - Add APIs for running optimization passes ([`cfb3029`](https://github.com/hydro-project/hydroflow/commit/cfb3029a6fb0836789db04a7d0d4a1e8b812b629))
 * **[#1090](https://github.com/hydro-project/hydroflow/issues/1090)**
    - Qualified path cleanups for clippy ([`7958fb0`](https://github.com/hydro-project/hydroflow/commit/7958fb0d900be8fe7359326abfa11dcb8fb35e8a))
 * **[#1098](https://github.com/hydro-project/hydroflow/issues/1098)**
    - Move persist manipulation into a proper optimization ([`5b65626`](https://github.com/hydro-project/hydroflow/commit/5b6562662ce3a0dd172ddc1103a591c1c6037e95))
 * **[#1140](https://github.com/hydro-project/hydroflow/issues/1140)**
    - Appease latest nightly clippy ([`fc447ff`](https://github.com/hydro-project/hydroflow/commit/fc447ffdf8fd1b2189545a991f08588238182f00))
 * **Uncategorized**
    - Release hydroflow_lang v0.6.2, hydroflow v0.6.2, hydroflow_plus v0.6.1, hydro_deploy v0.6.1, hydro_cli v0.6.1, hydroflow_plus_cli_integration v0.6.1, stageleft_tool v0.1.1 ([`23cfe08`](https://github.com/hydro-project/hydroflow/commit/23cfe0839079aa17d042bbd3976f6d188689d290))
    - Release hydroflow_cli_integration v0.5.2, hydroflow_lang v0.6.1, hydroflow_datalog_core v0.6.1, lattices v0.5.4, hydroflow v0.6.1, stageleft_macro v0.1.1, stageleft v0.2.1, hydroflow_plus v0.6.1, hydro_deploy v0.6.1, hydro_cli v0.6.1, hydroflow_plus_cli_integration v0.6.1, stageleft_tool v0.1.1 ([`cd63f22`](https://github.com/hydro-project/hydroflow/commit/cd63f2258c961a40f0e5dbef20ac329a2d570ad0))
</details>

## v0.1.0 (2024-01-29)

<csr-id-add8e602cbef513d1faa45f016a4e46d8bb5be6c/>

### Chore

 - <csr-id-add8e602cbef513d1faa45f016a4e46d8bb5be6c/> Commit empty CHANGELOG.md

### New Features

 - <csr-id-af6e3be60fdb69ceec1613347910f4dd49980d34/> push down persists and implement Pi example
   Also fixes type inference issues with reduce the same way as we did for fold.
 - <csr-id-c50ca121b6d5e30dc07843f82caa135b68626301/> split Rust core from Python bindings
 - <csr-id-71083233afc01e0132d7186f4af8c0b4a6323ec7/> support crates that have no entrypoints
   Also includes various bugfixes needed for Hydroflow+.
 - <csr-id-e5bdd12e32d6ea72fd91a55c12e09f07a0edaa5c/> add initial test using Hydro CLI from Hydroflow+
   This also required a change to Hydroflow core to make it possible to run the dataflow itself on a single thread (using a LocalSet), even if the surrounding runtime is not single-threaded (required to work around deadlocks because we can't use async APIs inside Hydroflow+). This requires us to spawn any Hydroflow tasks (only for `dest_sink` at the moment) right next to when we run the dataflow rather than when the Hydroflow graph is initialized. From a conceptual perspective, this seems _more right_, since now creating a Hydroflow program will not result in any actual tasks running.
   
   In the third PR of this series, I aim to add a new Hydroflow+ operator that will automate the setup of a `dest_sink`/`source_stream` pair that span nodes.
 - <csr-id-8b635683e5ac3c4ed2d896ae88e2953db1c6312c/> add a functional surface syntax using staging

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 5 unique issues were worked on: [#1021](https://github.com/hydro-project/hydroflow/issues/1021), [#899](https://github.com/hydro-project/hydroflow/issues/899), [#978](https://github.com/hydro-project/hydroflow/issues/978), [#983](https://github.com/hydro-project/hydroflow/issues/983), [#986](https://github.com/hydro-project/hydroflow/issues/986)

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
 * **[#986](https://github.com/hydro-project/hydroflow/issues/986)**
    - Split Rust core from Python bindings ([`c50ca12`](https://github.com/hydro-project/hydroflow/commit/c50ca121b6d5e30dc07843f82caa135b68626301))
 * **Uncategorized**
    - Release stageleft_tool v0.1.0 ([`4eb37db`](https://github.com/hydro-project/hydroflow/commit/4eb37db3c815005be9935556f049204f616ea801))
    - Commit empty CHANGELOG.md ([`add8e60`](https://github.com/hydro-project/hydroflow/commit/add8e602cbef513d1faa45f016a4e46d8bb5be6c))
</details>

