# sptr: The Strict Provenance Polyfill

[![crates.io](https://img.shields.io/crates/v/sptr.svg)](https://crates.io/crates/sptr) [![](https://docs.rs/sptr/badge.svg)](https://docs.rs/sptr) ![Rust CI](https://github.com/Gankra/sptr/workflows/Rust/badge.svg?branch=main)




This library provides a stable polyfill for Rust's [Strict Provenance] experiment.

Mapping to STD APIs:

```rust ,ignore
// core::ptr (sptr)
pub fn invalid<T>(addr: usize) -> *const T;
pub fn invalid_mut<T>(addr: usize) -> *mut T;

// core::pointer (sptr::Strict)
pub fn addr(self) -> usize;
pub fn with_addr(self, addr: usize) -> Self;
pub fn map_addr(self, f: impl FnOnce(usize) -> usize) -> Self;

// NON-STANDARD EXTENSIONS (feature = uptr)
sptr::uptr
sptr::iptr

// NON-STANDARD EXTENSIONS (feature = opaque_fn)
sptr::OpaqueFn

// DEPRECATED BY THIS MODEL in core::pointer (sptr::Strict)
// (disable with `default-features = false`)
pub fn to_bits(self) -> usize;
pub fn from_bits(usize) -> Self;
```

Swapping between the two should be as simple as switching between `sptr::` and `ptr::`
for static functions. For methods, you must import `sptr::Strict` into your module for
the extension trait's methods to overlay std. The compiler will (understandably)
complain that you are overlaying std, so you will need to also silence that as
seen in the following example:

```rust
#![allow(unstable_name_collisions)]
use sptr::Strict;

let ptr = sptr::invalid_mut::<u8>(1);
println!("{}", ptr.addr());
```

By default, this crate will also mark methods on pointers as "deprecated" if they are
incompatible with strict_provenance. If you don't want this, set `default-features = false`
in your Cargo.toml.

Rust is the canonical source of definitions for these APIs and semantics, but the docs
here will vaguely try to mirror the docs checked into Rust.


[Strict Provenance]: https://github.com/rust-lang/rust/issues/95228
