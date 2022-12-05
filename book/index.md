# Introduction

[Hydroflow](https://github.com/hydro-project/hydroflow) is a compiler for low-latency 
dataflow programs, written in Rust. It provides the runtime for the 
[Hydro Project](https://hydro-project.github.io/), which is under development
as a complete compiler stack for distributed programming languages. 

Hydroflow provides a DSL—a *surface syntax*—that it compiles to Rust (and 
from there to machine code). The Hydroflow DSL is the lowest level of the Hydro stack and 
requires some knowledge of Rust to use. In time the Hydro Project 
will deliver friendlier, higher-level languages that compile down to Hydroflow.

## This Book
This book teaches how to set up and get started with Hydroflow and program in its 
surface syntax.

Keep in mind that Hydroflow is under active development and is constantly
changing. However this book is tested in CI so should always be up-to-date
(though incomplete, for now).

If you have any questions, feel free to [create an issue on Github](https://github.com/hydro-project/hydroflow/issues/new).
