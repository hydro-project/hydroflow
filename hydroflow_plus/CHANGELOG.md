# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.10.0 (2024-11-08)

### Chore

 - <csr-id-d5677604e93c07a5392f4229af94a0b736eca382/> update pinned rust version, clippy lints, remove some dead code

### New Features

 - <csr-id-baedf23eaa056bc0dad8331d116bb71176764206/> improve quickstart ergonomics
 - <csr-id-98a21e36bd50d312402e46357fea6330816d0139/> add utility to dedup tees when debugging IR
 - <csr-id-2141c5f04cb7e9cb7cd2f50f849f6c4b3d745377/> add decouple and simple test and two_pc
 - <csr-id-074f2cf76158a126370a7e6b184bc6b928eb6fe2/> implement support for external network outputs
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
 - <csr-id-8a809315cd37929687fcabc34a12042db25d5767/> add API for external network inputs
   This is a key step towards being able to unit-test HF+ graphs, by being
   able to have controlled inputs. Outputs next.
 - <csr-id-60d9becaf0b67f9819316ce6d76bd867f7d46505/> splice UDFs with type hints to avoid inference failures

### Bug Fixes

 - <csr-id-2faffdbf2cc886da22e496df64f46aefa380766c/> properly handle `crate::` imports
 - <csr-id-275a0edf1fb8eba467728c24edf3a984c8eaca75/> be more careful about which parts of proposer and acceptor have to be maintained atomically
 - <csr-id-87a68346aa10051d9d205d791407ce85546802da/> adjust default features to allow compilation to musl targets
   Previously, the default `deploy` feature would pull in Hydro Deploy and
   its transitive native dependencies.
   
   Also sets up `examples/paxos.rs` with CLI flags to deploy to GCP.
 - <csr-id-d4320e311562a004c01342a2b0f03ab6e2520562/> add missing `sample_every` for singletons
   Discovered during a live-coding demo, we only had it for optionals
   before.

### Refactor

 - <csr-id-a1b45203178165683cb4b5ae611c598cc9c14853/> move rewrites to a submodule
 - <csr-id-e9d05bf11a0e85da8ed1a0fe00be7769298308c2/> move `HfCompiled` and friends to a module
 - <csr-id-9f744052dd4ac744f5a1baa4e0cb9253adaeba1b/> use `location.flow_state()` to avoid clone
 - <csr-id-5b819a2dc6c507222a3e22d71efcde8b43cebad5/> deduplicate some error messages and drop unused `Interval` IR node
 - <csr-id-244207c2acd2243ece6e787d54eadacf06e9e8bb/> dedup signatures for `Stream` operators
 - <csr-id-bf9dcd5a923dd4b5efa337a9127086e5609a1722/> clean up traits for cycles and forward references
 - <csr-id-d9634f242a97c06bdb53011bf3d75256425a1598/> split up location module and store locations directly in streams
 - <csr-id-534fe974101e38ecb847cd759dbaf503ff97f822/> use `usize` for slot numbers
 - <csr-id-0a5abab3dac224c9591bcdd837d07c6e5c2773c6/> make Paxos-KV generic
 - <csr-id-38b17cd977fb6c00ddc37e7a5b30e45dba17329e/> simplify latency calculations
 - <csr-id-8b7b1c60fd33b78f9a4b0873bbbd150260ae2ad5/> complete split into leader election and sequencing phases
 - <csr-id-dff2a40669736014349cf12744d6a057a7992e11/> start splitting out leader election into a separate module
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1485).
   * #1493
   * #1492
   * #1489
   * #1488
   * #1487
   * #1486
   * __->__ #1485
 - <csr-id-1b18b358c87caa37a6519612131c8674653a2407/> simplify `persist_pullup` code
   Instead of matching on `&mut` and juggling ownership, instead match on
   the owned node and always replaced `*node = new_node` (sometimes itself)
 - <csr-id-c752affc2ee2c5d82d19dd992f6a89b7070b8773/> use max and min in Paxos and make client generic over ballots

### Style

 - <csr-id-47cb703e771f7d1c451ceb9d185ada96410949da/> fixes for nightly clippy
   a couple few spurious `too_many_arguments` and a spurious
   `zombie_processes` still on current nightly (`clippy 0.1.84 (4392847410
   2024-10-21)`)

### New Features (BREAKING)

 - <csr-id-8d8b4b2288746e0aa2a95329d91297820aee7586/> implicitly apply default optimizations
   This also changes the behavior of `with_default_optimize` to be
   terminal, if users want to apply optimizations after these they should
   explicitly invoke the optimizations.
 - <csr-id-5d5209b4a5556618d8a8c8219e1e2a4e837256ef/> add an explicit API for creating tick contexts
   Previously, each location had a (semantic) global clock that drives
   ticks, and so all streams in a tick domain were all in the same atomic
   block. For future optimizations, we'd like developers to be able to
   place streams on the same location into different clocks to eliminate
   synchronization between them, which in turn would allow the computations
   in those separate clocks to be potentially decoupled across machines.
 - <csr-id-edd86496240e4ebb39e0cf3bc153d8f282ff2870/> strongly-typed runtime cluster IDs
   Instead of `u32`s everywhere, we now have a `ClusterId<C>` type that
   ensures that cluster IDs are not misused.
 - <csr-id-4f3b51b4b9187f1187be23e6f04034778fe76388/> provide an API for creating cycles across tick iterations
   Towards making it more clear which parts of a program depend on ticks
   versus don't.

