# TODO

## Concepts
- P1 (Mingwei) Hydroflow and Rust: how do they go together?
    - State, control, scoping
- P1 (Joe) State over time
    - lifetimes
    - explicit deletion
- P2 Coordination tricks?
    - End-of-stream to Distributed EOS?

## Docs
- P1 (Mingwei) `hydroflow` struct and its methods
- P2 Review the ops docs

- Operators not discussed (Joe)
    - P1 *fold* -- add to chapter on [state](state.md)
    - P1 *reduce* -- add to chapter on [state](state.md)
    - P1 *group_by* -- add to chapter on [state](state.md)
    - P3 *sort* -- add to chapter on [state](state.md)
    - P3 *sort_by* -- add to chapter on [state](state.md)
    - P2 *next_stratum* -- add to chapter on [time](time.md)
    - P2 *next_tick* -- add to chapter on [time](time.md)
    - P2 *inspect* -- add to chapter on [debugging](debugging.md)
    - P2 *null* -- add to chapter on [debugging](debugging.md)
    - dest_sink
    - identity
    - repeat_iter
    - unzip
    

## How-Tos / More examples
- P1 (Joe, State over Time) Illustrate `'static` and `'tick` lifetimes (KVS)
- P2 A replicated Service -- e.g. chat server or KVS
- P2 A partitioned Service -- e.g. chat server or KVS
- P3 Interfacing with external data
- P3 Interfacing with external services
- P2 Vector clocks
- P3 Illustrate the `next_stratum` operator for atomicity (eg Bloom's upsert `<+-` operator)
- P3 Illustrate ordered streams (need `zip` operator ... what's the example?)
- P3 Actor model implementation (Borrow an Akka or Ray Actors example?)
- P3 Futures emulation? (Borrow a Ray example)

## Odds and ends taken out of other chapters
- P1 (Mingwei, see above) **Document the methods on the `hydroflow` struct** -- especially the run methods.
    -  The [`run_tick()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_tick), [`run_stratum()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_stratum), [`run()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run), and [`run_async()`](https://hydro-project.github.io/hydroflow/doc/hydroflow/scheduled/graph/struct.Hydroflow.html#method.run_async) methods provide other ways to control the graph execution.
    - Also `run_available()` `next_stratum()` and `recv_events` are important
- More generally, document the **Core API**

- P2 **Make sure `src/examples/echoserver` is the same as the template project** -- or better, find a way to do that via github actions or a github submodule

## What's covered in current examples
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

