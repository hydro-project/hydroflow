# Variadics

Variadic generics in stable Rust

## Variadic Generics?

Variadic generics are one of the most discussed potential Rust features. They would enable
traits, functions, and data structures to be generic over variable length tuples of arbitrary
types.

Currently you can only implement generic code for tuples of a specific length. If you want to
handle tuples of varying lengths you must write a separate implementation for each length. This
leads to the notorious limitation that traits in Rust generally
[only apply for tuples up to length 12](https://doc.rust-lang.org/std/primitive.tuple.html#impl-From%3C(T,+T,+T,+T,+T,+T,+T,+T,+T,+T,+T,+T)%3E-for-%5BT;+12%5D).

Variadic generics allow generic code to handle tuples of any length.

## Tuple lists

Although variadic generics fundamentally require changing the Rust compiler, we can emulate
pretty well with tuple lists.

Any tuple `(A, B, C, D)` can be mapped to (and from) a recursive tuple `(A, (B, (C, (D, ()))))`.

Each element consists of a nested pair `(Item, Rest)`, where `Item` is tuple element and `Rest`
is the rest of the list. For last element `Rest` is a unit tuple `()`. Unlike regular flat
tuples, these recursive tuples can be effectively reasoned about in stable Rust.

You may recognize this fundamental structure from [`cons` lists in Lisp](https://en.wikipedia.org/wiki/Cons#Lists)
as well as [`HList`s in Haskell](https://hackage.haskell.org/package/HList/docs/Data-HList-HList.html).

This crate calls these lists "variadics" and provides traits and macros to allow simple,
ergonimc use of them.

## Usage

[`var_expr!`] creates variadic values/expressions,
[`var_type!`] creates variadic types,
and [`var_args!`] creates variadic patterns (used in unpacking arguments, on the left side of `let`
declarations, etc.).

These macros support the "spread" syntax `...`, also known as "splat". For example, `var_expr!(a, ...var_b, ...var_c, d)`
will concatenate `a`, the items of `var_b`, the items of `var_c` and `d` together into a single
variadic list.

## Acknowledgements

This crate is based on [`tuple_list` by VFLashM](https://github.com/VFLashM/tuple_list), which is MIT licensed:

<details>
    <summary>MIT license</summary>

```text
Copyright (c) 2020 Valerii Lashmanov

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE
```
</details>
