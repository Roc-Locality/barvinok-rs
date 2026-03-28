# Barvinok Rust Object Model

This crate treats isl and barvinok values as opaque reference-counted objects
owned by the underlying C library.

## Core rule

Every wrapper method should be classified by the ownership contract of the raw C
function it calls.

- `__isl_take`: consume an owned Rust wrapper.
- `__isl_keep`: borrow an existing Rust wrapper.
- `__isl_give`: construct and return a fresh Rust wrapper.

That rule is more important than whether the upstream C++ binding presents a
method as `const`. The Rust layer follows the raw RC contract directly.

## Context lifetime

`Context::scope` introduces `ContextRef<'a>`.

`ContextRef<'a>` is intentionally invariant and acts as the lifetime token for
all wrapper objects created inside the scope. The wrappers do not borrow Rust
memory from the context value; instead, they carry the context epoch in their
type so that handles from unrelated scopes cannot be mixed.

This is the design constraint that generated code must preserve.

- Generated constructors accept `ContextRef<'a>` when the raw API takes
  `isl_ctx *`.
- Generated wrapper types keep the same `'a` parameter as the rest of the
  library.
- Generated methods never erase or widen `'a`.

## Operation classes

The code generator and macro layer divide the API into a few explicit classes.

- Constructor: creates a new owned wrapper from a context or another owned
  wrapper.
- Transform: consumes `self` and returns an updated owned wrapper.
- Projection: borrows `self` and returns another owned wrapper.
- Query: borrows `self` and returns a plain value such as `bool`, `u32`, or
  `&str`.

These classes map cleanly onto the existing macro surface and make it easier to
review generated code for lifetime and RC correctness.
