# `barvinok-macros`

`barvinok-macros` contains the declarative wrapper-building macros used by the
high-level `barvinok` crate.

The macros are intentionally small and mechanical. They do not invent a new
ownership model; they encode the ownership and lifetime model that already
exists in isl and in `barvinok`'s `ContextRef<'a>` design.

## Object model

The important mental model is:

- An isl object is an opaque heap allocation managed by isl reference counts.
- `__isl_take` means the callee consumes one owned reference.
- `__isl_keep` means the callee only borrows the existing reference.
- `__isl_give` means the callee returns a newly owned reference.

The Rust wrappers mirror that model directly.

- `Self` by value means the Rust side is transferring ownership of the current
  handle to isl.
- `&Self` means the Rust side is only lending access to isl.
- Returned wrapper values always own the handle they contain.

This is why the macro families are separated by operation kind rather than by
syntax.

- `isl_ctor!`: create a fresh wrapper value from a context or another owned
  wrapper.
- `isl_transform!`: consume `self` and return a new owned value. This models
  the common isl pattern where an operation takes ownership of an object and
  returns the updated value.
- `isl_project!`: borrow `self` and return another owned value. This is used
  for `__isl_keep` accessors such as `get_space`.
- `isl_flag!`: borrow `self` and return a boolean-like query result.
- `isl_size!`: borrow `self` and return an `isl_size`-derived dimension/count.
- `isl_str!`: borrow `self` and read a string view from isl.

## Lifetime model

The lifetime parameter on wrapper types is not a borrow of the raw pointer
itself. It is an epoch marker tied to `ContextRef<'a>`.

`ContextRef<'a>` in the main crate is intentionally invariant, using a ghost
marker (`PhantomData<*mut &'a Context>`). That design matters:

- wrappers created inside `Context::scope` cannot be widened to a longer
  lifetime;
- wrapper values from different context epochs cannot be mixed accidentally;
- the macros only propagate `'a`; they never manufacture or erase it.

The wrapper handle types also use invariant markers. The effect is similar to a
ghost-cell style token: the context lifetime names the region in which those
opaque isl handles are allowed to exist.

## Why the macros stay declarative

The macros deliberately avoid hidden copies or implicit lifetime tricks.
Whether an operation is a consuming transform or a borrowing projection must be
visible in the macro that defines it.

That keeps two soundness properties obvious in generated code:

1. ownership transfer follows isl's RC annotations;
2. every produced wrapper remains tied to the originating `ContextRef<'a>`.