### Refactor (BREAKING)

 - <csr-id-0bd3a2d2230cbef24210f71a3ea83d82d1cc7244/> eliminate remaining `Hf` name prefixes
 - <csr-id-9107841700db0ae72de6269ab6f132be0ae51cd9/> location type parameter before boundedness
   When looking at a prefix in an IDE, the location type argument is
   generally more useful.
 - <csr-id-919099ea3a414560b473ec89b993eeb26dfa2579/> dedup signatures for `Singleton` and `Optional`
   Also renames `cross_singleton` to `zip` when both sides are
   singleton-like.
 - <csr-id-5657563c989566e7c7b69dcb395e40b024c83c6c/> fold `Tick` vs `NoTick` into the location type parameter
   Now, when the location is a top-level `Process` or `Cluster` that
   corresponds to a `NoTick`, and for streams inside a tick we wrap the
   location type (e.g. `Tick<Process<...>>`). This simplifies type
   signatures for a lot of our example code.
 - <csr-id-e5b456bdafcb80aae6039e4c90a2e60098e499bf/> simplify intervals and split Paxos-KV into separate module
 - <csr-id-30c4f708faff7875ab42e551dd4bccbe231dfdad/> move input APIs back to being on locations
 - <csr-id-8ad997b2dfd23bb09f7d361d763d6b5e78f406d6/> move `self_id` and `members` to be APIs on cluster instead of builder

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 38 commits contributed to the release.
 - 69 days passed between releases.
 - 38 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 38 unique issues were worked on: [#1434](https://github.com/hydro-project/hydroflow/issues/1434), [#1441](https://github.com/hydro-project/hydroflow/issues/1441), [#1443](https://github.com/hydro-project/hydroflow/issues/1443), [#1444](https://github.com/hydro-project/hydroflow/issues/1444), [#1449](https://github.com/hydro-project/hydroflow/issues/1449), [#1450](https://github.com/hydro-project/hydroflow/issues/1450), [#1451](https://github.com/hydro-project/hydroflow/issues/1451), [#1453](https://github.com/hydro-project/hydroflow/issues/1453), [#1455](https://github.com/hydro-project/hydroflow/issues/1455), [#1461](https://github.com/hydro-project/hydroflow/issues/1461), [#1464](https://github.com/hydro-project/hydroflow/issues/1464), [#1468](https://github.com/hydro-project/hydroflow/issues/1468), [#1471](https://github.com/hydro-project/hydroflow/issues/1471), [#1477](https://github.com/hydro-project/hydroflow/issues/1477), [#1485](https://github.com/hydro-project/hydroflow/issues/1485), [#1486](https://github.com/hydro-project/hydroflow/issues/1486), [#1488](https://github.com/hydro-project/hydroflow/issues/1488), [#1491](https://github.com/hydro-project/hydroflow/issues/1491), [#1505](https://github.com/hydro-project/hydroflow/issues/1505), [#1515](https://github.com/hydro-project/hydroflow/issues/1515), [#1516](https://github.com/hydro-project/hydroflow/issues/1516), [#1517](https://github.com/hydro-project/hydroflow/issues/1517), [#1519](https://github.com/hydro-project/hydroflow/issues/1519), [#1521](https://github.com/hydro-project/hydroflow/issues/1521), [#1523](https://github.com/hydro-project/hydroflow/issues/1523), [#1524](https://github.com/hydro-project/hydroflow/issues/1524), [#1525](https://github.com/hydro-project/hydroflow/issues/1525), [#1526](https://github.com/hydro-project/hydroflow/issues/1526), [#1527](https://github.com/hydro-project/hydroflow/issues/1527), [#1540](https://github.com/hydro-project/hydroflow/issues/1540), [#1541](https://github.com/hydro-project/hydroflow/issues/1541), [#1542](https://github.com/hydro-project/hydroflow/issues/1542), [#1543](https://github.com/hydro-project/hydroflow/issues/1543), [#1550](https://github.com/hydro-project/hydroflow/issues/1550), [#1551](https://github.com/hydro-project/hydroflow/issues/1551), [#1553](https://github.com/hydro-project/hydroflow/issues/1553), [#1554](https://github.com/hydro-project/hydroflow/issues/1554), [#1557](https://github.com/hydro-project/hydroflow/issues/1557)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1434](https://github.com/hydro-project/hydroflow/issues/1434)**
    - Splice UDFs with type hints to avoid inference failures ([`60d9bec`](https://github.com/hydro-project/hydroflow/commit/60d9becaf0b67f9819316ce6d76bd867f7d46505))
 * **[#1441](https://github.com/hydro-project/hydroflow/issues/1441)**
    - Provide an API for creating cycles across tick iterations ([`4f3b51b`](https://github.com/hydro-project/hydroflow/commit/4f3b51b4b9187f1187be23e6f04034778fe76388))
 * **[#1443](https://github.com/hydro-project/hydroflow/issues/1443)**
    - Use max and min in Paxos and make client generic over ballots ([`c752aff`](https://github.com/hydro-project/hydroflow/commit/c752affc2ee2c5d82d19dd992f6a89b7070b8773))
 * **[#1444](https://github.com/hydro-project/hydroflow/issues/1444)**
    - Update pinned rust version, clippy lints, remove some dead code ([`d567760`](https://github.com/hydro-project/hydroflow/commit/d5677604e93c07a5392f4229af94a0b736eca382))
 * **[#1449](https://github.com/hydro-project/hydroflow/issues/1449)**
    - Add API for external network inputs ([`8a80931`](https://github.com/hydro-project/hydroflow/commit/8a809315cd37929687fcabc34a12042db25d5767))
 * **[#1450](https://github.com/hydro-project/hydroflow/issues/1450)**
    - Add ability to have staged flows inside unit tests ([`afe78c3`](https://github.com/hydro-project/hydroflow/commit/afe78c343658472513b34d28658634b253148aee))
 * **[#1451](https://github.com/hydro-project/hydroflow/issues/1451)**
    - Implement support for external network outputs ([`074f2cf`](https://github.com/hydro-project/hydroflow/commit/074f2cf76158a126370a7e6b184bc6b928eb6fe2))
 * **[#1453](https://github.com/hydro-project/hydroflow/issues/1453)**
    - Add decouple and simple test and two_pc ([`2141c5f`](https://github.com/hydro-project/hydroflow/commit/2141c5f04cb7e9cb7cd2f50f849f6c4b3d745377))
 * **[#1455](https://github.com/hydro-project/hydroflow/issues/1455)**
    - Simplify `persist_pullup` code ([`1b18b35`](https://github.com/hydro-project/hydroflow/commit/1b18b358c87caa37a6519612131c8674653a2407))
 * **[#1461](https://github.com/hydro-project/hydroflow/issues/1461)**
    - Add missing `sample_every` for singletons ([`d4320e3`](https://github.com/hydro-project/hydroflow/commit/d4320e311562a004c01342a2b0f03ab6e2520562))
 * **[#1464](https://github.com/hydro-project/hydroflow/issues/1464)**
    - Adjust default features to allow compilation to musl targets ([`87a6834`](https://github.com/hydro-project/hydroflow/commit/87a68346aa10051d9d205d791407ce85546802da))
 * **[#1468](https://github.com/hydro-project/hydroflow/issues/1468)**
    - Move `self_id` and `members` to be APIs on cluster instead of builder ([`8ad997b`](https://github.com/hydro-project/hydroflow/commit/8ad997b2dfd23bb09f7d361d763d6b5e78f406d6))
 * **[#1471](https://github.com/hydro-project/hydroflow/issues/1471)**
    - Move input APIs back to being on locations ([`30c4f70`](https://github.com/hydro-project/hydroflow/commit/30c4f708faff7875ab42e551dd4bccbe231dfdad))
 * **[#1477](https://github.com/hydro-project/hydroflow/issues/1477)**
    - Strongly-typed runtime cluster IDs ([`edd8649`](https://github.com/hydro-project/hydroflow/commit/edd86496240e4ebb39e0cf3bc153d8f282ff2870))
 * **[#1485](https://github.com/hydro-project/hydroflow/issues/1485)**
    - Start splitting out leader election into a separate module ([`dff2a40`](https://github.com/hydro-project/hydroflow/commit/dff2a40669736014349cf12744d6a057a7992e11))
 * **[#1486](https://github.com/hydro-project/hydroflow/issues/1486)**
    - Complete split into leader election and sequencing phases ([`8b7b1c6`](https://github.com/hydro-project/hydroflow/commit/8b7b1c60fd33b78f9a4b0873bbbd150260ae2ad5))
 * **[#1488](https://github.com/hydro-project/hydroflow/issues/1488)**
    - Be more careful about which parts of proposer and acceptor have to be maintained atomically ([`275a0ed`](https://github.com/hydro-project/hydroflow/commit/275a0edf1fb8eba467728c24edf3a984c8eaca75))
 * **[#1491](https://github.com/hydro-project/hydroflow/issues/1491)**
    - Add utility to dedup tees when debugging IR ([`98a21e3`](https://github.com/hydro-project/hydroflow/commit/98a21e36bd50d312402e46357fea6330816d0139))
 * **[#1505](https://github.com/hydro-project/hydroflow/issues/1505)**
    - Fixes for nightly clippy ([`47cb703`](https://github.com/hydro-project/hydroflow/commit/47cb703e771f7d1c451ceb9d185ada96410949da))
 * **[#1515](https://github.com/hydro-project/hydroflow/issues/1515)**
    - Simplify latency calculations ([`38b17cd`](https://github.com/hydro-project/hydroflow/commit/38b17cd977fb6c00ddc37e7a5b30e45dba17329e))
 * **[#1516](https://github.com/hydro-project/hydroflow/issues/1516)**
    - Simplify intervals and split Paxos-KV into separate module ([`e5b456b`](https://github.com/hydro-project/hydroflow/commit/e5b456bdafcb80aae6039e4c90a2e60098e499bf))
 * **[#1517](https://github.com/hydro-project/hydroflow/issues/1517)**
    - Make Paxos-KV generic ([`0a5abab`](https://github.com/hydro-project/hydroflow/commit/0a5abab3dac224c9591bcdd837d07c6e5c2773c6))
 * **[#1519](https://github.com/hydro-project/hydroflow/issues/1519)**
    - Fold `Tick` vs `NoTick` into the location type parameter ([`5657563`](https://github.com/hydro-project/hydroflow/commit/5657563c989566e7c7b69dcb395e40b024c83c6c))
 * **[#1521](https://github.com/hydro-project/hydroflow/issues/1521)**
    - Use `usize` for slot numbers ([`534fe97`](https://github.com/hydro-project/hydroflow/commit/534fe974101e38ecb847cd759dbaf503ff97f822))
 * **[#1523](https://github.com/hydro-project/hydroflow/issues/1523)**
    - Split up location module and store locations directly in streams ([`d9634f2`](https://github.com/hydro-project/hydroflow/commit/d9634f242a97c06bdb53011bf3d75256425a1598))
 * **[#1524](https://github.com/hydro-project/hydroflow/issues/1524)**
    - Clean up traits for cycles and forward references ([`bf9dcd5`](https://github.com/hydro-project/hydroflow/commit/bf9dcd5a923dd4b5efa337a9127086e5609a1722))
 * **[#1525](https://github.com/hydro-project/hydroflow/issues/1525)**
    - Dedup signatures for `Stream` operators ([`244207c`](https://github.com/hydro-project/hydroflow/commit/244207c2acd2243ece6e787d54eadacf06e9e8bb))
 * **[#1526](https://github.com/hydro-project/hydroflow/issues/1526)**
    - Dedup signatures for `Singleton` and `Optional` ([`919099e`](https://github.com/hydro-project/hydroflow/commit/919099ea3a414560b473ec89b993eeb26dfa2579))
 * **[#1527](https://github.com/hydro-project/hydroflow/issues/1527)**
    - Properly handle `crate::` imports ([`2faffdb`](https://github.com/hydro-project/hydroflow/commit/2faffdbf2cc886da22e496df64f46aefa380766c))
 * **[#1540](https://github.com/hydro-project/hydroflow/issues/1540)**
    - Deduplicate some error messages and drop unused `Interval` IR node ([`5b819a2`](https://github.com/hydro-project/hydroflow/commit/5b819a2dc6c507222a3e22d71efcde8b43cebad5))
 * **[#1541](https://github.com/hydro-project/hydroflow/issues/1541)**
    - Use `location.flow_state()` to avoid clone ([`9f74405`](https://github.com/hydro-project/hydroflow/commit/9f744052dd4ac744f5a1baa4e0cb9253adaeba1b))
 * **[#1542](https://github.com/hydro-project/hydroflow/issues/1542)**
    - Move `HfCompiled` and friends to a module ([`e9d05bf`](https://github.com/hydro-project/hydroflow/commit/e9d05bf11a0e85da8ed1a0fe00be7769298308c2))
 * **[#1543](https://github.com/hydro-project/hydroflow/issues/1543)**
    - Move rewrites to a submodule ([`a1b4520`](https://github.com/hydro-project/hydroflow/commit/a1b45203178165683cb4b5ae611c598cc9c14853))
 * **[#1550](https://github.com/hydro-project/hydroflow/issues/1550)**
    - Add an explicit API for creating tick contexts ([`5d5209b`](https://github.com/hydro-project/hydroflow/commit/5d5209b4a5556618d8a8c8219e1e2a4e837256ef))
 * **[#1551](https://github.com/hydro-project/hydroflow/issues/1551)**
    - Location type parameter before boundedness ([`9107841`](https://github.com/hydro-project/hydroflow/commit/9107841700db0ae72de6269ab6f132be0ae51cd9))
 * **[#1553](https://github.com/hydro-project/hydroflow/issues/1553)**
    - Improve quickstart ergonomics ([`baedf23`](https://github.com/hydro-project/hydroflow/commit/baedf23eaa056bc0dad8331d116bb71176764206))
 * **[#1554](https://github.com/hydro-project/hydroflow/issues/1554)**
    - Eliminate remaining `Hf` name prefixes ([`0bd3a2d`](https://github.com/hydro-project/hydroflow/commit/0bd3a2d2230cbef24210f71a3ea83d82d1cc7244))
 * **[#1557](https://github.com/hydro-project/hydroflow/issues/1557)**
    - Implicitly apply default optimizations ([`8d8b4b2`](https://github.com/hydro-project/hydroflow/commit/8d8b4b2288746e0aa2a95329d91297820aee7586))
</details>

## v0.9.0 (2024-08-30)

<csr-id-11af32828bab6e4a4264d2635ff71a12bb0bb778/>
<csr-id-0a465e55dd39c76bc1aefb020460a639d792fe87/>
<csr-id-5f2789a13d1602f170e678fe9bbc59caf69db4b5/>
<csr-id-09d6d44eafc866881e73719813fe9edeb49ca2a6/>
<csr-id-fa417205569d8c49c85b0c2324118e0f9b1c8407/>

### Chore

 - <csr-id-11af32828bab6e4a4264d2635ff71a12bb0bb778/> lower min dependency versions where possible, update `Cargo.lock`
   Moved from #1418
   
   ---------

### Refactor (BREAKING)

 - <csr-id-128aaecd40edce57dc254afdcd61ecd5b9948d71/> simplify process/cluster specs
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1394).
   * #1395
   * __->__ #1394
 - <csr-id-0eba702f62e7a6816cf931b01a2ea5643bd7321d/> defer network instantiation until after finalizing IR
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1377).
   * #1395
   * #1394
   * __->__ #1377

### Documentation

 - <csr-id-f5f1eb0c612f5c0c1752360d972ef6853c5e12f0/> cleanup doc comments for clippy latest

### New Features

 - <csr-id-71f69aa5e9f2ba187f07c44c0a9f2becfe72aab1/> add API for cycle with initial value
 - <csr-id-82de6f5fc89fd44fd2ac18fddd94d121b4b10c8a/> add unbounded top-level singletons
 - <csr-id-7bf9ee2f707ddd5d8f51853ab7babe035fd8d964/> add paxos
 - <csr-id-46a8a2cb08732bb21096e824bc4542d208c68fb2/> use trybuild to compile subgraph binaries
 - <csr-id-eaf497b601928be37530bc8d81717d200fd5987a/> add operators necessary for Paxos / PBFT

### Bug Fixes

<csr-id-b518e674560971ebd1b32c737151214b8d3310b0/>
<csr-id-c12b2495c70f170eba655e458f4591ef7d0941a4/>
<csr-id-ab12e5b66718f06adc3c34bf879c9581d79ee0d2/>

 - <csr-id-22c72189bb76412955d29b03c5d99894c558a07c/> remove `FlowProps`
 - <csr-id-1aeacb212227f654e8f0cdc8a59816a68f059177/> rewrite IR in place to avoid stack overflow and disable cloning
   Cloning was unsafe because values behind a `Rc<RefCell<...>>` in the
   case of tee would be entangled with the old IR.
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1404).
   * #1405
   * #1398
   * __->__ #1404
* #1398
* __->__ #1404
 - <csr-id-1aeacb212227f654e8f0cdc8a59816a68f059177/> rewrite IR in place to avoid stack overflow and disable cloning
   Cloning was unsafe because values behind a `Rc<RefCell<...>>` in the
   case of tee would be entangled with the old IR.
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1404).
   * #1405
   * #1398
   * __->__ #1404

### New Features (BREAKING)

 - <csr-id-44c6b149bea102e8598460ba0286e370b36fd25a/> separate singletons into their own types
 - <csr-id-536e6442d68b0947da5bfef9991825003e6867fc/> refactor API to have no-tick semantics by default
   Now, by default streams exist at a "top-level" where there are no ticks
   and operators run over the entire collection. To perform iterative
   computations, developers must explicitly entire a tick domain (using
   `tick_batch`), and return to the outer domain (using `all_ticks`).

### Refactor (BREAKING)

 - <csr-id-0a465e55dd39c76bc1aefb020460a639d792fe87/> rename integration crates to drop CLI references
 - <csr-id-5f2789a13d1602f170e678fe9bbc59caf69db4b5/> disentangle instantiated nodes from locations
 - <csr-id-128aaecd40edce57dc254afdcd61ecd5b9948d71/> simplify process/cluster specs
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1394).
   * #1395
   * __->__ #1394
 - <csr-id-0eba702f62e7a6816cf931b01a2ea5643bd7321d/> defer network instantiation until after finalizing IR
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1377).
   * #1395
   * #1394
   * __->__ #1377
 - <csr-id-09d6d44eafc866881e73719813fe9edeb49ca2a6/> start rearranging stages of flow compilation to prepare for trybuild approach

<csr-id-128aaecd40edce57dc254afdcd61ecd5b9948d71/> simplify process/cluster specs
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1394).
   * #1395
   * __->__ #1394
 - <csr-id-0eba702f62e7a6816cf931b01a2ea5643bd7321d/> defer network instantiation until after finalizing IR
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1377).
   * #1395
   * #1394
   * __->__ #1377

<csr-id-0eba702f62e7a6816cf931b01a2ea5643bd7321d/> defer network instantiation until after finalizing IR
   ---
   [//]: # (BEGIN SAPLING FOOTER)
   Stack created with [Sapling](https://sapling-scm.com). Best reviewed
   with
   [ReviewStack](https://reviewstack.dev/hydro-project/hydroflow/pull/1377).
   * #1395
   * #1394
   * __->__ #1377
 - <csr-id-09d6d44eafc866881e73719813fe9edeb49ca2a6/> start rearranging stages of flow compilation to prepare for trybuild approach

### Style (BREAKING)

 - <csr-id-fa417205569d8c49c85b0c2324118e0f9b1c8407/> rename some `CLI`->`Deploy`, decapitalize acronym names

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 21 commits contributed to the release.
 - 38 days passed between releases.
 - 20 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 20 unique issues were worked on: [#1358](https://github.com/hydro-project/hydroflow/issues/1358), [#1368](https://github.com/hydro-project/hydroflow/issues/1368), [#1375](https://github.com/hydro-project/hydroflow/issues/1375), [#1376](https://github.com/hydro-project/hydroflow/issues/1376), [#1377](https://github.com/hydro-project/hydroflow/issues/1377), [#1394](https://github.com/hydro-project/hydroflow/issues/1394), [#1395](https://github.com/hydro-project/hydroflow/issues/1395), [#1398](https://github.com/hydro-project/hydroflow/issues/1398), [#1399](https://github.com/hydro-project/hydroflow/issues/1399), [#1404](https://github.com/hydro-project/hydroflow/issues/1404), [#1405](https://github.com/hydro-project/hydroflow/issues/1405), [#1410](https://github.com/hydro-project/hydroflow/issues/1410), [#1413](https://github.com/hydro-project/hydroflow/issues/1413), [#1420](https://github.com/hydro-project/hydroflow/issues/1420), [#1421](https://github.com/hydro-project/hydroflow/issues/1421), [#1423](https://github.com/hydro-project/hydroflow/issues/1423), [#1425](https://github.com/hydro-project/hydroflow/issues/1425), [#1427](https://github.com/hydro-project/hydroflow/issues/1427), [#1428](https://github.com/hydro-project/hydroflow/issues/1428), [#1430](https://github.com/hydro-project/hydroflow/issues/1430)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1358](https://github.com/hydro-project/hydroflow/issues/1358)**
    - Start rearranging stages of flow compilation to prepare for trybuild approach ([`09d6d44`](https://github.com/hydro-project/hydroflow/commit/09d6d44eafc866881e73719813fe9edeb49ca2a6))
 * **[#1368](https://github.com/hydro-project/hydroflow/issues/1368)**
    - Overly restrictive input types for `send_bincode_interleaved` ([`ab12e5b`](https://github.com/hydro-project/hydroflow/commit/ab12e5b66718f06adc3c34bf879c9581d79ee0d2))
 * **[#1375](https://github.com/hydro-project/hydroflow/issues/1375)**
    - Add `Clone` bounds to `cross_join` and simplify broadcast logic ([`c12b249`](https://github.com/hydro-project/hydroflow/commit/c12b2495c70f170eba655e458f4591ef7d0941a4))
 * **[#1376](https://github.com/hydro-project/hydroflow/issues/1376)**
    - Add operators necessary for Paxos / PBFT ([`eaf497b`](https://github.com/hydro-project/hydroflow/commit/eaf497b601928be37530bc8d81717d200fd5987a))
 * **[#1377](https://github.com/hydro-project/hydroflow/issues/1377)**
    - Defer network instantiation until after finalizing IR ([`0eba702`](https://github.com/hydro-project/hydroflow/commit/0eba702f62e7a6816cf931b01a2ea5643bd7321d))
 * **[#1394](https://github.com/hydro-project/hydroflow/issues/1394)**
    - Simplify process/cluster specs ([`128aaec`](https://github.com/hydro-project/hydroflow/commit/128aaecd40edce57dc254afdcd61ecd5b9948d71))
 * **[#1395](https://github.com/hydro-project/hydroflow/issues/1395)**
    - Disentangle instantiated nodes from locations ([`5f2789a`](https://github.com/hydro-project/hydroflow/commit/5f2789a13d1602f170e678fe9bbc59caf69db4b5))
 * **[#1398](https://github.com/hydro-project/hydroflow/issues/1398)**
    - Use trybuild to compile subgraph binaries ([`46a8a2c`](https://github.com/hydro-project/hydroflow/commit/46a8a2cb08732bb21096e824bc4542d208c68fb2))
 * **[#1399](https://github.com/hydro-project/hydroflow/issues/1399)**
    - Rename some `CLI`->`Deploy`, decapitalize acronym names ([`fa41720`](https://github.com/hydro-project/hydroflow/commit/fa417205569d8c49c85b0c2324118e0f9b1c8407))
 * **[#1404](https://github.com/hydro-project/hydroflow/issues/1404)**
    - Rewrite IR in place to avoid stack overflow and disable cloning ([`1aeacb2`](https://github.com/hydro-project/hydroflow/commit/1aeacb212227f654e8f0cdc8a59816a68f059177))
 * **[#1405](https://github.com/hydro-project/hydroflow/issues/1405)**
    - Wrong stream type for `source_interval` ([`b518e67`](https://github.com/hydro-project/hydroflow/commit/b518e674560971ebd1b32c737151214b8d3310b0))
 * **[#1410](https://github.com/hydro-project/hydroflow/issues/1410)**
    - Add paxos ([`7bf9ee2`](https://github.com/hydro-project/hydroflow/commit/7bf9ee2f707ddd5d8f51853ab7babe035fd8d964))
 * **[#1413](https://github.com/hydro-project/hydroflow/issues/1413)**
    - Rename integration crates to drop CLI references ([`0a465e5`](https://github.com/hydro-project/hydroflow/commit/0a465e55dd39c76bc1aefb020460a639d792fe87))
 * **[#1420](https://github.com/hydro-project/hydroflow/issues/1420)**
    - Remove `FlowProps` ([`22c7218`](https://github.com/hydro-project/hydroflow/commit/22c72189bb76412955d29b03c5d99894c558a07c))
 * **[#1421](https://github.com/hydro-project/hydroflow/issues/1421)**
    - Refactor API to have no-tick semantics by default ([`536e644`](https://github.com/hydro-project/hydroflow/commit/536e6442d68b0947da5bfef9991825003e6867fc))
 * **[#1423](https://github.com/hydro-project/hydroflow/issues/1423)**
    - Lower min dependency versions where possible, update `Cargo.lock` ([`11af328`](https://github.com/hydro-project/hydroflow/commit/11af32828bab6e4a4264d2635ff71a12bb0bb778))
 * **[#1425](https://github.com/hydro-project/hydroflow/issues/1425)**
    - Separate singletons into their own types ([`44c6b14`](https://github.com/hydro-project/hydroflow/commit/44c6b149bea102e8598460ba0286e370b36fd25a))
 * **[#1427](https://github.com/hydro-project/hydroflow/issues/1427)**
    - Add unbounded top-level singletons ([`82de6f5`](https://github.com/hydro-project/hydroflow/commit/82de6f5fc89fd44fd2ac18fddd94d121b4b10c8a))
 * **[#1428](https://github.com/hydro-project/hydroflow/issues/1428)**
    - Cleanup doc comments for clippy latest ([`f5f1eb0`](https://github.com/hydro-project/hydroflow/commit/f5f1eb0c612f5c0c1752360d972ef6853c5e12f0))
 * **[#1430](https://github.com/hydro-project/hydroflow/issues/1430)**
    - Add API for cycle with initial value ([`71f69aa`](https://github.com/hydro-project/hydroflow/commit/71f69aa5e9f2ba187f07c44c0a9f2becfe72aab1))
 * **Uncategorized**
    - Release hydroflow_lang v0.9.0, hydroflow_datalog_core v0.9.0, hydroflow_datalog v0.9.0, hydroflow_deploy_integration v0.9.0, hydroflow_macro v0.9.0, lattices_macro v0.5.6, lattices v0.5.7, multiplatform_test v0.2.0, variadics v0.0.6, pusherator v0.0.8, hydroflow v0.9.0, stageleft_macro v0.3.0, stageleft v0.4.0, stageleft_tool v0.3.0, hydroflow_plus v0.9.0, hydro_deploy v0.9.0, hydro_cli v0.9.0, hydroflow_plus_deploy v0.9.0, safety bump 8 crates ([`0750117`](https://github.com/hydro-project/hydroflow/commit/0750117de7088c01a439b102adeb4c832889f171))
</details>

<csr-unknown>
 wrong stream type for source_interval add Clone bounds to cross_join and simplify broadcast logic overly restrictive input types for send_bincode_interleavedThe original types prevented usage in cluster-to-cluster communication.<csr-unknown/>

## v0.8.0 (2024-07-23)

<csr-id-67c0e51fb25ea1a2e3aae197c1984920b46759fa/>

### Reverted

 - <csr-id-256779abece03bee662b351430d27141d10bd5ef/> "feat(hydroflow): Added poll_futures and poll_futures_async operators.", fix #1183
   This reverts commit 997d90a76db9a4e05dbac35073a09548750ce342.
   
   We have been trying to figure out the semantics a bit, and want to give
   it more thought before we commit to maintaining it
   
   Can un-revert and adjust the semantics later when we use it

### Refactor (BREAKING)

 - <csr-id-67c0e51fb25ea1a2e3aae197c1984920b46759fa/> require lifetime on `perist*()` operators

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 59 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 3 unique issues were worked on: [#1143](https://github.com/hydro-project/hydroflow/issues/1143), [#1216](https://github.com/hydro-project/hydroflow/issues/1216), [#1295](https://github.com/hydro-project/hydroflow/issues/1295)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1143](https://github.com/hydro-project/hydroflow/issues/1143)**
    - "feat(hydroflow): Added poll_futures and poll_futures_async operators.", fix #1183 ([`256779a`](https://github.com/hydro-project/hydroflow/commit/256779abece03bee662b351430d27141d10bd5ef))
 * **[#1216](https://github.com/hydro-project/hydroflow/issues/1216)**
    - "feat(hydroflow): Added poll_futures and poll_futures_async operators.", fix #1183 ([`256779a`](https://github.com/hydro-project/hydroflow/commit/256779abece03bee662b351430d27141d10bd5ef))
 * **[#1295](https://github.com/hydro-project/hydroflow/issues/1295)**
    - Require lifetime on `perist*()` operators ([`67c0e51`](https://github.com/hydro-project/hydroflow/commit/67c0e51fb25ea1a2e3aae197c1984920b46759fa))
 * **Uncategorized**
    - Release hydroflow_lang v0.8.0, hydroflow_datalog_core v0.8.0, hydroflow_datalog v0.8.0, hydroflow_macro v0.8.0, lattices_macro v0.5.5, lattices v0.5.6, variadics v0.0.5, pusherator v0.0.7, hydroflow v0.8.0, hydroflow_plus v0.8.0, hydro_deploy v0.8.0, hydro_cli v0.8.0, hydroflow_plus_cli_integration v0.8.0, safety bump 7 crates ([`ca6c16b`](https://github.com/hydro-project/hydroflow/commit/ca6c16b4a7ce35e155fe7fc6c7d1676c37c9e4de))
</details>

## v0.7.0 (2024-05-24)

<csr-id-c9dfddc680e0ce5415539d7b77bc5beb97ab59d9/>

### Chore

 - <csr-id-c9dfddc680e0ce5415539d7b77bc5beb97ab59d9/> use workaround for `cargo smart-release` not properly ordering `dev-`/`build-dependencies`

### New Features

 - <csr-id-6e571726ff40818fbe9bbe9923511877c20fb243/> add API to get the cluster ID of the current node
   feat(hydroflow_plus): add API to get the cluster ID of the current node
 - <csr-id-997d90a76db9a4e05dbac35073a09548750ce342/> Added poll_futures and poll_futures_async operators.
 - <csr-id-c3f5a37ff746401a2383a900f9004e33072d5b1a/> add prototype of tagging algebraic properties
 - <csr-id-29a263fb564c5ce4bc495ea4e9d20b8b2621b645/> add support for collecting counts and running perf

### Bug Fixes

 - <csr-id-0cafbdb74a665412a83aa900b4eb10c00e2498dd/> handle send_bincode with local structs
   fix(hydroflow_plus): handle send_bincode with local structs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 44 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 6 unique issues were worked on: [#1143](https://github.com/hydro-project/hydroflow/issues/1143), [#1151](https://github.com/hydro-project/hydroflow/issues/1151), [#1156](https://github.com/hydro-project/hydroflow/issues/1156), [#1157](https://github.com/hydro-project/hydroflow/issues/1157), [#1194](https://github.com/hydro-project/hydroflow/issues/1194), [#1238](https://github.com/hydro-project/hydroflow/issues/1238)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#1143](https://github.com/hydro-project/hydroflow/issues/1143)**
    - Added poll_futures and poll_futures_async operators. ([`997d90a`](https://github.com/hydro-project/hydroflow/commit/997d90a76db9a4e05dbac35073a09548750ce342))
 * **[#1151](https://github.com/hydro-project/hydroflow/issues/1151)**
    - Handle send_bincode with local structs ([`0cafbdb`](https://github.com/hydro-project/hydroflow/commit/0cafbdb74a665412a83aa900b4eb10c00e2498dd))
 * **[#1156](https://github.com/hydro-project/hydroflow/issues/1156)**
    - Add prototype of tagging algebraic properties ([`c3f5a37`](https://github.com/hydro-project/hydroflow/commit/c3f5a37ff746401a2383a900f9004e33072d5b1a))
 * **[#1157](https://github.com/hydro-project/hydroflow/issues/1157)**
    - Add support for collecting counts and running perf ([`29a263f`](https://github.com/hydro-project/hydroflow/commit/29a263fb564c5ce4bc495ea4e9d20b8b2621b645))
 * **[#1194](https://github.com/hydro-project/hydroflow/issues/1194)**
    - Add API to get the cluster ID of the current node ([`6e57172`](https://github.com/hydro-project/hydroflow/commit/6e571726ff40818fbe9bbe9923511877c20fb243))
 * **[#1238](https://github.com/hydro-project/hydroflow/issues/1238)**
    - Use workaround for `cargo smart-release` not properly ordering `dev-`/`build-dependencies` ([`c9dfddc`](https://github.com/hydro-project/hydroflow/commit/c9dfddc680e0ce5415539d7b77bc5beb97ab59d9))
 * **Uncategorized**
    - Release hydroflow_lang v0.7.0, hydroflow_datalog_core v0.7.0, hydroflow_datalog v0.7.0, hydroflow_macro v0.7.0, lattices v0.5.5, multiplatform_test v0.1.0, pusherator v0.0.6, hydroflow v0.7.0, stageleft_macro v0.2.0, stageleft v0.3.0, stageleft_tool v0.2.0, hydroflow_plus v0.7.0, hydro_deploy v0.7.0, hydro_cli v0.7.0, hydroflow_plus_cli_integration v0.7.0, safety bump 8 crates ([`2852147`](https://github.com/hydro-project/hydroflow/commit/285214740627685e911781793e05d234ab2ad2bd))
</details>

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

 - 9 commits contributed to the release.
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
    - Release hydroflow_plus v0.6.1, hydro_deploy v0.6.1, hydro_cli v0.6.1, hydroflow_plus_cli_integration v0.6.1 ([`c385c13`](https://github.com/hydro-project/hydroflow/commit/c385c132c9733d1bace82156aa14216b8e7fef9f))
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

 - 4 commits contributed to the release.
 - 32 days passed between releases.
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

 - 25 commits contributed to the release.
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

