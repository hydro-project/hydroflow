## Unreleased

Unchanged from previous release.

## v0.2.0 (2023-05-31)

### Chore

 - <csr-id-fd896fbe925fbd8ef1d16be7206ac20ba585081a/> manually bump versions for v0.2.0 release

### New Features

 - <csr-id-8b2c9f09b1f423ac6d562c29d4ea587578f1c98a/> Add more detailed Hydro Deploy docs and rename `ConnectedBidi` => `ConnectedDirect`

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#723](https://github.com/hydro-project/hydroflow/issues/723)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#723](https://github.com/hydro-project/hydroflow/issues/723)**
    - Add more detailed Hydro Deploy docs and rename `ConnectedBidi` => `ConnectedDirect` ([`8b2c9f0`](https://github.com/hydro-project/hydroflow/commit/8b2c9f09b1f423ac6d562c29d4ea587578f1c98a))
 * **Uncategorized**
    - Manually bump versions for v0.2.0 release ([`fd896fb`](https://github.com/hydro-project/hydroflow/commit/fd896fbe925fbd8ef1d16be7206ac20ba585081a))
</details>

## v0.1.0 (2023-05-30)

<csr-id-665ad20d996c7873117ff7cccfac22366117d71a/>
<csr-id-382a83c2304eda476d4ff8195a96efebd8dbbcb7/>
<csr-id-52ee8f8e443f0a8b5caf92d2c5f028c00302a79b/>
<csr-id-51a3a9e5f19594a21702d66730d5d1668009b550/>
<csr-id-2bd8517768ff3924b7af274d8d97f126143c4a2a/>
<csr-id-cd0a86d9271d0e3daab59c46f079925f863424e1/>
<csr-id-20a1b2c0cd04a8b495a02ce345db3d48a99ea0e9/>
<csr-id-1eda91a2ef8794711ef037240f15284e8085d863/>

### Chore

 - <csr-id-665ad20d996c7873117ff7cccfac22366117d71a/> Cargo.toml documentation and description
 - <csr-id-382a83c2304eda476d4ff8195a96efebd8dbbcb7/> set hydroflow_cli_integration version
 - <csr-id-52ee8f8e443f0a8b5caf92d2c5f028c00302a79b/> bump versions to 0.1.0 for release
   For release on crates.io for v0.1

### Other

 - <csr-id-61a1a0509b465ed57003bd0cdfedee8b847a48c8/> initialize hydro_cli/CHANGELOG.md

### Chore

 - <csr-id-e3ddfb8b47effd03a9bb346811ea360a14ab17b3/> Cargo.toml documentation and description

### New Features

 - <csr-id-4536ac6bbcd14a621b5a039d7fe213bff72a8db1/> finish up WebSocket chat example and avoid deadlocks in network setup

### Bug Fixes

 - <csr-id-1c06b3b9ed253aea8c1d2cfd87a1ea77ce550f70/> don't create file copies on when deploying to localhost
   This causes issues on M1, likely due to some signing issue?
 - <csr-id-268f83794d77fbb95f7d3ce7e2439371ccbf8e0c/> mismatched package name in CLI build and attempt to really fix crashes
 - <csr-id-508b00e064427211d6ec6c884af1eb4a602d19b9/> Prepare action logic to publish CLI to PyPi and eliminate GIL acquires
   Hopefully this will work on the first try? Not really a good way to test it. It seems that acquiring the GIL in async/await code is asking for trouble, so this also eliminates those.

### Other

 - <csr-id-51a3a9e5f19594a21702d66730d5d1668009b550/> initialize hydro_cli/CHANGELOG.md
 - <csr-id-2bd8517768ff3924b7af274d8d97f126143c4a2a/> publish hydro_cli
   Will bump versions for python deploy.
   Update build-cli.yml to publish on hydro_cli release

### Style

 - <csr-id-cd0a86d9271d0e3daab59c46f079925f863424e1/> Warn lint `unused_qualifications`
 - <csr-id-20a1b2c0cd04a8b495a02ce345db3d48a99ea0e9/> rustfmt group imports
 - <csr-id-1eda91a2ef8794711ef037240f15284e8085d863/> rustfmt prescribe flat-module `use` format

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 71 commits contributed to the release over the course of 101 calendar days.
 - 12 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 63 unique issues were worked on: [#390](https://github.com/hydro-project/hydroflow/issues/390), [#397](https://github.com/hydro-project/hydroflow/issues/397), [#410](https://github.com/hydro-project/hydroflow/issues/410), [#411](https://github.com/hydro-project/hydroflow/issues/411), [#417](https://github.com/hydro-project/hydroflow/issues/417), [#420](https://github.com/hydro-project/hydroflow/issues/420), [#433](https://github.com/hydro-project/hydroflow/issues/433), [#436](https://github.com/hydro-project/hydroflow/issues/436), [#437](https://github.com/hydro-project/hydroflow/issues/437), [#445](https://github.com/hydro-project/hydroflow/issues/445), [#446](https://github.com/hydro-project/hydroflow/issues/446), [#451](https://github.com/hydro-project/hydroflow/issues/451), [#452](https://github.com/hydro-project/hydroflow/issues/452), [#460](https://github.com/hydro-project/hydroflow/issues/460), [#461](https://github.com/hydro-project/hydroflow/issues/461), [#462](https://github.com/hydro-project/hydroflow/issues/462), [#466](https://github.com/hydro-project/hydroflow/issues/466), [#473](https://github.com/hydro-project/hydroflow/issues/473), [#474](https://github.com/hydro-project/hydroflow/issues/474), [#477](https://github.com/hydro-project/hydroflow/issues/477), [#479](https://github.com/hydro-project/hydroflow/issues/479), [#481](https://github.com/hydro-project/hydroflow/issues/481), [#484](https://github.com/hydro-project/hydroflow/issues/484), [#492](https://github.com/hydro-project/hydroflow/issues/492), [#494](https://github.com/hydro-project/hydroflow/issues/494), [#498](https://github.com/hydro-project/hydroflow/issues/498), [#503](https://github.com/hydro-project/hydroflow/issues/503), [#513](https://github.com/hydro-project/hydroflow/issues/513), [#515](https://github.com/hydro-project/hydroflow/issues/515), [#525](https://github.com/hydro-project/hydroflow/issues/525), [#527](https://github.com/hydro-project/hydroflow/issues/527), [#531](https://github.com/hydro-project/hydroflow/issues/531), [#532](https://github.com/hydro-project/hydroflow/issues/532), [#533](https://github.com/hydro-project/hydroflow/issues/533), [#534](https://github.com/hydro-project/hydroflow/issues/534), [#535](https://github.com/hydro-project/hydroflow/issues/535), [#537](https://github.com/hydro-project/hydroflow/issues/537), [#542](https://github.com/hydro-project/hydroflow/issues/542), [#557](https://github.com/hydro-project/hydroflow/issues/557), [#560](https://github.com/hydro-project/hydroflow/issues/560), [#576](https://github.com/hydro-project/hydroflow/issues/576), [#582](https://github.com/hydro-project/hydroflow/issues/582), [#586](https://github.com/hydro-project/hydroflow/issues/586), [#596](https://github.com/hydro-project/hydroflow/issues/596), [#600](https://github.com/hydro-project/hydroflow/issues/600), [#612](https://github.com/hydro-project/hydroflow/issues/612), [#617](https://github.com/hydro-project/hydroflow/issues/617), [#620](https://github.com/hydro-project/hydroflow/issues/620), [#626](https://github.com/hydro-project/hydroflow/issues/626), [#627](https://github.com/hydro-project/hydroflow/issues/627), [#628](https://github.com/hydro-project/hydroflow/issues/628), [#631](https://github.com/hydro-project/hydroflow/issues/631), [#647](https://github.com/hydro-project/hydroflow/issues/647), [#656](https://github.com/hydro-project/hydroflow/issues/656), [#660](https://github.com/hydro-project/hydroflow/issues/660), [#679](https://github.com/hydro-project/hydroflow/issues/679), [#681](https://github.com/hydro-project/hydroflow/issues/681), [#684](https://github.com/hydro-project/hydroflow/issues/684), [#694](https://github.com/hydro-project/hydroflow/issues/694), [#699](https://github.com/hydro-project/hydroflow/issues/699), [#708](https://github.com/hydro-project/hydroflow/issues/708), [#712](https://github.com/hydro-project/hydroflow/issues/712), [#715](https://github.com/hydro-project/hydroflow/issues/715)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#390](https://github.com/hydro-project/hydroflow/issues/390)**
    - Introduce initial Hydro CLI architecture ([`52aa6e0`](https://github.com/hydro-project/hydroflow/commit/52aa6e0e5d5417bc185cf8f1f961c5494b5b5129))
 * **[#397](https://github.com/hydro-project/hydroflow/issues/397)**
    - Add basic support for connecting services with Unix/TCP sockets ([`dbdad61`](https://github.com/hydro-project/hydroflow/commit/dbdad61d43412a44449495b4204e37d5d128c12c))
 * **[#410](https://github.com/hydro-project/hydroflow/issues/410)**
    - Fixup! Initial support for GCP deployments ([`8695b5d`](https://github.com/hydro-project/hydroflow/commit/8695b5de22a03a4f5f06352c216183e9e10c5199))
    - Initial support for GCP deployments ([`f10a54f`](https://github.com/hydro-project/hydroflow/commit/f10a54ff1eee3e71e1c488d5948762171cca3f5b))
 * **[#411](https://github.com/hydro-project/hydroflow/issues/411)**
    - Fix non-unix (windows) build referencing unix sockets ([`5dac7e4`](https://github.com/hydro-project/hydroflow/commit/5dac7e4fcd2022c4fb9538d55f9a793139b98c6f))
 * **[#417](https://github.com/hydro-project/hydroflow/issues/417)**
    - Add API for defining custom services in deployment ([`2fb8871`](https://github.com/hydro-project/hydroflow/commit/2fb88710603948479580aea58f894ab3929280c8))
 * **[#420](https://github.com/hydro-project/hydroflow/issues/420)**
    - Update clap ([`4be709f`](https://github.com/hydro-project/hydroflow/commit/4be709f03acd854d27e551638e31af7ce5b26c0b))
 * **[#433](https://github.com/hydro-project/hydroflow/issues/433)**
    - Package CLI as a Python wheel to simplify distribution ([`b952257`](https://github.com/hydro-project/hydroflow/commit/b95225770b8ab43a414d5f3c41387d6941f45f26))
 * **[#436](https://github.com/hydro-project/hydroflow/issues/436)**
    - Support passing through extra arguments to deployment scripts ([`f40009c`](https://github.com/hydro-project/hydroflow/commit/f40009c2eab949c533ae5fb69fd9433a6b75c686))
 * **[#437](https://github.com/hydro-project/hydroflow/issues/437)**
    - Extract common logic for establishing CLI-configured connections ([`44cce72`](https://github.com/hydro-project/hydroflow/commit/44cce727b4363d1b6e7f73d72e0a3bec7b6ace53))
 * **[#445](https://github.com/hydro-project/hydroflow/issues/445)**
    - Add `demux` operator to Hydro CLI to map node IDs to connections ([`886d00f`](https://github.com/hydro-project/hydroflow/commit/886d00f6694ba926c9e1ff184acb31a5d60cee23))
 * **[#446](https://github.com/hydro-project/hydroflow/issues/446)**
    - Support running example deployment script without CLI ([`4b3233a`](https://github.com/hydro-project/hydroflow/commit/4b3233a3b791cfbde4a7721b6796436ef41233d0))
 * **[#451](https://github.com/hydro-project/hydroflow/issues/451)**
    - Enable local deployments on non-Linux hosts ([`74c8d3d`](https://github.com/hydro-project/hydroflow/commit/74c8d3d1f18c564808c930147e4d31463b80c735))
 * **[#452](https://github.com/hydro-project/hydroflow/issues/452)**
    - Build CLI wheels in CI and minimize CLI dependencies ([`3e33d0c`](https://github.com/hydro-project/hydroflow/commit/3e33d0cf6b068f0567e55462732598f8a4e2da6a))
 * **[#460](https://github.com/hydro-project/hydroflow/issues/460)**
    - Allow specifying args to launch `HydroflowCrate` with ([`3575fd3`](https://github.com/hydro-project/hydroflow/commit/3575fd3dd2b4aa98361cc4f723d590eff4794f5f))
 * **[#461](https://github.com/hydro-project/hydroflow/issues/461)**
    - Support networking topologies that mix local and cloud through SSH tunneling ([`0ec6d88`](https://github.com/hydro-project/hydroflow/commit/0ec6d889469331a212c04f9568136f770f0c973d))
 * **[#462](https://github.com/hydro-project/hydroflow/issues/462)**
    - Directly expose Rust bindings as Python APIs ([`b94413a`](https://github.com/hydro-project/hydroflow/commit/b94413a380007f5f4f710d2c849c412602a8f8c2))
 * **[#466](https://github.com/hydro-project/hydroflow/issues/466)**
    - Add APIs for sending data to a Hydroflow service from Python ([`c2203a1`](https://github.com/hydro-project/hydroflow/commit/c2203a15f0144308365af227f3ca044ae6a7954b))
 * **[#473](https://github.com/hydro-project/hydroflow/issues/473)**
    - Fixup! Add initial VPC configuration API and improve interrupt handling ([`7f21514`](https://github.com/hydro-project/hydroflow/commit/7f21514d2be2d9dd5e877ad5be534c81579367ce))
    - Add initial VPC configuration API and improve interrupt handling ([`c729fc0`](https://github.com/hydro-project/hydroflow/commit/c729fc0fe01ba75b0ba622e9bc68d891c5353e03))
 * **[#474](https://github.com/hydro-project/hydroflow/issues/474)**
    - Extract common SSH host logic into a separate module ([`5cc884e`](https://github.com/hydro-project/hydroflow/commit/5cc884e4063729216990c1793fb412edd60b0c63))
 * **[#477](https://github.com/hydro-project/hydroflow/issues/477)**
    - Properly handle interrupts and fix non-flushing demux ([`00ea017`](https://github.com/hydro-project/hydroflow/commit/00ea017e40b796e7561979efa0921658dfe072fd))
 * **[#479](https://github.com/hydro-project/hydroflow/issues/479)**
    - Allow custom ports to be used as sinks ([`8da15b7`](https://github.com/hydro-project/hydroflow/commit/8da15b7cbd8bdbf960d3ed58b69f98538ccacd2c))
 * **[#481](https://github.com/hydro-project/hydroflow/issues/481)**
    - Display Anyhow traces when using directly using CLI APIs ([`0f19fa4`](https://github.com/hydro-project/hydroflow/commit/0f19fa4ab1c821649e7f400b1842515e83fb4585))
 * **[#484](https://github.com/hydro-project/hydroflow/issues/484)**
    - Add merge API to CLI to have multiple sources for one sink ([`e09b567`](https://github.com/hydro-project/hydroflow/commit/e09b5670795292f66a004f41314c3c4aa7a24eeb))
 * **[#492](https://github.com/hydro-project/hydroflow/issues/492)**
    - Add API to gracefully shutdown services ([`eda517a`](https://github.com/hydro-project/hydroflow/commit/eda517a3435093830135a9f0384bfae1de5c853e))
 * **[#494](https://github.com/hydro-project/hydroflow/issues/494)**
    - Fixup! Add initial VPC configuration API and improve interrupt handling ([`7f21514`](https://github.com/hydro-project/hydroflow/commit/7f21514d2be2d9dd5e877ad5be534c81579367ce))
 * **[#498](https://github.com/hydro-project/hydroflow/issues/498)**
    - Add API to get CLI connection config as JSON ([`323e0f0`](https://github.com/hydro-project/hydroflow/commit/323e0f0afd73b66f321b2e88498627e76a186a4e))
 * **[#503](https://github.com/hydro-project/hydroflow/issues/503)**
    - Allow redeployment in CLI with updated services and hosts ([`967df05`](https://github.com/hydro-project/hydroflow/commit/967df05e7ec97201cdc602316bd99c03b541b5d4))
 * **[#513](https://github.com/hydro-project/hydroflow/issues/513)**
    - Add `hydro.null` API to connect no-op sources and sinks ([`9b2a4a6`](https://github.com/hydro-project/hydroflow/commit/9b2a4a690798d2a976221901fa25a908b7600f52))
 * **[#515](https://github.com/hydro-project/hydroflow/issues/515)**
    - Initial TopoloTree actor implementation for binary tree ([`e9fcc24`](https://github.com/hydro-project/hydroflow/commit/e9fcc24761b676f7f0796767d6f910eaad1ee9b4))
 * **[#525](https://github.com/hydro-project/hydroflow/issues/525)**
    - Add `existing` parameter to `GCPNetwork` to use existing VPCs ([`33249e4`](https://github.com/hydro-project/hydroflow/commit/33249e4517e8ca3735a0949957ef9b43c55ff947))
 * **[#527](https://github.com/hydro-project/hydroflow/issues/527)**
    - Actually return a `GCPComputeEngineHost` when creating one ([`0eef370`](https://github.com/hydro-project/hydroflow/commit/0eef370485b9904185f846a553c94accc0a91118))
 * **[#531](https://github.com/hydro-project/hydroflow/issues/531)**
    - Provision hosts even if they are not being used by a service ([`abdf61d`](https://github.com/hydro-project/hydroflow/commit/abdf61d8982e83262e8a452214936c0f9d90e456))
 * **[#532](https://github.com/hydro-project/hydroflow/issues/532)**
    - Generalize null source support into `SourcePath` abstraction ([`835ba3b`](https://github.com/hydro-project/hydroflow/commit/835ba3bdaf553dad8261b89087e0ab45f017325b))
 * **[#533](https://github.com/hydro-project/hydroflow/issues/533)**
    - Add `hydro.mux` operator and initial API tests ([`c25272b`](https://github.com/hydro-project/hydroflow/commit/c25272b90f8cc5ec7614caa29f0be889d2220510))
 * **[#534](https://github.com/hydro-project/hydroflow/issues/534)**
    - Allow specifying the user to sign in as on a GCP machine ([`ad1609d`](https://github.com/hydro-project/hydroflow/commit/ad1609d0c9a700ada5678a8df05694ff9606c54c))
 * **[#535](https://github.com/hydro-project/hydroflow/issues/535)**
    - Ignore GCP port requests for ports that have already been allocated ([`c948ab8`](https://github.com/hydro-project/hydroflow/commit/c948ab8aaad2204b277eb80752529283351536d6))
 * **[#537](https://github.com/hydro-project/hydroflow/issues/537)**
    - Use the correct user account ([`86135f4`](https://github.com/hydro-project/hydroflow/commit/86135f4efa3375e3ce527f40f05474d7011c1487))
 * **[#542](https://github.com/hydro-project/hydroflow/issues/542)**
    - Avoid deadlock in port loading when a service connects to itself ([`559f115`](https://github.com/hydro-project/hydroflow/commit/559f1154cb4b84b7b4cd3963c2d212e2bc05d524))
 * **[#557](https://github.com/hydro-project/hydroflow/issues/557)**
    - Have Python drive CLI cancellations to support interrupting loops ([`f3e57c9`](https://github.com/hydro-project/hydroflow/commit/f3e57c9ff7df36e24419aab9d6a957a11b5ab7cb))
 * **[#560](https://github.com/hydro-project/hydroflow/issues/560)**
    - Refactor `hydro.mux` to `source.tagged(id)` and support connections where the tagged source is the server ([`3f0ecc9`](https://github.com/hydro-project/hydroflow/commit/3f0ecc92abed7a0c95c04255adcc6d39c0767703))
 * **[#576](https://github.com/hydro-project/hydroflow/issues/576)**
    - Add classic counter CRDT benchmark to compare against ([`2f3bf04`](https://github.com/hydro-project/hydroflow/commit/2f3bf04ab33768b04d44f3f58907f958d4cd8dc8))
 * **[#582](https://github.com/hydro-project/hydroflow/issues/582)**
    - Add a global cache for Cargo builds initiated by the CLI ([`83c1df7`](https://github.com/hydro-project/hydroflow/commit/83c1df792d0dbb1d89fd9383ea284ca3ff167778))
 * **[#586](https://github.com/hydro-project/hydroflow/issues/586)**
    - Bump pinned nightly and fix build failures on latest nightly ([`84a831e`](https://github.com/hydro-project/hydroflow/commit/84a831efca6eddac20bac140c9c67bf4ab2d5cf8))
 * **[#596](https://github.com/hydro-project/hydroflow/issues/596)**
    - Improve CLI interrupt handling when subtasks are spawned ([`93fb340`](https://github.com/hydro-project/hydroflow/commit/93fb34040b12a74d246729e37bb6a3bd9924b807))
 * **[#600](https://github.com/hydro-project/hydroflow/issues/600)**
    - Display rich progress for deployment tasks in console ([`467e2fb`](https://github.com/hydro-project/hydroflow/commit/467e2fb719fb101e1c706814c07ebfc43f324eec))
 * **[#612](https://github.com/hydro-project/hydroflow/issues/612)**
    - Fix lints on windows ([`2f8d3e2`](https://github.com/hydro-project/hydroflow/commit/2f8d3e212f4d60d908e733d1b1f1348501596df8))
 * **[#617](https://github.com/hydro-project/hydroflow/issues/617)**
    - Update `Cargo.toml`s for publishing ([`a78ff9a`](https://github.com/hydro-project/hydroflow/commit/a78ff9aace6771787c2b72aad83be6ad8d49a828))
 * **[#620](https://github.com/hydro-project/hydroflow/issues/620)**
    - Replace using `cargo` as a library to shell out with `cargo-metadata` instead ([`5f2e8f3`](https://github.com/hydro-project/hydroflow/commit/5f2e8f3abffec38ba99afeb60969788e16e2f4ff))
 * **[#626](https://github.com/hydro-project/hydroflow/issues/626)**
    - Print logs from services with a prefix identifying the service ([`79dda6a`](https://github.com/hydro-project/hydroflow/commit/79dda6ab463f51c0c3e1c932cba0f45ef95a4f78))
 * **[#627](https://github.com/hydro-project/hydroflow/issues/627)**
    - Display cargo build status formatted next to a progress bar ([`5cbe43a`](https://github.com/hydro-project/hydroflow/commit/5cbe43a44e9e118eaf790886bef8409cd6b211ee))
 * **[#628](https://github.com/hydro-project/hydroflow/issues/628)**
    - Handle Terraform printing a log about reading existing resources ([`6bf7b71`](https://github.com/hydro-project/hydroflow/commit/6bf7b7182cfe137cfda3164898b461e5e5602ae7))
 * **[#631](https://github.com/hydro-project/hydroflow/issues/631)**
    - Avoid clobbering Rust errors with the progress bar ([`6f3cf4b`](https://github.com/hydro-project/hydroflow/commit/6f3cf4bcff4de658e9a4d80180748aefe393a0bb))
 * **[#647](https://github.com/hydro-project/hydroflow/issues/647)**
    - Fix Hydro CLI builds failing due to breaking Maturin change ([`ffee23f`](https://github.com/hydro-project/hydroflow/commit/ffee23f33a77e54a7ab6af3a678f95ed35f0b4eb))
 * **[#656](https://github.com/hydro-project/hydroflow/issues/656)**
    - Add WebSocket with CLI example and simplify init API ([`1015980`](https://github.com/hydro-project/hydroflow/commit/1015980ed995634ff8735e4daf33796e73bab563))
 * **[#660](https://github.com/hydro-project/hydroflow/issues/660)**
    - Warn lint `unused_qualifications` ([`cd0a86d`](https://github.com/hydro-project/hydroflow/commit/cd0a86d9271d0e3daab59c46f079925f863424e1))
    - Rustfmt group imports ([`20a1b2c`](https://github.com/hydro-project/hydroflow/commit/20a1b2c0cd04a8b495a02ce345db3d48a99ea0e9))
    - Rustfmt prescribe flat-module `use` format ([`1eda91a`](https://github.com/hydro-project/hydroflow/commit/1eda91a2ef8794711ef037240f15284e8085d863))
 * **[#679](https://github.com/hydro-project/hydroflow/issues/679)**
    - Only load converters helper module once in the CLI ([`860d74f`](https://github.com/hydro-project/hydroflow/commit/860d74fcab8525397eb630b14ca7c6619fcef1f4))
 * **[#681](https://github.com/hydro-project/hydroflow/issues/681)**
    - Migrate playground to new docs site ([`4d16bd2`](https://github.com/hydro-project/hydroflow/commit/4d16bd218104e1abcc1e1210942b0ec5b63301d0))
 * **[#684](https://github.com/hydro-project/hydroflow/issues/684)**
    - Bump versions to 0.1.0 for release ([`52ee8f8`](https://github.com/hydro-project/hydroflow/commit/52ee8f8e443f0a8b5caf92d2c5f028c00302a79b))
 * **[#694](https://github.com/hydro-project/hydroflow/issues/694)**
    - Prepare action logic to publish CLI to PyPi and eliminate GIL acquires ([`508b00e`](https://github.com/hydro-project/hydroflow/commit/508b00e064427211d6ec6c884af1eb4a602d19b9))
 * **[#699](https://github.com/hydro-project/hydroflow/issues/699)**
    - Mismatched package name in CLI build and attempt to really fix crashes ([`268f837`](https://github.com/hydro-project/hydroflow/commit/268f83794d77fbb95f7d3ce7e2439371ccbf8e0c))
 * **[#708](https://github.com/hydro-project/hydroflow/issues/708)**
    - Finish up WebSocket chat example and avoid deadlocks in network setup ([`4536ac6`](https://github.com/hydro-project/hydroflow/commit/4536ac6bbcd14a621b5a039d7fe213bff72a8db1))
 * **[#712](https://github.com/hydro-project/hydroflow/issues/712)**
    - Publish hydro_cli ([`2bd8517`](https://github.com/hydro-project/hydroflow/commit/2bd8517768ff3924b7af274d8d97f126143c4a2a))
 * **[#715](https://github.com/hydro-project/hydroflow/issues/715)**
    - Don't create file copies on when deploying to localhost ([`1c06b3b`](https://github.com/hydro-project/hydroflow/commit/1c06b3b9ed253aea8c1d2cfd87a1ea77ce550f70))
 * **Uncategorized**
    - Release hydro_cli v0.1.0 ([`5d48544`](https://github.com/hydro-project/hydroflow/commit/5d485442691f878ae6835f631ae13ff856fd941c))
    - Cargo.toml documentation and description ([`e3ddfb8`](https://github.com/hydro-project/hydroflow/commit/e3ddfb8b47effd03a9bb346811ea360a14ab17b3))
    - Initialize hydro_cli/CHANGELOG.md ([`61a1a05`](https://github.com/hydro-project/hydroflow/commit/61a1a0509b465ed57003bd0cdfedee8b847a48c8))
    - Set hydroflow_cli_integration version ([`382a83c`](https://github.com/hydro-project/hydroflow/commit/382a83c2304eda476d4ff8195a96efebd8dbbcb7))
    - Update pinned nightly rust version 2023-04-18 ([`6ced3c1`](https://github.com/hydro-project/hydroflow/commit/6ced3c177969dec3d3e3cf5938ab3973c1d1239b))
</details>

