# Hydroflow

[Hydroflow](https://github.com/hydro-project/hydroflow) is a small language and compiler for low-latency
dataflow programs, written in Rust. Hydroflow serves as the runtime library for the
[Hydro language stack](https://hydro.run/docs/hydroflow/ecosystem), which is under development
as a complete compiler stack for distributed programming languages.

Hydroflow is designed with two use cases in mind:
  - Expert developers can program directly in the Hydroflow language to build individual components that can interoperate in a distributed program or service.
  - Higher levels of the Hydro stack will offer higher-level abstractions and DSLs, and treat Hydroflow as a compiler target. 

Hydroflow is targeted at supporting the following unique features:
  1. A type system that helps developers reason about progress and consistency guarantees in a distributed program. This includes an emphasis on [lattice types](https://hydro.run/docs/hydroflow/lattices_crate/) that can allow for consistent outcomes in the face of network messages that may be interleaved, reordered, batched, and resent.
  2. A [dataflow programming model](https://hydro.run/docs/hydroflow/syntax/surface_flows), capturing the message- and data-driven nature of many distributed services.
  3. Extremely low-latency handling of asynchronously-arriving data/messages, via aggressive exploitation of Rust's [monomorphization](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html) techniques.
  4. Dataflow optimizations, both to optimize single-node Hydroflow flows, and to enable distributed optimizations across multiple flows.
  
Hydroflow's language—the Hydroflow *surface syntax*—is embedded in Rust, which compiles Hydroflow code to high-efficiency machine code.
As the lowest level of the Hydro stack, Hydroflow requires some knowledge of Rust to use.

The most recent release of the [Hydroflow Book docs](https://hydro.run/docs/hydroflow/#this-book) are online, providing documentation and numerous annotated examples.

You can also check out the [Hydroflow Playground](https://hydro.run/playground) to see Hydroflow's surface syntax in action!  
