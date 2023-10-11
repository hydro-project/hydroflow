# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.5.0 (2023-10-11)

<csr-id-f19eccc79d6d7c88de7ba1ef6a0abf1caaef377f/>
<csr-id-2eac8ce42545cb543892b28267fb2f7089a92cdb/>
<csr-id-1126266e69c2c4364bc8de558f11859e5bad1c69/>

### Chore

 - <csr-id-f19eccc79d6d7c88de7ba1ef6a0abf1caaef377f/> bump proc-macro2 min version to 1.0.63
 - <csr-id-2eac8ce42545cb543892b28267fb2f7089a92cdb/> refactor collect-String into fold for `clippy::clippy::format_collect` lint
   using `.map(|| format!()).collect()` results in a heap `String` allocation for each item in the iterator, the fold only does one. Doesn't really matter for this case, just appeasing the lint

### New Features

 - <csr-id-b3d114827256f2b82a3c357f3419c6853a97f5c0/> initial technically working version of `demux_enum` with very bad error messages
   Technically does not check port names at all, just depends on their order.
 - <csr-id-f013c3ca15f2cc9413fcfb92898f71d5fc00073a/> add import!() expression
 - <csr-id-7714403e130969b96c8f405444d4daf451450fdf/> Add `monotonic_fn` and `morphism` macros, update snapshots for flow props.

### Bug Fixes

 - <csr-id-5ac9ddebedf615f87684d1092382ba64826c1c1c/> separate internal compiler operators in docs name/category/sort order
 - <csr-id-80d985f2870b80771c23eed9e2d9d2589d17088e/> properly feature gate `macro_invocation_path` `source_file()` #937

### Refactor

 - <csr-id-1126266e69c2c4364bc8de558f11859e5bad1c69/> `demux_enum` requires enum type name, add better error handling

