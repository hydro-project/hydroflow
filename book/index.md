# Introduction

[Hydroflow](https://github.com/hydro-project/hydroflow) is a compiler for low-latency 
dataflow programs, written in Rust. Hydroflow is the runtime library for the 
[Hydro Project](https://hydro-project.github.io/), which is under development
as a complete compiler stack for distributed programming languages. 

Hydroflow provides a DSL—a *surface syntax*—embedded in Rust, which compiles to high-efficiency machine code. The Hydroflow DSL is the lowest level of the Hydro stack and 
requires some knowledge of Rust to use. In time the Hydro Project 
will deliver friendlier, higher-level languages that compile down to Hydroflow.

## This Book
This book will teach you how to set up your environment to get started with Hydroflow, and how to program in the Hydroflow surface syntax.

Keep in mind that Hydroflow is under active development and is constantly
changing. However this book is tested in CI so should always be up-to-date
(though incomplete, for now).

If you have any questions, feel free to [create an issue on Github](https://github.com/hydro-project/hydroflow/issues/new).
