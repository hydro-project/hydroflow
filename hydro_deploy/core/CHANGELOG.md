# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.5.1 (2024-01-29)

### Chore

 - <csr-id-1b555e57c8c812bed4d6495d2960cbf77fb0b3ef/> manually set lockstep-versioned crates (and `lattices`) to version `0.5.1`
   Setting manually since
   https://github.com/frewsxcv/rust-crates-index/issues/159 is messing with
   smart-release

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

 - 10 commits contributed to the release over the course of 39 calendar days.
 - 10 commits were understood as [conventional](https://www.conventionalcommits.org).
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
    - Manually set lockstep-versioned crates (and `lattices`) to version `0.5.1` ([`1b555e5`](https://github.com/hydro-project/hydroflow/commit/1b555e57c8c812bed4d6495d2960cbf77fb0b3ef))
</details>

