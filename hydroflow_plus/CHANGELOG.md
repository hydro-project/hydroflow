# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.6.1 (2024-04-09)

<csr-id-fc447ffdf8fd1b2189545a991f08588238182f00/>

### Chore

 - <csr-id-fc447ffdf8fd1b2189545a991f08588238182f00/> appease latest nightly clippy
   Also updates `surface_keyed_fold.rs` `test_fold_keyed_infer_basic` test.

### New Features

 - <csr-id-7f68ebf2a23e8e73719229a6f0408bffc7fbe7af/> simplify Location trait to remove lifetimes
 - <csr-id-77f3e5afb9e276d1d6c643574ebac75ed0003939/> simplify lifetime bounds for processes and clusters
   feat(hydroflow_plus): simplify lifetime bounds for processes and
   clusters
   
   This allows `extract` to move the flow builder, which is a prerequisite
   for having developers run the optimizer during deployment as well in
   case it changes the network topology.
 - <csr-id-5b6562662ce3a0dd172ddc1103a591c1c6037e95/> move persist manipulation into a proper optimization
   feat(hydroflow_plus): move persist manipulation into a proper
   optimization
 - <csr-id-cfb3029a6fb0836789db04a7d0d4a1e8b812b629/> add APIs for running optimization passes
   feat(hydroflow_plus): add APIs for running optimization passes

