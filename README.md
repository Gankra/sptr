# sptr: The Strict Provenance Polyfill

[![crates.io](https://img.shields.io/crates/v/sptr.svg)](https://crates.io/crates/sptr) [![](https://docs.rs/sptr/badge.svg)](https://docs.rs/sptr) ![Rust CI](https://github.com/Gankra/sptr/workflows/Rust/badge.svg?branch=main)




This library provides a stable polyfill for Rust's [Strict Provenance] experiment.

# Mapping to STD APIs:

This crate "overlays" a bunch of unstable std apis, here are the mappings:

## core::ptr (sptr)

* `pub fn `[`invalid`]`<T>(addr: usize) -> *const T;`
* `pub fn `[`invalid_mut`]`<T>(addr: usize) -> *mut T;`
* `pub fn `[`from_exposed_addr`]`<T>(addr: usize) -> *const T;`
* `pub fn `[`from_exposed_addr_mut`]`<T>(addr: usize) -> *mut T;`


## core::pointer (sptr::Strict)

* `pub fn `[`addr`]`(self) -> usize;`
* `pub fn `[`expose_addr`]`(self) -> usize;`
* `pub fn `[`with_addr`]`(self, addr: usize) -> Self;`
* `pub fn `[`map_addr`]`(self, f: impl FnOnce(usize) -> usize) -> Self;`


## NON-STANDARD EXTENSIONS (disabled by default, use at your own risk)

* `sptr::`[`uptr`] (feature = uptr)
* `sptr::`[`iptr`] (feature = uptr)
* `sptr::`[`OpaqueFnPtr`] (feature = opaque_fn)




# Applying The Overlay

Swapping between sptr and core::ptr should be as simple as switching between `sptr::` and `ptr::`
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


[`invalid`]: https://docs.rs/sptr/latest/sptr/fn.invalid.html
[`invalid_mut`]: https://docs.rs/sptr/latest/sptr/fn.invalid_mut.html
[`from_exposed_addr`]: https://docs.rs/sptr/latest/sptr/fn.from_exposed_addr.html
[`from_exposed_addr_mut`]: https://docs.rs/sptr/latest/sptr/fn.from_exposed_addr_mut.html
[`addr`]: https://docs.rs/sptr/latest/sptr/trait.Strict.html#tymethod.addr
[`expose_addr`]: https://docs.rs/sptr/latest/sptr/trait.Strict.html#tymethod.expose_addr
[`with_addr`]: https://docs.rs/sptr/latest/sptr/trait.Strict.html#tymethod.with_addr
[`map_addr`]: https://docs.rs/sptr/latest/sptr/trait.Strict.html#tymethod.map_addr
[`uptr`]: https://docs.rs/sptr/latest/sptr/int/struct.uptr.html
[`iptr`]: https://docs.rs/sptr/latest/sptr/int/struct.iptr.html
[`OpaqueFnPtr`]: https://docs.rs/sptr/latest/sptr/func/struct.OpaqueFnPtr.html
