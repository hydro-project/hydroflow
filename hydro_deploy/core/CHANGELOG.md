# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.8.0 (2024-07-23)

### Refactor

 - <csr-id-e3e69334fcba8488b6fad3975fb0ba88e82a4b02/> remove unneeded `Arc<RwLock<` wrapping of `launch_binary` return value (1/3)
   > Curious if there was any intention behind why it was `Arc<RwLock<`?
   
   > I think before some refactors we took the I/O handles instead of using broadcast channels.
 - <csr-id-0feae7454e4674eea1f3308b3d6d4e9d459cda67/> build cache cleanup
   * Replace mystery tuple with new `struct BuildOutput`
   * Replace `Mutex` and `Arc`-infested `HashMap` with `memo-map` crate,
   greatly simplifying build cache typing
   * Remove redundant build caching in `HydroflowCrateService`, expose and
   use cache parameters as `BuildParams`
   * Remove `once_cell` and `async-once-cell` dependencies, use `std`'s
   `OnceLock`
   * Add `Failed to execute command: {}` context to `perf` error message
   * Cleanup some repeated `format!` expressions

### Style

 - <csr-id-947ebc1cb21a07fbfacae4ac956dbd0015a8a418/> rename `SSH` -> `Ssh`

### Refactor (BREAKING)

 - <csr-id-22865583a4260fe401c28aa39a74987478edc73d/> make `Service::collect_resources` take `&self` instead of `&mut self`
   #430 but still has `RwLock` wrapping
   
   Depends on #1347
 - <csr-id-c5a8de28e7844b3c29d58116d8340967f2e6bcc4/> make `Host` trait use `&self` interior mutability to remove `RwLock` wrappings #430
   Depends on #1346
 - <csr-id-f536eccf7297be8185108b60897e92ad0efffe4a/> Make `Host::provision` not async anymore
   I noticed that none of the method impls have any `await`s
 - <csr-id-057a0a510568cf81932368c8c65e056f91af7202/> make `HydroflowSource`, `HydroflowSink` traits use `&self` interior mutability to remove `RwLock` wrappings #430
   Depends on #1339
 - <csr-id-60390782dd7dcec18d193c800af716843a944dba/> replace `async-channel` with `tokio::sync::mpsc::unbounded_channel`
   Depends on #1339
   
   We could make the publicly facing `stdout`, `stderr` APIs return `impl Stream<Output = String>` in the future, maybe
 - <csr-id-141eae1c3a1869fa42756250618a21ea2a2c7e34/> replace some uses of `tokio::sync::RwLock` with `std::sync::Mutex` #430 (3/3)

