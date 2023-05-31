# Hydroflow

[Hydroflow](https://github.com/hydro-project/hydroflow) is a compiler for low-latency
dataflow programs, written in Rust. Hydroflow is the runtime library for the
[Hydro language stack](https://hydro.run/docs/hydroflow/ecosystem), which is under development
as a complete compiler stack for distributed programming languages.

Hydroflow is designed with two goals in mind:
- Expert developers can program Hydroflow directly to build components in a distributed system.
- Higher levels of the Hydro stack will offer friendlier languages with more abstractions, and treat Hydroflow as a compiler target.

Hydroflow provides a DSL—the *surface syntax*—embedded in Rust, which compiles to high-efficiency machine code.
As the lowest level of the Hydro stack, Hydroflow requires some knowledge of Rust to use.

Check out the [Hydroflow Playground](https://hydro.run/playground) to see Hydroflow's surface syntax in action!  
Or read the [Hydroflow Book docs](https://hydro.run/docs/hydroflow/#this-book) to get started.
