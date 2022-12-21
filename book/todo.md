# TODO

## Concepts
- Big Picture Architecture: A low-level framework. Threads (actors) communicating async via channels. No global constructs in Hydroflow. No SPMD assumptions.
- Hydroflow and Rust: how do they go together?
    - State, control, scoping
- Time
    - The Hydroflow event loop and Ticks. local clock
    - Assembling a global vector clock
- Fix points and Strata
- State over time
    - lifetimes
    - explicit deletion
- Coordination tricks?
    - End-of-stream to Distributed EOS?

## Docs
- `hydroflow` struct and its methods
- Review the ops docs

## What's covered in examples
- Concepts covered
    - cargo generate for templating
    - Hydroflow program specs embedded in Rust
    - Tokio Channels and how to use them in Hydroflow
        - Network sources and sinks (source_stream)
        - Built-in serde (source_stream_serde, dest_sink_serde)
    - Hydroflow syntax: operators, ->, variables, indexing multi-input/output operators
    - running Hydroflow via `run_available` and `run_async`
    - Recursion via cyclic dataflow
    - Fixpoints and Strata
    - Template structure: clap, message types
    - source_stdin
    - Messages and `demux`
    - broadcast pattern
    - gated buffer pattern
    - bootstrapping pipelines

- Operators covered
    - cross_join
    - demux
    - dest_sink_serde
    - difference
    - filter
    - filter_map
    - flatten
    - flat_map
    - for_each
    - join
    - map
    - merge
    - source_iter
    - source_stdin
    - source_stream
    - source_stream_serde
    - tee
    - unique

- Operators not discussed
    - dest_sink
    - identity
    - repeat_iter
    - unzip
    - *fold* -- add to chapter on [state](state.md)
    - *reduce* -- add to chapter on [state](state.md)
    - *group_by* -- add to chapter on [state](state.md)
    - *sort* -- add to chapter on [state](state.md)
    - *sort_by* -- add to chapter on [state](state.md)
    - *next_stratum* -- add to chapter on [time](time.md)
    - *next_epoch* -- add to chapter on [time](time.md)
    - *inspect* -- add to chapter on [debugging](debugging.md)
    - *null* -- add to chapter on [debugging](debugging.md)

## How-Tos
- A partitioned Service
- A replicated Service
- Interfacing with external data
- Interfacing with external services

## Odds and ends taken out of other chapters
- **Document the methods on the `hydroflow` struct** -- especially the run methods.
    -  The [`run_epoch()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_epoch), [`run_stratum()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_stratum), [`run()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run), and [`run_async()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_async) methods provide other ways to control the graph execution.
    - Also `run_available()` `next_stratum()` and `recv_events` are important
- More generally, document the **Core API**

- **Strata and Epochs** -- explain the concept of strata and epochs, and how they relate to the `run_epoch()` and `run_stratum()` methods.

- **Make sure `src/examples/echoserver` is the same as the template project** -- or better, find a way to do that via github actions or a github submodule

## More Examples
- Illustrate `'static` and `'epoch` lifetimes (KVS)
- Illustrate partitioning and replication (KVS)
- Illustrate the `next_stratum` operator for atomicity (eg Bloom's upsert `<+-` operator)
- Illustrate ordered streams (need `zip` operator ... what's the example?)
- Actor model implementation (Borrow an Akka or Ray Actors example?)
- Futures emulation? (Borrow a Ray example)
- Illustrate external storage source and sink (e.g. for WAL of KVS)