### Style (BREAKING)

 - <csr-id-12b8ba53f28eb9de1318b41cdf1e23282f6f0eb6/> enable clippy `upper-case-acronyms-aggressive`
   * rename `GCP` -> `Gcp`, `NodeID` -> `NodeId`
   * update CI `cargo-generate` template testing to use PR's branch instead
   of whatever `main` happens to be

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 10 calendar days.
 - 59 days passed between releases.
 - 10 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 10 unique issues were worked on: [#1334](https://github.com/hydro-project/hydroflow/issues/1334), [#1338](https://github.com/hydro-project/hydroflow/issues/1338), [#1339](https://github.com/hydro-project/hydroflow/issues/1339), [#1340](https://github.com/hydro-project/hydroflow/issues/1340), [#1343](https://github.com/hydro-project/hydroflow/issues/1343), [#1345](https://github.com/hydro-project/hydroflow/issues/1345), [#1346](https://github.com/hydro-project/hydroflow/issues/1346), [#1347](https://github.com/hydro-project/hydroflow/issues/1347), [#1348](https://github.com/hydro-project/hydroflow/issues/1348), [#1356](https://github.com/hydro-project/hydroflow/issues/1356)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1334](https://github.com/hydro-project/hydroflow/issues/1334)**
    - Build cache cleanup ([`0feae74`](https://github.com/hydro-project/hydroflow/commit/0feae7454e4674eea1f3308b3d6d4e9d459cda67))
 * **[#1338](https://github.com/hydro-project/hydroflow/issues/1338)**
    - Remove unneeded `Arc<RwLock<` wrapping of `launch_binary` return value (1/3) ([`e3e6933`](https://github.com/hydro-project/hydroflow/commit/e3e69334fcba8488b6fad3975fb0ba88e82a4b02))
 * **[#1339](https://github.com/hydro-project/hydroflow/issues/1339)**
    - Replace some uses of `tokio::sync::RwLock` with `std::sync::Mutex` #430 (3/3) ([`141eae1`](https://github.com/hydro-project/hydroflow/commit/141eae1c3a1869fa42756250618a21ea2a2c7e34))
 * **[#1340](https://github.com/hydro-project/hydroflow/issues/1340)**
    - Rename `SSH` -> `Ssh` ([`947ebc1`](https://github.com/hydro-project/hydroflow/commit/947ebc1cb21a07fbfacae4ac956dbd0015a8a418))
 * **[#1343](https://github.com/hydro-project/hydroflow/issues/1343)**
    - Make `Host::provision` not async anymore ([`f536ecc`](https://github.com/hydro-project/hydroflow/commit/f536eccf7297be8185108b60897e92ad0efffe4a))
 * **[#1345](https://github.com/hydro-project/hydroflow/issues/1345)**
    - Enable clippy `upper-case-acronyms-aggressive` ([`12b8ba5`](https://github.com/hydro-project/hydroflow/commit/12b8ba53f28eb9de1318b41cdf1e23282f6f0eb6))
 * **[#1346](https://github.com/hydro-project/hydroflow/issues/1346)**
    - Make `HydroflowSource`, `HydroflowSink` traits use `&self` interior mutability to remove `RwLock` wrappings #430 ([`057a0a5`](https://github.com/hydro-project/hydroflow/commit/057a0a510568cf81932368c8c65e056f91af7202))
 * **[#1347](https://github.com/hydro-project/hydroflow/issues/1347)**
    - Make `Host` trait use `&self` interior mutability to remove `RwLock` wrappings #430 ([`c5a8de2`](https://github.com/hydro-project/hydroflow/commit/c5a8de28e7844b3c29d58116d8340967f2e6bcc4))
 * **[#1348](https://github.com/hydro-project/hydroflow/issues/1348)**
    - Make `Service::collect_resources` take `&self` instead of `&mut self` ([`2286558`](https://github.com/hydro-project/hydroflow/commit/22865583a4260fe401c28aa39a74987478edc73d))
 * **[#1356](https://github.com/hydro-project/hydroflow/issues/1356)**
    - Replace `async-channel` with `tokio::sync::mpsc::unbounded_channel` ([`6039078`](https://github.com/hydro-project/hydroflow/commit/60390782dd7dcec18d193c800af716843a944dba))
</details>

## v0.7.0 (2024-05-24)

### New Features

 - <csr-id-29a263fb564c5ce4bc495ea4e9d20b8b2621b645/> add support for collecting counts and running perf

### Bug Fixes

 - <csr-id-92c72ba9527241f88dfb23f64b999c8e4bd2b26c/> end processes with SIGTERM instead of SIGKILL
   fix(hydro_deploy): end processes with SIGTERM instead of SIGKILL

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 41 calendar days.
 - 44 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#1129](https://github.com/hydro-project/hydroflow/issues/1129), [#1157](https://github.com/hydro-project/hydroflow/issues/1157)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1129](https://github.com/hydro-project/hydroflow/issues/1129)**
    - End processes with SIGTERM instead of SIGKILL ([`92c72ba`](https://github.com/hydro-project/hydroflow/commit/92c72ba9527241f88dfb23f64b999c8e4bd2b26c))
 * **[#1157](https://github.com/hydro-project/hydroflow/issues/1157)**
    - Add support for collecting counts and running perf ([`29a263f`](https://github.com/hydro-project/hydroflow/commit/29a263fb564c5ce4bc495ea4e9d20b8b2621b645))
 * **Uncategorized**
    - Release hydroflow_lang v0.7.0, hydroflow_datalog_core v0.7.0, hydroflow_datalog v0.7.0, hydroflow_macro v0.7.0, lattices v0.5.5, multiplatform_test v0.1.0, pusherator v0.0.6, hydroflow v0.7.0, stageleft_macro v0.2.0, stageleft v0.3.0, stageleft_tool v0.2.0, hydroflow_plus v0.7.0, hydro_deploy v0.7.0, hydro_cli v0.7.0, hydroflow_plus_cli_integration v0.7.0, safety bump 8 crates ([`2852147`](https://github.com/hydro-project/hydroflow/commit/285214740627685e911781793e05d234ab2ad2bd))
</details>

## v0.6.1 (2024-04-09)

<csr-id-7958fb0d900be8fe7359326abfa11dcb8fb35e8a/>

### Style

 - <csr-id-7958fb0d900be8fe7359326abfa11dcb8fb35e8a/> qualified path cleanups for clippy

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 34 calendar days.
 - 38 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#1090](https://github.com/hydro-project/hydroflow/issues/1090)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1090](https://github.com/hydro-project/hydroflow/issues/1090)**
    - Qualified path cleanups for clippy ([`7958fb0`](https://github.com/hydro-project/hydroflow/commit/7958fb0d900be8fe7359326abfa11dcb8fb35e8a))
 * **Uncategorized**
    - Release hydroflow_plus v0.6.1, hydro_deploy v0.6.1, hydro_cli v0.6.1, hydroflow_plus_cli_integration v0.6.1 ([`c385c13`](https://github.com/hydro-project/hydroflow/commit/c385c132c9733d1bace82156aa14216b8e7fef9f))
    - Release hydroflow_lang v0.6.2, hydroflow v0.6.2, hydroflow_plus v0.6.1, hydro_deploy v0.6.1, hydro_cli v0.6.1, hydroflow_plus_cli_integration v0.6.1, stageleft_tool v0.1.1 ([`23cfe08`](https://github.com/hydro-project/hydroflow/commit/23cfe0839079aa17d042bbd3976f6d188689d290))
    - Release hydroflow_cli_integration v0.5.2, hydroflow_lang v0.6.1, hydroflow_datalog_core v0.6.1, lattices v0.5.4, hydroflow v0.6.1, stageleft_macro v0.1.1, stageleft v0.2.1, hydroflow_plus v0.6.1, hydro_deploy v0.6.1, hydro_cli v0.6.1, hydroflow_plus_cli_integration v0.6.1, stageleft_tool v0.1.1 ([`cd63f22`](https://github.com/hydro-project/hydroflow/commit/cd63f2258c961a40f0e5dbef20ac329a2d570ad0))
</details>

## v0.6.0 (2024-03-02)

<csr-id-39ab8b0278e9e3fe96552ace0a4ae768a6bc10d8/>
<csr-id-e9639f608f8dafd3f384837067800a66951b25df/>

### Chore

 - <csr-id-39ab8b0278e9e3fe96552ace0a4ae768a6bc10d8/> appease various clippy lints

### New Features

 - <csr-id-fcf43bf86fe550247dffa4641a9ce3aff3b9afc3/> Add support for azure
   I accidentally committed some large files, so you won't see the commit
   history because I copied over the changes onto a fresh clone.

### Other

 - <csr-id-e9639f608f8dafd3f384837067800a66951b25df/> consolidate tasks and use sccache and nextest

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 6 calendar days.
 - 31 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#1015](https://github.com/hydro-project/hydroflow/issues/1015), [#1043](https://github.com/hydro-project/hydroflow/issues/1043), [#1084](https://github.com/hydro-project/hydroflow/issues/1084)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1015](https://github.com/hydro-project/hydroflow/issues/1015)**
    - Consolidate tasks and use sccache and nextest ([`e9639f6`](https://github.com/hydro-project/hydroflow/commit/e9639f608f8dafd3f384837067800a66951b25df))
 * **[#1043](https://github.com/hydro-project/hydroflow/issues/1043)**
    - Add support for azure ([`fcf43bf`](https://github.com/hydro-project/hydroflow/commit/fcf43bf86fe550247dffa4641a9ce3aff3b9afc3))
 * **[#1084](https://github.com/hydro-project/hydroflow/issues/1084)**
    - Appease various clippy lints ([`39ab8b0`](https://github.com/hydro-project/hydroflow/commit/39ab8b0278e9e3fe96552ace0a4ae768a6bc10d8))
 * **Uncategorized**
    - Release hydroflow_lang v0.6.0, hydroflow_datalog_core v0.6.0, hydroflow_datalog v0.6.0, hydroflow_macro v0.6.0, lattices v0.5.3, variadics v0.0.4, pusherator v0.0.5, hydroflow v0.6.0, stageleft v0.2.0, hydroflow_plus v0.6.0, hydro_deploy v0.6.0, hydro_cli v0.6.0, hydroflow_plus_cli_integration v0.6.0, safety bump 7 crates ([`09ea65f`](https://github.com/hydro-project/hydroflow/commit/09ea65fe9cd45c357c43bffca30e60243fa45cc8))
</details>

## v0.5.1 (2024-01-29)

<csr-id-1b555e57c8c812bed4d6495d2960cbf77fb0b3ef/>

### Chore

 - <csr-id-1b555e57c8c812bed4d6495d2960cbf77fb0b3ef/> manually set lockstep-versioned crates (and `lattices`) to version `0.5.1`
   Setting manually since
   https://github.com/frewsxcv/rust-crates-index/issues/159 is messing with
   smart-release

### Documentation

 - <csr-id-3b36020d16792f26da4df3c5b09652a4ab47ec4f/> actually committing empty CHANGELOG.md is required

### New Features

 - <csr-id-20fd1e5f876c5977e44a58757f41c66bdf6a3d15/> improve build error message debuggability
 - <csr-id-46d87fa364d3fe01422cf3c404fbc8a1d5e9fb88/> pass subgraph ID through deploy metadata
 - <csr-id-b7aafd3c97897db4bff62c4ab0b7480ef9a799e0/> improve API naming and eliminate wire API for builders
 - <csr-id-53d7aee8dcc574d47864ec89bfea30a82eab0ee7/> improve Rust API for defining services
 - <csr-id-c50ca121b6d5e30dc07843f82caa135b68626301/> split Rust core from Python bindings

### Bug Fixes

 - <csr-id-d23c2299098dd62058c0951c99a62bb9e0af5b25/> avoid inflexible `\\?\` canonical paths on windows to mitigate `/` separator errors
 - <csr-id-f8a0b95113e92e003061d2a3865c84d69851dd8e/> race conditions when handshake channels capture other outputs
   Timeouts in Hydroflow+ tests were being caused by a race condition in Hydro Deploy where stdout sent after a handshake message would sometimes be sent to the `cli_stdout` channel for handshakes.
   
   This PR adjusts the handshake channels to always be oneshot, so that the broadcaster immediately knows when to send data to the regular stdout channels.
   
   Also refactors Hydro Deploy sources to split up more modules.
 - <csr-id-1ae27de6aafb72cee5da0cce6cf52748161d0f33/> don't vendor openssl and fix docker build
 - <csr-id-1d8adc1df15bac74c6f4496589d615e361019f50/> fix docs and remove unnecessary async_trait

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release over the course of 39 calendar days.
 - 11 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 9 unique issues were worked on: [#1010](https://github.com/hydro-project/hydroflow/issues/1010), [#1014](https://github.com/hydro-project/hydroflow/issues/1014), [#986](https://github.com/hydro-project/hydroflow/issues/986), [#987](https://github.com/hydro-project/hydroflow/issues/987), [#992](https://github.com/hydro-project/hydroflow/issues/992), [#994](https://github.com/hydro-project/hydroflow/issues/994), [#995](https://github.com/hydro-project/hydroflow/issues/995), [#996](https://github.com/hydro-project/hydroflow/issues/996), [#999](https://github.com/hydro-project/hydroflow/issues/999)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1010](https://github.com/hydro-project/hydroflow/issues/1010)**
    - Improve build error message debuggability ([`20fd1e5`](https://github.com/hydro-project/hydroflow/commit/20fd1e5f876c5977e44a58757f41c66bdf6a3d15))
 * **[#1014](https://github.com/hydro-project/hydroflow/issues/1014)**
    - Avoid inflexible `\\?\` canonical paths on windows to mitigate `/` separator errors ([`d23c229`](https://github.com/hydro-project/hydroflow/commit/d23c2299098dd62058c0951c99a62bb9e0af5b25))
 * **[#986](https://github.com/hydro-project/hydroflow/issues/986)**
    - Split Rust core from Python bindings ([`c50ca12`](https://github.com/hydro-project/hydroflow/commit/c50ca121b6d5e30dc07843f82caa135b68626301))
 * **[#987](https://github.com/hydro-project/hydroflow/issues/987)**
    - Improve Rust API for defining services ([`53d7aee`](https://github.com/hydro-project/hydroflow/commit/53d7aee8dcc574d47864ec89bfea30a82eab0ee7))
 * **[#992](https://github.com/hydro-project/hydroflow/issues/992)**
    - Fix docs and remove unnecessary async_trait ([`1d8adc1`](https://github.com/hydro-project/hydroflow/commit/1d8adc1df15bac74c6f4496589d615e361019f50))
 * **[#994](https://github.com/hydro-project/hydroflow/issues/994)**
    - Don't vendor openssl and fix docker build ([`1ae27de`](https://github.com/hydro-project/hydroflow/commit/1ae27de6aafb72cee5da0cce6cf52748161d0f33))
 * **[#995](https://github.com/hydro-project/hydroflow/issues/995)**
    - Improve API naming and eliminate wire API for builders ([`b7aafd3`](https://github.com/hydro-project/hydroflow/commit/b7aafd3c97897db4bff62c4ab0b7480ef9a799e0))
 * **[#996](https://github.com/hydro-project/hydroflow/issues/996)**
    - Pass subgraph ID through deploy metadata ([`46d87fa`](https://github.com/hydro-project/hydroflow/commit/46d87fa364d3fe01422cf3c404fbc8a1d5e9fb88))
 * **[#999](https://github.com/hydro-project/hydroflow/issues/999)**
    - Race conditions when handshake channels capture other outputs ([`f8a0b95`](https://github.com/hydro-project/hydroflow/commit/f8a0b95113e92e003061d2a3865c84d69851dd8e))
 * **Uncategorized**
    - Release hydro_deploy v0.5.1 ([`f7a54c7`](https://github.com/hydro-project/hydroflow/commit/f7a54c7ae7c771b16ed2853b28a480fba5f06e5b))
    - Actually committing empty CHANGELOG.md is required ([`3b36020`](https://github.com/hydro-project/hydroflow/commit/3b36020d16792f26da4df3c5b09652a4ab47ec4f))
    - Manually set lockstep-versioned crates (and `lattices`) to version `0.5.1` ([`1b555e5`](https://github.com/hydro-project/hydroflow/commit/1b555e57c8c812bed4d6495d2960cbf77fb0b3ef))
</details>