### Bug Fixes

 - <csr-id-2d2c43dc001dbea17d46d73de464c95066b18fa2/> allow BuiltFlow to be cloned even if the deploy flavor can't

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 28 calendar days.
 - 38 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 6 unique issues were worked on: [#1083](https://github.com/hydro-project/hydroflow/issues/1083), [#1098](https://github.com/hydro-project/hydroflow/issues/1098), [#1100](https://github.com/hydro-project/hydroflow/issues/1100), [#1101](https://github.com/hydro-project/hydroflow/issues/1101), [#1107](https://github.com/hydro-project/hydroflow/issues/1107), [#1140](https://github.com/hydro-project/hydroflow/issues/1140)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1083](https://github.com/hydro-project/hydroflow/issues/1083)**
    - Add APIs for running optimization passes ([`cfb3029`](https://github.com/hydro-project/hydroflow/commit/cfb3029a6fb0836789db04a7d0d4a1e8b812b629))
 * **[#1098](https://github.com/hydro-project/hydroflow/issues/1098)**
    - Move persist manipulation into a proper optimization ([`5b65626`](https://github.com/hydro-project/hydroflow/commit/5b6562662ce3a0dd172ddc1103a591c1c6037e95))
 * **[#1100](https://github.com/hydro-project/hydroflow/issues/1100)**
    - Simplify lifetime bounds for processes and clusters ([`77f3e5a`](https://github.com/hydro-project/hydroflow/commit/77f3e5afb9e276d1d6c643574ebac75ed0003939))
 * **[#1101](https://github.com/hydro-project/hydroflow/issues/1101)**
    - Simplify Location trait to remove lifetimes ([`7f68ebf`](https://github.com/hydro-project/hydroflow/commit/7f68ebf2a23e8e73719229a6f0408bffc7fbe7af))
 * **[#1107](https://github.com/hydro-project/hydroflow/issues/1107)**
    - Allow BuiltFlow to be cloned even if the deploy flavor can't ([`2d2c43d`](https://github.com/hydro-project/hydroflow/commit/2d2c43dc001dbea17d46d73de464c95066b18fa2))
 * **[#1140](https://github.com/hydro-project/hydroflow/issues/1140)**
    - Appease latest nightly clippy ([`fc447ff`](https://github.com/hydro-project/hydroflow/commit/fc447ffdf8fd1b2189545a991f08588238182f00))
 * **Uncategorized**
    - Release hydroflow_lang v0.6.2, hydroflow v0.6.2, hydroflow_plus v0.6.1, hydro_deploy v0.6.1, hydro_cli v0.6.1, hydroflow_plus_cli_integration v0.6.1, stageleft_tool v0.1.1 ([`23cfe08`](https://github.com/hydro-project/hydroflow/commit/23cfe0839079aa17d042bbd3976f6d188689d290))
    - Release hydroflow_cli_integration v0.5.2, hydroflow_lang v0.6.1, hydroflow_datalog_core v0.6.1, lattices v0.5.4, hydroflow v0.6.1, stageleft_macro v0.1.1, stageleft v0.2.1, hydroflow_plus v0.6.1, hydro_deploy v0.6.1, hydro_cli v0.6.1, hydroflow_plus_cli_integration v0.6.1, stageleft_tool v0.1.1 ([`cd63f22`](https://github.com/hydro-project/hydroflow/commit/cd63f2258c961a40f0e5dbef20ac329a2d570ad0))
</details>

## v0.6.0 (2024-03-02)

<csr-id-39ab8b0278e9e3fe96552ace0a4ae768a6bc10d8/>

### Chore

 - <csr-id-39ab8b0278e9e3fe96552ace0a4ae768a6bc10d8/> appease various clippy lints

### New Features

 - <csr-id-c1d1b51ee26cc9946af59ac02c040e0a33d15fde/> unify send/demux/tagged APIs
   feat(hydroflow_plus): unify send/demux/tagged APIs
 - <csr-id-eb34ccd13f56e1d07cbae35ead79daeb3b9bad20/> use an IR before lowering to Hydroflow
   Makes it possible to write custom optimization passes.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 2 calendar days.
 - 31 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#1070](https://github.com/hydro-project/hydroflow/issues/1070), [#1080](https://github.com/hydro-project/hydroflow/issues/1080), [#1084](https://github.com/hydro-project/hydroflow/issues/1084)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1070](https://github.com/hydro-project/hydroflow/issues/1070)**
    - Use an IR before lowering to Hydroflow ([`eb34ccd`](https://github.com/hydro-project/hydroflow/commit/eb34ccd13f56e1d07cbae35ead79daeb3b9bad20))
 * **[#1080](https://github.com/hydro-project/hydroflow/issues/1080)**
    - Unify send/demux/tagged APIs ([`c1d1b51`](https://github.com/hydro-project/hydroflow/commit/c1d1b51ee26cc9946af59ac02c040e0a33d15fde))
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

 - <csr-id-5a03ed41548b5766b945efbd1eedb0dfceb714d9/> add core negation operators
 - <csr-id-7d930a2ccf656d3d6bc5db3e22eb63c5fd6d37d1/> add APIs for declaring external ports on clusters
 - <csr-id-73e9b68ec2f5b2627784addcce9fba684848bb55/> implement keyed fold and reduce
 - <csr-id-5e6ebac1a7f128227ae92a8c195235b27532e17a/> add interleaved shortcut when sending from a cluster
 - <csr-id-af6e3be60fdb69ceec1613347910f4dd49980d34/> push down persists and implement Pi example
   Also fixes type inference issues with reduce the same way as we did for fold.
 - <csr-id-6eeb9be9bc4136041a2855f650ae640c478b7fc9/> improve API naming and polish docs
 - <csr-id-44a308f77bddd67b5c51723ac39f3bc10af52553/> tweak naming of windowing operators
 - <csr-id-1edc5ae5b5f70e1390183e8c8eb27eb0ab32196d/> provide simpler API for launching and minimize dependencies
 - <csr-id-b7aafd3c97897db4bff62c4ab0b7480ef9a799e0/> improve API naming and eliminate wire API for builders
 - <csr-id-d288e51f980577510bb2ed45c04554102c4f1e14/> split API for building single-node graphs
 - <csr-id-26f4d6f610b78a75c41b1ae63366d089ad08b322/> require explicit batching for aggregation operators
 - <csr-id-174607d12277d7544d0f42890c9a5da2ff184df4/> support building graphs for symmetric clusters in Hydroflow+
 - <csr-id-9e275824c88b24d060a7de5822e1359959b36b03/> auto-configure Hydro Deploy based on Hydroflow+ plans
 - <csr-id-27dabcf6878576dc3675788ce3381cb25116033a/> add preliminary `send_to` operator for multi-node graphs
 - <csr-id-e5bdd12e32d6ea72fd91a55c12e09f07a0edaa5c/> add initial test using Hydro CLI from Hydroflow+
   This also required a change to Hydroflow core to make it possible to run the dataflow itself on a single thread (using a LocalSet), even if the surrounding runtime is not single-threaded (required to work around deadlocks because we can't use async APIs inside Hydroflow+). This requires us to spawn any Hydroflow tasks (only for `dest_sink` at the moment) right next to when we run the dataflow rather than when the Hydroflow graph is initialized. From a conceptual perspective, this seems _more right_, since now creating a Hydroflow program will not result in any actual tasks running.
   
   In the third PR of this series, I aim to add a new Hydroflow+ operator that will automate the setup of a `dest_sink`/`source_stream` pair that span nodes.
 - <csr-id-05fb1353cf3e0e8c5da9522365150bd78bd3c5f8/> allow Hydroflow+ programs to emit multiple graphs
   This PR adds support for tagging elements of Hydroflow+ graphs with a node ID, an integer which specifies which Hydroflow graph the computation should be emitted to. The generated code includes the Hydroflow graph for each node ID, so that the appropriate graph can be selected at runtime.
   
   At a larger scale, this is a precursor to adding network operators to Hydroflow+, which will allow distributed logic to be described in a single Hydroflow+ program by specifying points at which data is transferred between different graphs.
 - <csr-id-8b635683e5ac3c4ed2d896ae88e2953db1c6312c/> add a functional surface syntax using staging

### Bug Fixes

 - <csr-id-88a17967d0c9e681a04de4b5796f532f4833272c/> persist cluster IDs for broadcast
   I'll follow this up with a unit test for this, but want to get this fixed ASAP first.
 - <csr-id-bd2bf233302e3638c8f4bc9c0460e1a47edc00aa/> rewrite uses of alloc crate in bincode operators
 - <csr-id-2addaed8a8a441bff7acf9a0a265cc09483fd487/> disallow joining streams on different nodes
 - <csr-id-38411ea007d4feb30dd16bdd1505802a111a67d1/> fix spelling of "propagate"

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 25 commits contributed to the release over the course of 76 calendar days.
 - 23 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 20 unique issues were worked on: [#1001](https://github.com/hydro-project/hydroflow/issues/1001), [#1003](https://github.com/hydro-project/hydroflow/issues/1003), [#1004](https://github.com/hydro-project/hydroflow/issues/1004), [#1006](https://github.com/hydro-project/hydroflow/issues/1006), [#1013](https://github.com/hydro-project/hydroflow/issues/1013), [#1021](https://github.com/hydro-project/hydroflow/issues/1021), [#1022](https://github.com/hydro-project/hydroflow/issues/1022), [#1023](https://github.com/hydro-project/hydroflow/issues/1023), [#1035](https://github.com/hydro-project/hydroflow/issues/1035), [#1036](https://github.com/hydro-project/hydroflow/issues/1036), [#899](https://github.com/hydro-project/hydroflow/issues/899), [#976](https://github.com/hydro-project/hydroflow/issues/976), [#978](https://github.com/hydro-project/hydroflow/issues/978), [#981](https://github.com/hydro-project/hydroflow/issues/981), [#982](https://github.com/hydro-project/hydroflow/issues/982), [#984](https://github.com/hydro-project/hydroflow/issues/984), [#989](https://github.com/hydro-project/hydroflow/issues/989), [#991](https://github.com/hydro-project/hydroflow/issues/991), [#993](https://github.com/hydro-project/hydroflow/issues/993), [#995](https://github.com/hydro-project/hydroflow/issues/995)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1001](https://github.com/hydro-project/hydroflow/issues/1001)**
    - Disallow joining streams on different nodes ([`2addaed`](https://github.com/hydro-project/hydroflow/commit/2addaed8a8a441bff7acf9a0a265cc09483fd487))
 * **[#1003](https://github.com/hydro-project/hydroflow/issues/1003)**
    - Provide simpler API for launching and minimize dependencies ([`1edc5ae`](https://github.com/hydro-project/hydroflow/commit/1edc5ae5b5f70e1390183e8c8eb27eb0ab32196d))
 * **[#1004](https://github.com/hydro-project/hydroflow/issues/1004)**
    - Rewrite uses of alloc crate in bincode operators ([`bd2bf23`](https://github.com/hydro-project/hydroflow/commit/bd2bf233302e3638c8f4bc9c0460e1a47edc00aa))
 * **[#1006](https://github.com/hydro-project/hydroflow/issues/1006)**
    - Tweak naming of windowing operators ([`44a308f`](https://github.com/hydro-project/hydroflow/commit/44a308f77bddd67b5c51723ac39f3bc10af52553))
 * **[#1013](https://github.com/hydro-project/hydroflow/issues/1013)**
    - Improve API naming and polish docs ([`6eeb9be`](https://github.com/hydro-project/hydroflow/commit/6eeb9be9bc4136041a2855f650ae640c478b7fc9))
 * **[#1021](https://github.com/hydro-project/hydroflow/issues/1021)**
    - Push down persists and implement Pi example ([`af6e3be`](https://github.com/hydro-project/hydroflow/commit/af6e3be60fdb69ceec1613347910f4dd49980d34))
 * **[#1022](https://github.com/hydro-project/hydroflow/issues/1022)**
    - Add interleaved shortcut when sending from a cluster ([`5e6ebac`](https://github.com/hydro-project/hydroflow/commit/5e6ebac1a7f128227ae92a8c195235b27532e17a))
 * **[#1023](https://github.com/hydro-project/hydroflow/issues/1023)**
    - Implement keyed fold and reduce ([`73e9b68`](https://github.com/hydro-project/hydroflow/commit/73e9b68ec2f5b2627784addcce9fba684848bb55))
 * **[#1035](https://github.com/hydro-project/hydroflow/issues/1035)**
    - Persist cluster IDs for broadcast ([`88a1796`](https://github.com/hydro-project/hydroflow/commit/88a17967d0c9e681a04de4b5796f532f4833272c))
 * **[#1036](https://github.com/hydro-project/hydroflow/issues/1036)**
    - Add core negation operators ([`5a03ed4`](https://github.com/hydro-project/hydroflow/commit/5a03ed41548b5766b945efbd1eedb0dfceb714d9))
 * **[#899](https://github.com/hydro-project/hydroflow/issues/899)**
    - Add a functional surface syntax using staging ([`8b63568`](https://github.com/hydro-project/hydroflow/commit/8b635683e5ac3c4ed2d896ae88e2953db1c6312c))
 * **[#976](https://github.com/hydro-project/hydroflow/issues/976)**
    - Allow Hydroflow+ programs to emit multiple graphs ([`05fb135`](https://github.com/hydro-project/hydroflow/commit/05fb1353cf3e0e8c5da9522365150bd78bd3c5f8))
 * **[#978](https://github.com/hydro-project/hydroflow/issues/978)**
    - Add initial test using Hydro CLI from Hydroflow+ ([`e5bdd12`](https://github.com/hydro-project/hydroflow/commit/e5bdd12e32d6ea72fd91a55c12e09f07a0edaa5c))
 * **[#981](https://github.com/hydro-project/hydroflow/issues/981)**
    - Add preliminary `send_to` operator for multi-node graphs ([`27dabcf`](https://github.com/hydro-project/hydroflow/commit/27dabcf6878576dc3675788ce3381cb25116033a))
 * **[#982](https://github.com/hydro-project/hydroflow/issues/982)**
    - Auto-configure Hydro Deploy based on Hydroflow+ plans ([`9e27582`](https://github.com/hydro-project/hydroflow/commit/9e275824c88b24d060a7de5822e1359959b36b03))
 * **[#984](https://github.com/hydro-project/hydroflow/issues/984)**
    - Support building graphs for symmetric clusters in Hydroflow+ ([`174607d`](https://github.com/hydro-project/hydroflow/commit/174607d12277d7544d0f42890c9a5da2ff184df4))
 * **[#989](https://github.com/hydro-project/hydroflow/issues/989)**
    - Fix spelling of "propagate" ([`38411ea`](https://github.com/hydro-project/hydroflow/commit/38411ea007d4feb30dd16bdd1505802a111a67d1))
 * **[#991](https://github.com/hydro-project/hydroflow/issues/991)**
    - Require explicit batching for aggregation operators ([`26f4d6f`](https://github.com/hydro-project/hydroflow/commit/26f4d6f610b78a75c41b1ae63366d089ad08b322))
 * **[#993](https://github.com/hydro-project/hydroflow/issues/993)**
    - Split API for building single-node graphs ([`d288e51`](https://github.com/hydro-project/hydroflow/commit/d288e51f980577510bb2ed45c04554102c4f1e14))
 * **[#995](https://github.com/hydro-project/hydroflow/issues/995)**
    - Improve API naming and eliminate wire API for builders ([`b7aafd3`](https://github.com/hydro-project/hydroflow/commit/b7aafd3c97897db4bff62c4ab0b7480ef9a799e0))
 * **Uncategorized**
    - Release hydroflow_plus v0.5.1 ([`58d1d71`](https://github.com/hydro-project/hydroflow/commit/58d1d7166f026a8c7a08a23bc1d77045d7e5f2a9))
    - Release stageleft_macro v0.1.0, stageleft v0.1.0, hydroflow_plus v0.5.1 ([`1a48db5`](https://github.com/hydro-project/hydroflow/commit/1a48db5a1ba058a718ac777367bf6eba3a236b7c))
    - Actually committing empty CHANGELOG.md is required ([`3b36020`](https://github.com/hydro-project/hydroflow/commit/3b36020d16792f26da4df3c5b09652a4ab47ec4f))
    - Manually set lockstep-versioned crates (and `lattices`) to version `0.5.1` ([`1b555e5`](https://github.com/hydro-project/hydroflow/commit/1b555e57c8c812bed4d6495d2960cbf77fb0b3ef))
    - Add APIs for declaring external ports on clusters ([`7d930a2`](https://github.com/hydro-project/hydroflow/commit/7d930a2ccf656d3d6bc5db3e22eb63c5fd6d37d1))
</details>