### New Features (BREAKING)

 - <csr-id-9ed0ce02128a0eeaf0b603efcbe896427e47ef62/> Simplify graph printing code, add delta/cumul green edges, allow hiding of vars/subgraphs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 55 calendar days.
 - 56 days passed between releases.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 6 unique issues were worked on: [#881](https://github.com/hydro-project/hydroflow/issues/881), [#882](https://github.com/hydro-project/hydroflow/issues/882), [#884](https://github.com/hydro-project/hydroflow/issues/884), [#898](https://github.com/hydro-project/hydroflow/issues/898), [#932](https://github.com/hydro-project/hydroflow/issues/932), [#938](https://github.com/hydro-project/hydroflow/issues/938)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#881](https://github.com/hydro-project/hydroflow/issues/881)**
    - Refactor collect-String into fold for `clippy::clippy::format_collect` lint ([`2eac8ce`](https://github.com/hydro-project/hydroflow/commit/2eac8ce42545cb543892b28267fb2f7089a92cdb))
 * **[#882](https://github.com/hydro-project/hydroflow/issues/882)**
    - Add `monotonic_fn` and `morphism` macros, update snapshots for flow props. ([`7714403`](https://github.com/hydro-project/hydroflow/commit/7714403e130969b96c8f405444d4daf451450fdf))
 * **[#884](https://github.com/hydro-project/hydroflow/issues/884)**
    - Separate internal compiler operators in docs name/category/sort order ([`5ac9dde`](https://github.com/hydro-project/hydroflow/commit/5ac9ddebedf615f87684d1092382ba64826c1c1c))
 * **[#898](https://github.com/hydro-project/hydroflow/issues/898)**
    - Add import!() expression ([`f013c3c`](https://github.com/hydro-project/hydroflow/commit/f013c3ca15f2cc9413fcfb92898f71d5fc00073a))
 * **[#932](https://github.com/hydro-project/hydroflow/issues/932)**
    - Simplify graph printing code, add delta/cumul green edges, allow hiding of vars/subgraphs ([`9ed0ce0`](https://github.com/hydro-project/hydroflow/commit/9ed0ce02128a0eeaf0b603efcbe896427e47ef62))
 * **[#938](https://github.com/hydro-project/hydroflow/issues/938)**
    - Properly feature gate `macro_invocation_path` `source_file()` #937 ([`80d985f`](https://github.com/hydro-project/hydroflow/commit/80d985f2870b80771c23eed9e2d9d2589d17088e))
 * **Uncategorized**
    - Release hydroflow_lang v0.5.0, hydroflow_datalog_core v0.5.0, hydroflow_datalog v0.5.0, hydroflow_macro v0.5.0, lattices v0.5.0, hydroflow v0.5.0, hydro_cli v0.5.0, safety bump 4 crates ([`2e2d8b3`](https://github.com/hydro-project/hydroflow/commit/2e2d8b386fb086c8276a2853d2a1f96ad4d7c221))
    - Bump proc-macro2 min version to 1.0.63 ([`f19eccc`](https://github.com/hydro-project/hydroflow/commit/f19eccc79d6d7c88de7ba1ef6a0abf1caaef377f))
    - `demux_enum` requires enum type name, add better error handling ([`1126266`](https://github.com/hydro-project/hydroflow/commit/1126266e69c2c4364bc8de558f11859e5bad1c69))
    - Initial technically working version of `demux_enum` with very bad error messages ([`b3d1148`](https://github.com/hydro-project/hydroflow/commit/b3d114827256f2b82a3c357f3419c6853a97f5c0))
</details>

## 0.4.0 (2023-08-15)

### New Features

 - <csr-id-b4b9644a19e8e7e7725c9c5b88e3a6b8c2be7364/> Add `use` statements to hydroflow syntax
   And use in doc tests.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 26 calendar days.
 - 42 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#845](https://github.com/hydro-project/hydroflow/issues/845)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#845](https://github.com/hydro-project/hydroflow/issues/845)**
    - Add `use` statements to hydroflow syntax ([`b4b9644`](https://github.com/hydro-project/hydroflow/commit/b4b9644a19e8e7e7725c9c5b88e3a6b8c2be7364))
 * **Uncategorized**
    - Release hydroflow_lang v0.4.0, hydroflow_datalog_core v0.4.0, hydroflow_datalog v0.4.0, hydroflow_macro v0.4.0, lattices v0.4.0, pusherator v0.0.3, hydroflow v0.4.0, hydro_cli v0.4.0, safety bump 4 crates ([`cb313f0`](https://github.com/hydro-project/hydroflow/commit/cb313f0635214460a8308d05cbef4bf7f4bfaa15))
</details>

## 0.3.0 (2023-07-04)

### Documentation

 - <csr-id-c6b39e72590a01590690b0e42237c9853b105edc/> avoid double-extension for generated files which breaks navigation

### New Features

 - <csr-id-ea65349d241873f8460d7a8b024d64c63180246f/> emit `compile_error!` diagnostics for stable
 - <csr-id-22abcaff806c7de6e4a7725656bbcf201e7d9259/> allow stable build, refactors behind `nightly` feature flag

### Bug Fixes

 - <csr-id-8d3494b5afee858114a602a3e23077bb6d24dd77/> update proc-macro2, use new span location API where possible
   requires latest* rust nightly version
   
   *latest = 2023-06-28 or something

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 25 calendar days.
 - 33 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#758](https://github.com/hydro-project/hydroflow/issues/758), [#780](https://github.com/hydro-project/hydroflow/issues/780), [#801](https://github.com/hydro-project/hydroflow/issues/801)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#758](https://github.com/hydro-project/hydroflow/issues/758)**
    - Avoid double-extension for generated files which breaks navigation ([`c6b39e7`](https://github.com/hydro-project/hydroflow/commit/c6b39e72590a01590690b0e42237c9853b105edc))
 * **[#780](https://github.com/hydro-project/hydroflow/issues/780)**
    - Emit `compile_error!` diagnostics for stable ([`ea65349`](https://github.com/hydro-project/hydroflow/commit/ea65349d241873f8460d7a8b024d64c63180246f))
    - Allow stable build, refactors behind `nightly` feature flag ([`22abcaf`](https://github.com/hydro-project/hydroflow/commit/22abcaff806c7de6e4a7725656bbcf201e7d9259))
 * **[#801](https://github.com/hydro-project/hydroflow/issues/801)**
    - Update proc-macro2, use new span location API where possible ([`8d3494b`](https://github.com/hydro-project/hydroflow/commit/8d3494b5afee858114a602a3e23077bb6d24dd77))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.3.0, hydroflow_lang v0.3.0, hydroflow_datalog_core v0.3.0, hydroflow_datalog v0.3.0, hydroflow_macro v0.3.0, lattices v0.3.0, pusherator v0.0.2, hydroflow v0.3.0, hydro_cli v0.3.0, safety bump 5 crates ([`ec9633e`](https://github.com/hydro-project/hydroflow/commit/ec9633e2e393c2bf106223abeb0b680200fbdf84))
</details>

## 0.2.0 (2023-05-31)

<csr-id-fd896fbe925fbd8ef1d16be7206ac20ba585081a/>
<csr-id-c9e8603c6ede61d5098869d3d0b5e24c7254f7a4/>

### Chore

 - <csr-id-fd896fbe925fbd8ef1d16be7206ac20ba585081a/> manually bump versions for v0.2.0 release

### Documentation

 - <csr-id-989adcbcd304ad0890d71351d56a22977bdcf73f/> categorize operators, organize op docs, fix #727

### Bug Fixes

 - <csr-id-554d563fe53a1303c5a5c9352197365235c607e3/> make `build.rs`s infallible, log to stderr, to fix release

### Refactor

 - <csr-id-c9e8603c6ede61d5098869d3d0b5e24c7254f7a4/> remove `hydroflow_internalmacro`, use `hydroflow_lang/build.rs` instead

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 1 day passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#728](https://github.com/hydro-project/hydroflow/issues/728), [#730](https://github.com/hydro-project/hydroflow/issues/730)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#728](https://github.com/hydro-project/hydroflow/issues/728)**
    - Remove `hydroflow_internalmacro`, use `hydroflow_lang/build.rs` instead ([`c9e8603`](https://github.com/hydro-project/hydroflow/commit/c9e8603c6ede61d5098869d3d0b5e24c7254f7a4))
 * **[#730](https://github.com/hydro-project/hydroflow/issues/730)**
    - Categorize operators, organize op docs, fix #727 ([`989adcb`](https://github.com/hydro-project/hydroflow/commit/989adcbcd304ad0890d71351d56a22977bdcf73f))
 * **Uncategorized**
    - Release hydroflow_lang v0.2.0, hydroflow_datalog_core v0.2.0, hydroflow_datalog v0.2.0, hydroflow_macro v0.2.0, lattices v0.2.0, hydroflow v0.2.0, hydro_cli v0.2.0 ([`ca464c3`](https://github.com/hydro-project/hydroflow/commit/ca464c32322a7ad39eb53e1794777c849aa548a0))
    - Make `build.rs`s infallible, log to stderr, to fix release ([`554d563`](https://github.com/hydro-project/hydroflow/commit/554d563fe53a1303c5a5c9352197365235c607e3))
    - Manually bump versions for v0.2.0 release ([`fd896fb`](https://github.com/hydro-project/hydroflow/commit/fd896fbe925fbd8ef1d16be7206ac20ba585081a))
</details>

## 0.1.1 (2023-05-30)

### Documentation

 - <csr-id-28c90251dd877dd84f28886eecb7b366abf3d45b/> Add initial Hydro Deploy docs
   Renamed from Hydro CLI because the CLI isn't really the main thing. Also moves the Hydroflow docs to a subdirectory and sets up a dropdown for multiple docs.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 6 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#686](https://github.com/hydro-project/hydroflow/issues/686)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#686](https://github.com/hydro-project/hydroflow/issues/686)**
    - Add initial Hydro Deploy docs ([`28c9025`](https://github.com/hydro-project/hydroflow/commit/28c90251dd877dd84f28886eecb7b366abf3d45b))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.1.1, hydroflow_lang v0.1.1, hydroflow_datalog_core v0.1.1, hydroflow_macro v0.1.1, lattices v0.1.2, hydroflow v0.1.1, hydro_cli v0.1.0 ([`d9fa8b3`](https://github.com/hydro-project/hydroflow/commit/d9fa8b387e303b33d9614dbde80abf1af08bd8eb))
</details>

## 0.1.0 (2023-05-23)

<csr-id-52ee8f8e443f0a8b5caf92d2c5f028c00302a79b/>

### Chore

 - <csr-id-52ee8f8e443f0a8b5caf92d2c5f028c00302a79b/> bump versions to 0.1.0 for release
   For release on crates.io for v0.1

### Documentation

 - <csr-id-a8957ec4457aae1cfd6fae031bede5e3f4fcc75d/> Add rustdocs to hydroflow's proc macros

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 4 unique issues were worked on: [#661](https://github.com/hydro-project/hydroflow/issues/661), [#671](https://github.com/hydro-project/hydroflow/issues/671), [#677](https://github.com/hydro-project/hydroflow/issues/677), [#684](https://github.com/hydro-project/hydroflow/issues/684)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#661](https://github.com/hydro-project/hydroflow/issues/661)**
    - Add hydroflow_{test, main} so that hydroflow is actually singlethreaded ([`f61054e`](https://github.com/hydro-project/hydroflow/commit/f61054eaeca6fab1ab0cb588b7ed546b87772e91))
 * **[#671](https://github.com/hydro-project/hydroflow/issues/671)**
    - Migrate docs to a unified Docusuarus site ([`feed326`](https://github.com/hydro-project/hydroflow/commit/feed3268c0aabeb027b19abd9ed06c565a0462f4))
 * **[#677](https://github.com/hydro-project/hydroflow/issues/677)**
    - Add rustdocs to hydroflow's proc macros ([`a8957ec`](https://github.com/hydro-project/hydroflow/commit/a8957ec4457aae1cfd6fae031bede5e3f4fcc75d))
 * **[#684](https://github.com/hydro-project/hydroflow/issues/684)**
    - Bump versions to 0.1.0 for release ([`52ee8f8`](https://github.com/hydro-project/hydroflow/commit/52ee8f8e443f0a8b5caf92d2c5f028c00302a79b))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.1.0, hydroflow_internalmacro v0.1.0, hydroflow_lang v0.1.0, hydroflow_datalog_core v0.1.0, hydroflow_datalog v0.1.0, hydroflow_macro v0.1.0, lattices v0.1.1, hydroflow v0.1.0 ([`7324974`](https://github.com/hydro-project/hydroflow/commit/73249744293c9b89cbaa2d84b23ca3f25b00ae4e))
</details>

## 0.0.1 (2023-05-21)

<csr-id-20a1b2c0cd04a8b495a02ce345db3d48a99ea0e9/>

### Style

 - <csr-id-20a1b2c0cd04a8b495a02ce345db3d48a99ea0e9/> rustfmt group imports

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 14 calendar days.
 - 24 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 2 unique issues were worked on: [#643](https://github.com/hydro-project/hydroflow/issues/643), [#660](https://github.com/hydro-project/hydroflow/issues/660)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#643](https://github.com/hydro-project/hydroflow/issues/643)**
    - Fix book operators build ([`30d68a6`](https://github.com/hydro-project/hydroflow/commit/30d68a6865ae9769b073d31c39ccf40f724355d9))
 * **[#660](https://github.com/hydro-project/hydroflow/issues/660)**
    - Rustfmt group imports ([`20a1b2c`](https://github.com/hydro-project/hydroflow/commit/20a1b2c0cd04a8b495a02ce345db3d48a99ea0e9))
 * **Uncategorized**
    - Release hydroflow_cli_integration v0.0.1, hydroflow_lang v0.0.1, hydroflow_datalog_core v0.0.1, hydroflow_datalog v0.0.1, hydroflow_macro v0.0.1, lattices v0.1.0, variadics v0.0.2, pusherator v0.0.1, hydroflow v0.0.2 ([`809395a`](https://github.com/hydro-project/hydroflow/commit/809395acddb78949d7a2bf036e1a94972f23b1ad))
</details>

## 0.0.0 (2023-04-26)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 42 commits contributed to the release over the course of 302 calendar days.
 - 0 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 13 unique issues were worked on: [#162](https://github.com/hydro-project/hydroflow/issues/162), [#310](https://github.com/hydro-project/hydroflow/issues/310), [#311](https://github.com/hydro-project/hydroflow/issues/311), [#318](https://github.com/hydro-project/hydroflow/issues/318), [#329](https://github.com/hydro-project/hydroflow/issues/329), [#404](https://github.com/hydro-project/hydroflow/issues/404), [#419](https://github.com/hydro-project/hydroflow/issues/419), [#441 11/14](https://github.com/hydro-project/hydroflow/issues/441 11/14), [#441 14/14](https://github.com/hydro-project/hydroflow/issues/441 14/14), [#501](https://github.com/hydro-project/hydroflow/issues/501), [#603](https://github.com/hydro-project/hydroflow/issues/603), [#609](https://github.com/hydro-project/hydroflow/issues/609), [#617](https://github.com/hydro-project/hydroflow/issues/617)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#162](https://github.com/hydro-project/hydroflow/issues/162)**
    - SerdeGraph from parser to be callable at runtime ([`17dd150`](https://github.com/hydro-project/hydroflow/commit/17dd1500be1dab5f7abbd498d8f96b6ed00dba59))
 * **[#310](https://github.com/hydro-project/hydroflow/issues/310)**
    - Sort ops while generating the op doc in the book ([`8193409`](https://github.com/hydro-project/hydroflow/commit/8193409eff2e20c4c192b4435df609ea99ea5598))
 * **[#311](https://github.com/hydro-project/hydroflow/issues/311)**
    - Better autogen of input/output specs for ops docs ([`2cbd3e7`](https://github.com/hydro-project/hydroflow/commit/2cbd3e7757da427a47fdde74278de3ec8cbbf9fb))
 * **[#318](https://github.com/hydro-project/hydroflow/issues/318)**
    - Reintroduce code to generate streaming/blocking annotations in docs ([`9530fd7`](https://github.com/hydro-project/hydroflow/commit/9530fd7f501fd078972c6fefe3aa211f23bc1814))
 * **[#329](https://github.com/hydro-project/hydroflow/issues/329)**
    - Get hydroflow to compile to WASM ([`24354d2`](https://github.com/hydro-project/hydroflow/commit/24354d2e11c69e38e4e021aa4acf1525b376b2b1))
 * **[#404](https://github.com/hydro-project/hydroflow/issues/404)**
    - Fix op docs "blocking" to check elided port names, fix #400 ([`608e65b`](https://github.com/hydro-project/hydroflow/commit/608e65b61788376a06ab56b7f92dfd45820b4c0e))
 * **[#419](https://github.com/hydro-project/hydroflow/issues/419)**
    - Encapsulate `FlatGraph`, separate `FlatGraphBuilder` ([`fceaea5`](https://github.com/hydro-project/hydroflow/commit/fceaea5659ac76c2275c1487582a17b646858602))
 * **[#441 11/14](https://github.com/hydro-project/hydroflow/issues/441 11/14)**
    - Remove `FlatGraph`, unify under `PartitionedGraph` ([`b640b53`](https://github.com/hydro-project/hydroflow/commit/b640b532e34b29f44c768d523fbf780dba9785ff))
 * **[#441 14/14](https://github.com/hydro-project/hydroflow/issues/441 14/14)**
    - Cleanup graph docs, organize method names ([`09d3b57`](https://github.com/hydro-project/hydroflow/commit/09d3b57eb03f3920bd10f5c10277d3ef4f9cb0ec))
 * **[#501](https://github.com/hydro-project/hydroflow/issues/501)**
    - Preserve serialize diagnostics for hydroflow graph, stop emitting expected warnings in tests ([`0c810e5`](https://github.com/hydro-project/hydroflow/commit/0c810e5fdd3445923c0c7afbe651f2b4a72c115e))
 * **[#603](https://github.com/hydro-project/hydroflow/issues/603)**
    - Improve operator docs page ([`4241ca0`](https://github.com/hydro-project/hydroflow/commit/4241ca07bad6c8777b4e1a05c6c900cfa8276c81))
 * **[#609](https://github.com/hydro-project/hydroflow/issues/609)**
    - Update syn to 2.0 ([`2e7d802`](https://github.com/hydro-project/hydroflow/commit/2e7d8024f35893ef0abcb6851e370b00615f9562))
 * **[#617](https://github.com/hydro-project/hydroflow/issues/617)**
    - Update `Cargo.toml`s for publishing ([`a78ff9a`](https://github.com/hydro-project/hydroflow/commit/a78ff9aace6771787c2b72aad83be6ad8d49a828))
 * **Uncategorized**
    - Setup release workflow ([`108d0e9`](https://github.com/hydro-project/hydroflow/commit/108d0e933a08b183c4dadf8c3499e4946696e263))
    - Use `HydroflowGraph` for graph writing, delete `SerdeGraph` ([`d1ef14e`](https://github.com/hydro-project/hydroflow/commit/d1ef14ee459c51d5a2dd9e7ea03050772e14178c))
    - Refactor `FlatGraph` assembly into separate `FlatGraphBuilder` ([`9dd3bd9`](https://github.com/hydro-project/hydroflow/commit/9dd3bd91586966484abaf01c4330d831804b1983))
    - Emit type guards inline, configurable #263 ([`c6510da`](https://github.com/hydro-project/hydroflow/commit/c6510da4b4cb46ec026e3c1c69b5ce29b17c473c))
    - Separate surface doctests by operator ([`851d97d`](https://github.com/hydro-project/hydroflow/commit/851d97de7ba3435bac98264f4b8679973536486a))
    - Add `hydroflow_macr/build.rs` to autogen operator book docs ([`a5de404`](https://github.com/hydro-project/hydroflow/commit/a5de404cd06c10137f7584d152269327c698a65d))
    - Refactor out surface syntax diagnostics (error messages) ([`008425b`](https://github.com/hydro-project/hydroflow/commit/008425bb436042524f540fc05a855f5fa5535c76))
    - Enable partitioned output on hydroflow_parser! ([`4936198`](https://github.com/hydro-project/hydroflow/commit/4936198a2328057fa4115e38d49898bfd18fb3bb))
    - Add more tests, fix surface syntax bugs ([`eb62ef1`](https://github.com/hydro-project/hydroflow/commit/eb62ef1a47ec58abcf6a11745667e00d69df6d93))
    - Fix surface syntax macro import issues on internal doctests and examples ([`d0be1fa`](https://github.com/hydro-project/hydroflow/commit/d0be1fa76443a8adaa5a2d8ccc1f4e8a3db40280))
    - Cleanups, rename `hydroflow_core` to `hydroflow_lang` ([`c8f2b56`](https://github.com/hydro-project/hydroflow/commit/c8f2b56295555c04e8240432ff686d89fccef01c))
    - Make surface syntax macro fail early for better error messages ([`cbf8a62`](https://github.com/hydro-project/hydroflow/commit/cbf8a62ea24131b5092cb91aea5939764e681760))
    - Wip on codegen w/ some code cleanups ([`d29fb7f`](https://github.com/hydro-project/hydroflow/commit/d29fb7fc275c2774be3f5c08b75f12fdaf6970ff))
    - Add #![allow(clippy::explicit_auto_deref)] due to false positives ([`20382f1`](https://github.com/hydro-project/hydroflow/commit/20382f13d9baf49ee896a6c643bb25788aff2db0))
    - Cleanup and rearrange hydroflow_core graph code ([`49476d3`](https://github.com/hydro-project/hydroflow/commit/49476d397e14a8616e8a963451f19f9752befaa6))
    - Separate into FlatGraph and PartitionedGraph ([`13b7830`](https://github.com/hydro-project/hydroflow/commit/13b783098b5b87ff0e1819b5e5fbb16395c9e308))
    - Move hydroflow macro code into hydroflow_core ([`2623c70`](https://github.com/hydro-project/hydroflow/commit/2623c70b80431a0d6a5e3531f93e3248443af03a))
    - Fix unused method and complex type lints ([`70727c0`](https://github.com/hydro-project/hydroflow/commit/70727c04fd6062b9e6c01799dd87f94bada19cd3))
    - Display handoffs separately in mermaid ([`a913dc6`](https://github.com/hydro-project/hydroflow/commit/a913dc67655e5c934d05576e4bf8ecac9551afdf))
    - Add handling of multi-edges, insertion of handoffs ([`dc8f2db`](https://github.com/hydro-project/hydroflow/commit/dc8f2db9e02304964c2f36caa11891c971c385f7))
    - Subgraph partitioning algorithm working ([`cc8c29c`](https://github.com/hydro-project/hydroflow/commit/cc8c29ccb52e662b80989904b32bb7ef8b487c28))
    - Add checker for operator arity ([`8e7f85a`](https://github.com/hydro-project/hydroflow/commit/8e7f85a7681e62354d5640fd95a703247b984bfb))
    - Cleanup old code, add helpful comments ([`0fe0f40`](https://github.com/hydro-project/hydroflow/commit/0fe0f40dd49bcd1164032ea331f06c209de2ce16))
    - Add mermaid rendering of surface syntax ([`09c9647`](https://github.com/hydro-project/hydroflow/commit/09c964784006898825f1a91893dc20c30bc7853f))
    - New parsing with nice error messages ([`b896108`](https://github.com/hydro-project/hydroflow/commit/b896108792a809e4cbc5053d5214a891c37d330b))
    - WIP issues with keeping spans around, idx collision ([`c963290`](https://github.com/hydro-project/hydroflow/commit/c963290d0e838031d2e0b9a29ce89fb2af047629))
    - Parse updated arrow syntax ([`b7f131c`](https://github.com/hydro-project/hydroflow/commit/b7f131ce38cffc6c8491c778500ceb32d44221d8))
    - Implement basic arrow syntax ([`de8ed49`](https://github.com/hydro-project/hydroflow/commit/de8ed492c1220a131052544079085f44266fe87f))
    - Hydroflow_macro boilerplate ([`b2a8b85`](https://github.com/hydro-project/hydroflow/commit/b2a8b853907ee93ad02ceeb39b95da08a0970330))
</details>

