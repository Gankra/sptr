#![allow(unstable_name_collisions)]
#![no_std]

//! This library provides a stable polyfill for Rust's [Strict Provenance] experiment.
//!
//! # Mapping to STD APIs:
//!
//! This crate "overlays" a bunch of unstable std apis, here are the mappings:
//!
//! ## core::ptr (sptr)
//!
//! * `pub fn `[`invalid`]`<T>(addr: usize) -> *const T;`
//! * `pub fn `[`invalid_mut`]`<T>(addr: usize) -> *mut T;`
//! * `pub fn `[`from_exposed_addr`]`<T>(addr: usize) -> *const T;`
//! * `pub fn `[`from_exposed_addr_mut`]`<T>(addr: usize) -> *mut T;`
//!
//!
//! ## core::pointer (sptr::Strict)
//!
//! * `pub fn `[`addr`]`(self) -> usize;`
//! * `pub fn `[`expose_addr`]`(self) -> usize;`
//! * `pub fn `[`with_addr`]`(self, addr: usize) -> Self;`
//! * `pub fn `[`map_addr`]`(self, f: impl FnOnce(usize) -> usize) -> Self;`
//!
//!
//! ## NON-STANDARD EXTENSIONS (disabled by default, use at your own risk)
//!
//! * `sptr::`[`uptr`] (feature = uptr)
//! * `sptr::`[`iptr`] (feature = uptr)
//! * `sptr::`[`OpaqueFnPtr`] (feature = opaque_fn)
//!
//!
//!
//!
//! # Applying The Overlay
//!
//! Swapping between sptr and core::ptr should be as simple as switching between `sptr::` and `ptr::`
//! for static functions. For methods, you must import `sptr::Strict` into your module for
//! the extension trait's methods to overlay std. The compiler will (understandably)
//! complain that you are overlaying std, so you will need to also silence that as
//! seen in the following example:
//!
//! ```rust
//! #![allow(unstable_name_collisions)]
//! use sptr::Strict;
//!
//! let ptr = sptr::invalid_mut::<u8>(1);
//! println!("{}", ptr.addr());
//! ```
//!
//! By default, this crate will also mark methods on pointers as "deprecated" if they are
//! incompatible with strict_provenance. If you don't want this, set `default-features = false`
//! in your Cargo.toml.
//!
//! Rust is the canonical source of definitions for these APIs and semantics, but the docs
//! here will vaguely try to mirror the docs checked into Rust.
//!
//! The following explanation of the model should also appear at the top of `core::ptr`:
//!
//! # Strict Provenance
//!
//! **The following text is non-normative, insufficiently formal, and is an extremely strict
//! interpretation of provenance. It's ok if your code doesn't strictly conform to it.**
//!
//! [Strict Provenance][] is an experimental set of APIs that help tools that try
//! to validate the memory-safety of your program's execution. Notably this includes [Miri][]
//! and [CHERI][], which can detect when you access out of bounds memory or otherwise violate
//! Rust's memory model.
//!
//! Provenance must exist in some form for any programming
//! language compiled for modern computer architectures, but specifying a model for provenance
//! in a way that is useful to both compilers and programmers is an ongoing challenge.
//! The [Strict Provenance][] experiment seeks to explore the question: *what if we just said you
//! couldn't do all the nasty operations that make provenance so messy?*
//!
//! What APIs would have to be removed? What APIs would have to be added? How much would code
//! have to change, and is it worse or better now? Would any patterns become truly inexpressible?
//! Could we carve out special exceptions for those patterns? Should we?
//!
//! A secondary goal of this project is to see if we can disamiguate the many functions of
//! pointer<->integer casts enough for the definition of `usize` to be loosened so that it
//! isn't *pointer*-sized but address-space/offset/allocation-sized (we'll probably continue
//! to conflate these notions). This would potentially make it possible to more efficiently
//! target platforms where pointers are larger than offsets, such as CHERI and maybe some
//! segmented architecures.
//!
//! ## Provenance
//!
//! **This section is *non-normative* and is part of the [Strict Provenance][] experiment.**
//!
//! Pointers are not *simply* an "integer" or "address". For instance, it's uncontroversial
//! to say that a Use After Free is clearly Undefined Behaviour, even if you "get lucky"
//! and the freed memory gets reallocated before your read/write (in fact this is the
//! worst-case scenario, UAFs would be much less concerning if this didn't happen!).
//! To rationalize this claim, pointers need to somehow be *more* than just their addresses:
//! they must have provenance.
//!
//! When an allocation is created, that allocation has a unique Original Pointer. For alloc
//! APIs this is literally the pointer the call returns, and for local variables and statics,
//! this is the name of the variable/static. This is mildly overloading the term "pointer"
//! for the sake of brevity/exposition.
//!
//! The Original Pointer for an allocation is guaranteed to have unique access to the entire
//! allocation and *only* that allocation. In this sense, an allocation can be thought of
//! as a "sandbox" that cannot be broken into or out of. *Provenance* is the permission
//! to access an allocation's sandbox and has both a *spatial* and *temporal* component:
//!
//! * Spatial: A range of bytes that the pointer is allowed to access.
//! * Temporal: The lifetime (of the allocation) that access to these bytes is tied to.
//!
//! Spatial provenance makes sure you don't go beyond your sandbox, while temporal provenance
//! makes sure that you can't "get lucky" after your permission to access some memory
//! has been revoked (either through deallocations or borrows expiring).
//!
//! Provenance is implicitly shared with all pointers transitively derived from
//! The Original Pointer through operations like [`offset`], borrowing, and pointer casts.
//! Some operations may *shrink* the derived provenance, limiting how much memory it can
//! access or how long it's valid for (i.e. borrowing a subfield and subslicing).
//!
//! Shrinking provenance cannot be undone: even if you "know" there is a larger allocation, you
//! can't derive a pointer with a larger provenance. Similarly, you cannot "recombine"
//! two contiguous provenances back into one (i.e. with a `fn merge(&[T], &[T]) -> &[T]`).
//!
//! A reference to a value always has provenance over exactly the memory that field occupies.
//! A reference to a slice always has provenance over exactly the range that slice describes.
//!
//! If an allocation is deallocated, all pointers with provenance to that allocation become
//! invalidated, and effectively lose their provenance.
//!
//! The strict provenance experiment is mostly only interested in exploring stricter *spatial*
//! provenance. In this sense it can be thought of as a subset of the more ambitious and
//! formal [Stacked Borrows][] research project, which is what tools like [Miri][] are based on.
//! In particular, Stacked Borrows is necessary to properly describe what borrows are allowed
//! to do and when they become invalidated. This necessarily involves much more complex
//! *temporal* reasoning than simply identifying allocations. Adjusting APIs and code
//! for the strict provenance experiment will also greatly help Stacked Borrows.
//!
//!
//! ## Pointer Vs Addresses
//!
//! **This section is *non-normative* and is part of the [Strict Provenance][] experiment.**
//!
//! One of the largest historical issues with trying to define provenance is that programmers
//! freely convert between pointers and integers. Once you allow for this, it generally becomes
//! impossible to accurately track and preserve provenance information, and you need to appeal
//! to very complex and unreliable heuristics. But of course, converting between pointers and
//! integers is very useful, so what can we do?
//!
//! Also did you know WASM is actually a "Harvard Architecture"? As in function pointers are
//! handled completely differently from data pointers? And we kind of just shipped Rust on WASM
//! without really addressing the fact that we let you freely convert between function pointers
//! and data pointers, because it mostly Just Works? Let's just put that on the "pointer casts
//! are dubious" pile.
//!
//! Strict Provenance attempts to square these circles by decoupling Rust's traditional conflation
//! of pointers and `usize` (and `isize`), and defining a pointer to semantically contain the
//! following information:
//!
//! * The **address-space** it is part of.
//! * The **address** it points to, which can be represented by a `usize`.
//! * The **provenance** it has, defining the memory it has permission to access.
//!
//! Under Strict Provenance, a usize *cannot* accurately represent a pointer, and converting from
//! a pointer to a usize is generally an operation which *only* extracts the address. It is
//! therefore *impossible* to construct a valid pointer from a usize because there is no way
//! to restore the address-space and provenance. In other words, pointer-integer-pointer
//! roundtrips are not possible (in the sense that the resulting pointer is not dereferencable).
//!
//! The key insight to making this model *at all* viable is the [`with_addr`][] method:
//!
//! ```text
//!     /// Creates a new pointer with the given address.
//!     ///
//!     /// This performs the same operation as an `addr as ptr` cast, but copies
//!     /// the *address-space* and *provenance* of `self` to the new pointer.
//!     /// This allows us to dynamically preserve and propagate this important
//!     /// information in a way that is otherwise impossible with a unary cast.
//!     ///
//!     /// This is equivalent to using `wrapping_offset` to offset `self` to the
//!     /// given address, and therefore has all the same capabilities and restrictions.
//!     pub fn with_addr(self, addr: usize) -> Self;
//! ```
//!
//! So you're still able to drop down to the address representation and do whatever
//! clever bit tricks you want *as long as* you're able to keep around a pointer
//! into the allocation you care about that can "reconstitute" the other parts of the pointer.
//! Usually this is very easy, because you only are taking a pointer, messing with the address,
//! and then immediately converting back to a pointer. To make this use case more ergonomic,
//! we provide the [`map_addr`][] method.
//!
//! To help make it clear that code is "following" Strict Provenance semantics, we also provide an
//! [`addr`][] method which promises that the returned address is not part of a
//! pointer-usize-pointer roundtrip. In the future we may provide a lint for pointer<->integer
//! casts to help you audit if your code conforms to strict provenance.
//!
//!
//! ## Using Strict Provenance
//!
//! Most code needs no changes to conform to strict provenance, as the only really concerning
//! operation that *wasn't* obviously already Undefined Behaviour is casts from usize to a
//! pointer. For code which *does* cast a usize to a pointer, the scope of the change depends
//! on exactly what you're doing.
//!
//! In general you just need to make sure that if you want to convert a usize address to a
//! pointer and then use that pointer to read/write memory, you need to keep around a pointer
//! that has sufficient provenance to perform that read/write itself. In this way all of your
//! casts from an address to a pointer are essentially just applying offsets/indexing.
//!
//! This is generally trivial to do for simple cases like tagged pointers *as long as you
//! represent the tagged pointer as an actual pointer and not a usize*. For instance:
//!
//! ```
//! // #![feature(strict_provenance)]
//! #![allow(unstable_name_collisions)]
//! use sptr::Strict;
//!
//! unsafe {
//!     // A flag we want to pack into our pointer
//!     static HAS_DATA: usize = 0x1;
//!     static FLAG_MASK: usize = !HAS_DATA;
//!
//!     // Our value, which must have enough alignment to have spare least-significant-bits.
//!     let my_precious_data: u32 = 17;
//!     assert!(core::mem::align_of::<u32>() > 1);
//!
//!     // Create a tagged pointer
//!     let ptr = &my_precious_data as *const u32;
//!     let tagged = ptr.map_addr(|addr| addr | HAS_DATA);
//!
//!     // Check the flag:
//!     if tagged.addr() & HAS_DATA != 0 {
//!         // Untag and read the pointer
//!         let data = *tagged.map_addr(|addr| addr & FLAG_MASK);
//!         assert_eq!(data, 17);
//!     } else {
//!         unreachable!()
//!     }
//! }
//! ```
//!
//! (Yes, if you've been using AtomicUsize for pointers in concurrent datastructures, you should
//! be using AtomicPtr instead. If that messes up the way you atomically manipulate pointers,
//! we would like to know why, and what needs to be done to fix it.)
//!
//! Something more complicated and just generally *evil* like a XOR-List requires more significant
//! changes like allocating all nodes in a pre-allocated Vec or Arena and using a pointer
//! to the whole allocation to reconstitute the XORed addresses.
//!
//! Situations where a valid pointer *must* be created from just an address, such as baremetal code
//! accessing a memory-mapped interface at a fixed address, are an open question on how to support.
//! These situations *will* still be allowed, but we might require some kind of "I know what I'm
//! doing" annotation to explain the situation to the compiler. It's also possible they need no
//! special attention at all, because they're generally accessing memory outside the scope of
//! "the abstract machine", or already using "I know what I'm doing" annotations like "volatile".
//!
//! Under [Strict Provenance] is is Undefined Behaviour to:
//!
//! * Access memory through a pointer that does not have provenance over that memory.
//!
//! * [`offset`] a pointer to or from an address it doesn't have provenance over.
//!   This means it's always UB to offset a pointer derived from something deallocated,
//!   even if the offset is 0. Note that a pointer "one past the end" of its provenance
//!   is not actually outside its provenance, it just has 0 bytes it can load/store.
//!
//! But it *is* still sound to:
//!
//! * Create an invalid pointer from just an address (see [`ptr::invalid`][]). This can
//!   be used for sentinel values like `null` *or* to represent a tagged pointer that will
//!   never be dereferencable. In general, it is always sound for an integer to pretend
//!   to be a pointer "for fun" as long as you don't use operations on it which require
//!   it to be valid (offset, read, write, etc).
//!
//! * Forge an allocation of size zero at any sufficiently aligned non-null address.
//!   i.e. the usual "ZSTs are fake, do what you want" rules apply *but* this only applies
//!   for actual forgery (integers cast to pointers). If you borrow some struct's field
//!   that *happens* to be zero-sized, the resulting pointer will have provenance tied to
//!   that allocation and it will still get invalidated if the allocation gets deallocated.
//!   In the future we may introduce an API to make such a forged allocation explicit.
//!
//! * [`wrapping_offset`][] a pointer outside its provenance. This includes invalid pointers
//!   which have "no" provenance. Unfortunately there may be practical limits on this for a
//!   particular platform, and it's an open question as to how to specify this (if at all).
//!   Notably, [CHERI][] relies on a compression scheme that can't handle a
//!   pointer getting offset "too far" out of bounds. If this happens, the address
//!   returned by `addr` will be the value you expect, but the provenance will get invalidated
//!   and using it to read/write will fault. The details of this are architecture-specific
//!   and based on alignment, but the buffer on either side of the pointer's range is pretty
//!   generous (think kilobytes, not bytes).
//!
//! * Compare arbitrary pointers by address. Addresses *are* just integers and so there is
//!   always a coherent answer, even if the pointers are invalid or from different
//!   address-spaces/provenances. Of course, comparing addresses from different address-spaces
//!   is generally going to be *meaningless*, but so is comparing Kilograms to Meters, and Rust
//!   doesn't prevent that either. Similarly, if you get "lucky" and notice that a pointer
//!   one-past-the-end is the "same" address as the start of an unrelated allocation, anything
//!   you do with that fact is *probably* going to be gibberish. The scope of that gibberish
//!   is kept under control by the fact that the two pointers *still* aren't allowed to access
//!   the other's allocation (bytes), because they still have different provenance.
//!
//! * Perform pointer tagging tricks. This falls out of [`wrapping_offset`] but is worth
//!   mentioning in more detail because of the limitations of [CHERI][]. Low-bit tagging
//!   is very robust, and often doesn't even go out of bounds because types ensure
//!   size >= align (and over-aligning actually gives CHERI more flexibility). Anything
//!   more complex than this rapidly enters "extremely platform-specific" territory as
//!   certain things may or may not be allowed based on specific supported operations.
//!   For instance, ARM explicitly supports high-bit tagging, and so CHERI on ARM inherits
//!   that and should support it.
//!
//! ## Pointer-usize-pointer roundtrips and 'exposed' provenance
//!
//! **This section is *non-normative* and is part of the [Strict Provenance] experiment.**
//!
//! As discussed above, pointer-usize-pointer roundtrips are not possible under [Strict Provenance].
//! However, there exists legacy Rust code that is full of such roundtrips, and legacy platform APIs
//! regularly assume that `usize` can capture all the information that makes up a pointer. There
//! also might be code that cannot be ported to Strict Provenance (which is something we would [like
//! to hear about][Strict Provenance]).
//!
//! For situations like this, there is a fallback plan, a way to 'opt out' of Strict Provenance.
//! However, note that this makes your code a lot harder to specify, and the code will not work
//! (well) with tools like [Miri] and [CHERI].
//!
//! This fallback plan is provided by the [`expose_addr`] and [`from_exposed_addr`] methods (which
//! are equivalent to `as` casts between pointers and integers). [`expose_addr`] is a lot like
//! [`addr`], but additionally adds the provenance of the pointer to a global list of 'exposed'
//! provenances. (This list is purely conceptual, it exists for the purpose of specifying Rust but
//! is not materialized in actual executions, except in tools like [Miri].) [`from_exposed_addr`]
//! can be used to construct a pointer with one of these previously 'exposed' provenances.
//! [`from_exposed_addr`] takes only `addr: usize` as arguments, so unlike in [`with_addr`] there is
//! no indication of what the correct provenance for the returned pointer is -- and that is exactly
//! what makes pointer-usize-pointer roundtrips so tricky to rigorously specify! There is no
//! algorithm that decides which provenance will be used. You can think of this as "guessing" the
//! right provenance, and the guess will be "maximally in your favor", in the sense that if there is
//! any way to avoid undefined behavior, then that is the guess that will be taken. However, if
//! there is *no* previously 'exposed' provenance that justifies the way the returned pointer will
//! be used, the program has undefined behavior.
//!
//! Using [`expose_addr`] or [`from_exposed_addr`] (or the equivalent `as` casts) means that code is
//! *not* following Strict Provenance rules. The goal of the Strict Provenance experiment is to
//! determine whether it is possible to use Rust without [`expose_addr`] and [`from_exposed_addr`].
//! If this is successful, it would be a major win for avoiding specification complexity and to
//! facilitate adoption of tools like [CHERI] and [Miri] that can be a big help in increasing the
//! confidence in (unsafe) Rust code.
//!
//!
//! [aliasing]: https://doc.rust-lang.org/nightly/nomicon/aliasing.html
//! [book]: https://doc.rust-lang.org/nightly/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer
//! [ub]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
//! [zst]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
//! [atomic operations]: core::sync::atomic
//! [`offset`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.offset
//! [`wrapping_offset`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.wrapping_offset
//! [`with_addr`]: Strict::with_addr
//! [`map_addr`]: Strict::map_addr
//! [`addr`]: Strict::addr
//! [`ptr::invalid`]: crate::invalid
//! [`expose_addr`]: Strict::expose_addr
//! [`from_exposed_addr`]: crate::from_exposed_addr
//! [Miri]: https://github.com/rust-lang/miri
//! [CHERI]: https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/
//! [Strict Provenance]: https://github.com/rust-lang/rust/issues/95228
//! [Stacked Borrows]: https://plv.mpi-sws.org/rustbelt/stacked-borrows/

/// Creates an invalid pointer with the given address.
///
/// This is different from `addr as *const T`, which creates a pointer that picks up a previously
/// exposed provenance. See [`from_exposed_addr`] for more details on that operation.
///
/// The module's top-level documentation discusses the precise meaning of an "invalid"
/// pointer but essentially this expresses that the pointer is not associated
/// with any actual allocation and is little more than a usize address in disguise.
///
/// This pointer will have no provenance associated with it and is therefore
/// UB to read/write/offset. This mostly exists to facilitate things
/// like `ptr::null` and `NonNull::dangling` which make invalid pointers.
///
/// (Standard "Zero-Sized-Types get to cheat and lie" caveats apply, although it
/// may be desirable to give them their own API just to make that 100% clear.)
///
/// This API and its claimed semantics are part of the Strict Provenance experiment,
/// see the [module documentation][crate] for details.
#[inline(always)]
#[must_use]
pub const fn invalid<T>(addr: usize) -> *const T {
    // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
    // We use transmute rather than a cast so tools like Miri can tell that this
    // is *not* the same as from_exposed_addr.
    // SAFETY: every valid integer is also a valid pointer (as long as you don't dereference that
    // pointer).
    #[cfg(miri)]
    return unsafe { core::mem::transmute(addr) };
    // Outside Miri we keep using casts, so that we can be a `const fn` on old Rust (pre-1.56).
    #[cfg(not(miri))]
    return addr as *const T;
}

/// Creates an invalid mutable pointer with the given address.
///
/// This is different from `addr as *mut T`, which creates a pointer that picks up a previously
/// exposed provenance. See [`from_exposed_addr_mut`] for more details on that operation.
///
/// The module's top-level documentation discusses the precise meaning of an "invalid"
/// pointer but essentially this expresses that the pointer is not associated
/// with any actual allocation and is little more than a usize address in disguise.
///
/// This pointer will have no provenance associated with it and is therefore
/// UB to read/write/offset. This mostly exists to facilitate things
/// like `ptr::null` and `NonNull::dangling` which make invalid pointers.
///
/// (Standard "Zero-Sized-Types get to cheat and lie" caveats apply, although it
/// may be desirable to give them their own API just to make that 100% clear.)
///
/// This API and its claimed semantics are part of the Strict Provenance experiment,
/// see the [module documentation][crate] for details.
#[inline(always)]
#[must_use]
pub const fn invalid_mut<T>(addr: usize) -> *mut T {
    // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
    // We use transmute rather than a cast so tools like Miri can tell that this
    // is *not* the same as from_exposed_addr.
    // SAFETY: every valid integer is also a valid pointer (as long as you don't dereference that
    // pointer).
    #[cfg(miri)]
    return unsafe { core::mem::transmute(addr) };
    // Outside Miri we keep using casts, so that we can be a `const fn` on old Rust (pre-1.56).
    #[cfg(not(miri))]
    return addr as *mut T;
}

/// Convert an address back to a pointer, picking up a previously 'exposed' provenance.
///
/// This is equivalent to `addr as *const T`. The provenance of the returned pointer is that of *any*
/// pointer that was previously passed to [`expose_addr`][Strict::expose_addr] or a `ptr as usize`
/// cast. If there is no previously 'exposed' provenance that justifies the way this pointer will be
/// used, the program has undefined behavior. Note that there is no algorithm that decides which
/// provenance will be used. You can think of this as "guessing" the right provenance, and the guess
/// will be "maximally in your favor", in the sense that if there is any way to avoid undefined
/// behavior, then that is the guess that will be taken.
///
/// On platforms with multiple address spaces, it is your responsibility to ensure that the
/// address makes sense in the address space that this pointer will be used with.
///
/// Using this method means that code is *not* following strict provenance rules. "Guessing" a
/// suitable provenance complicates specification and reasoning and may not be supported by
/// tools that help you to stay conformant with the Rust memory model, so it is recommended to
/// use [`with_addr`][Strict::with_addr] wherever possible.
///
/// On most platforms this will produce a value with the same bytes as the address. Platforms
/// which need to store additional information in a pointer may not support this operation,
/// since it is generally not possible to actually *compute* which provenance the returned
/// pointer has to pick up.
///
/// This API and its claimed semantics are part of the Strict Provenance experiment, see the
/// [module documentation][crate] for details.
#[must_use]
#[inline]
pub fn from_exposed_addr<T>(addr: usize) -> *const T
where
    T: Sized,
{
    // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
    addr as *const T
}

/// Convert an address back to a mutable pointer, picking up a previously 'exposed' provenance.
///
/// This is equivalent to `addr as *mut T`. The provenance of the returned pointer is that of *any*
/// pointer that was previously passed to [`expose_addr`][Strict::expose_addr] or a `ptr as usize`
/// cast. If there is no previously 'exposed' provenance that justifies the way this pointer will be
/// used, the program has undefined behavior. Note that there is no algorithm that decides which
/// provenance will be used. You can think of this as "guessing" the right provenance, and the guess
/// will be "maximally in your favor", in the sense that if there is any way to avoid undefined
/// behavior, then that is the guess that will be taken.
///
/// On platforms with multiple address spaces, it is your responsibility to ensure that the
/// address makes sense in the address space that this pointer will be used with.
///
/// Using this method means that code is *not* following strict provenance rules. "Guessing" a
/// suitable provenance complicates specification and reasoning and may not be supported by
/// tools that help you to stay conformant with the Rust memory model, so it is recommended to
/// use [`with_addr`][Strict::with_addr] wherever possible.
///
/// On most platforms this will produce a value with the same bytes as the address. Platforms
/// which need to store additional information in a pointer may not support this operation,
/// since it is generally not possible to actually *compute* which provenance the returned
/// pointer has to pick up.
///
/// This API and its claimed semantics are part of the Strict Provenance experiment, see the
/// [module documentation][crate] for details.
#[must_use]
#[inline]
pub fn from_exposed_addr_mut<T>(addr: usize) -> *mut T
where
    T: Sized,
{
    // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
    addr as *mut T
}

mod private {
    pub trait Sealed {}
}

pub trait Strict: private::Sealed {
    type Pointee;
    /// Gets the "address" portion of the pointer.
    ///
    /// This is similar to `self as usize`, which semantically discards *provenance* and
    /// *address-space* information. However, unlike `self as usize`, casting the returned address
    /// back to a pointer yields [`invalid`][], which is undefined behavior to dereference. To
    /// properly restore the lost information and obtain a dereferencable pointer, use
    /// [`with_addr`][Strict::with_addr] or [`map_addr`][Strict::map_addr].
    ///
    /// If using those APIs is not possible because there is no way to preserve a pointer with the
    /// required provenance, use [`expose_addr`][Strict::expose_addr] and
    /// [`from_exposed_addr`][from_exposed_addr] instead. However, note that this makes
    /// your code less portable and less amenable to tools that check for compliance with the Rust
    /// memory model.
    ///
    /// On most platforms this will produce a value with the same bytes as the original
    /// pointer, because all the bytes are dedicated to describing the address.
    /// Platforms which need to store additional information in the pointer may
    /// perform a change of representation to produce a value containing only the address
    /// portion of the pointer. What that means is up to the platform to define.
    ///
    /// This API and its claimed semantics are part of the Strict Provenance experiment, and as such
    /// might change in the future (including possibly weakening this so it becomes wholly
    /// equivalent to `self as usize`). See the [module documentation][crate] for details.
    #[must_use]
    fn addr(self) -> usize
    where
        Self::Pointee: Sized;

    /// Gets the "address" portion of the pointer, and 'exposes' the "provenance" part for future
    /// use in [`from_exposed_addr`][].
    ///
    /// This is equivalent to `self as usize`, which semantically discards *provenance* and
    /// *address-space* information. Furthermore, this (like the `as` cast) has the implicit
    /// side-effect of marking the provenance as 'exposed', so on platforms that support it you can
    /// later call [`from_exposed_addr`][] to reconstitute the original pointer including its
    /// provenance. (Reconstructing address space information, if required, is your responsibility.)
    ///
    /// Using this method means that code is *not* following Strict Provenance rules. Supporting
    /// [`from_exposed_addr`][] complicates specification and reasoning and may not be supported by
    /// tools that help you to stay conformant with the Rust memory model, so it is recommended to
    /// use [`addr`][Strict::addr] wherever possible.
    ///
    /// On most platforms this will produce a value with the same bytes as the original pointer,
    /// because all the bytes are dedicated to describing the address. Platforms which need to store
    /// additional information in the pointer may not support this operation, since the 'expose'
    /// side-effect which is required for [`from_exposed_addr`][] to work is typically not
    /// available.
    ///
    /// This API and its claimed semantics are part of the Strict Provenance experiment, see the
    /// [module documentation][crate] for details.
    ///
    /// [`from_exposed_addr`]: crate::from_exposed_addr
    #[must_use]
    fn expose_addr(self) -> usize
    where
        Self::Pointee: Sized;

    /// Creates a new pointer with the given address.
    ///
    /// This performs the same operation as an `addr as ptr` cast, but copies
    /// the *address-space* and *provenance* of `self` to the new pointer.
    /// This allows us to dynamically preserve and propagate this important
    /// information in a way that is otherwise impossible with a unary cast.
    ///
    /// This is equivalent to using [`wrapping_offset`][] to offset
    /// `self` to the given address, and therefore has all the same capabilities and restrictions.
    ///
    /// This API and its claimed semantics are part of the Strict Provenance experiment,
    /// see the [module documentation][crate] for details.
    ///
    /// [`wrapping_offset`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.wrapping_offset
    #[must_use]
    fn with_addr(self, addr: usize) -> Self
    where
        Self::Pointee: Sized;

    /// Creates a new pointer by mapping `self`'s address to a new one.
    ///
    /// This is a convenience for [`with_addr`][Strict::with_addr], see that method for details.
    ///
    /// This API and its claimed semantics are part of the Strict Provenance experiment,
    /// see the [module documentation][crate] for details.
    #[must_use]
    fn map_addr(self, f: impl FnOnce(usize) -> usize) -> Self
    where
        Self::Pointee: Sized;
}

impl<T> private::Sealed for *mut T {}
impl<T> private::Sealed for *const T {}

impl<T> Strict for *mut T {
    type Pointee = T;

    #[must_use]
    #[inline]
    fn addr(self) -> usize
    where
        T: Sized,
    {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        // SAFETY: Pointer-to-integer transmutes are valid (if you are okay with losing the
        // provenance).
        unsafe { core::mem::transmute(self) }
    }

    #[must_use]
    #[inline]
    fn expose_addr(self) -> usize
    where
        T: Sized,
    {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        self as usize
    }

    #[must_use]
    #[inline]
    fn with_addr(self, addr: usize) -> Self
    where
        T: Sized,
    {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        //
        // In the mean-time, this operation is defined to be "as if" it was
        // a wrapping_offset, so we can emulate it as such. This should properly
        // restore pointer provenance even under today's compiler.
        let self_addr = self.addr() as isize;
        let dest_addr = addr as isize;
        let offset = dest_addr.wrapping_sub(self_addr);

        // This is the canonical desugarring of this operation,
        // but `pointer::cast` was only stabilized in 1.38.
        // self.cast::<u8>().wrapping_offset(offset).cast::<T>()
        (self as *mut u8).wrapping_offset(offset) as *mut T
    }

    #[must_use]
    #[inline]
    fn map_addr(self, f: impl FnOnce(usize) -> usize) -> Self
    where
        T: Sized,
    {
        self.with_addr(f(self.addr()))
    }
}

impl<T> Strict for *const T {
    type Pointee = T;

    #[must_use]
    #[inline]
    fn addr(self) -> usize
    where
        T: Sized,
    {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        // SAFETY: Pointer-to-integer transmutes are valid (if you are okay with losing the
        // provenance).
        unsafe { core::mem::transmute(self) }
    }

    #[must_use]
    #[inline]
    fn expose_addr(self) -> usize
    where
        T: Sized,
    {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        self as usize
    }

    #[must_use]
    #[inline]
    fn with_addr(self, addr: usize) -> Self
    where
        T: Sized,
    {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        //
        // In the mean-time, this operation is defined to be "as if" it was
        // a wrapping_offset, so we can emulate it as such. This should properly
        // restore pointer provenance even under today's compiler.
        let self_addr = self.addr() as isize;
        let dest_addr = addr as isize;
        let offset = dest_addr.wrapping_sub(self_addr);

        // This is the canonical desugarring of this operation,
        // but `pointer::cast` was only stabilized in 1.38.
        // self.cast::<u8>().wrapping_offset(offset).cast::<T>()
        (self as *const u8).wrapping_offset(offset) as *const T
    }

    #[must_use]
    #[inline]
    fn map_addr(self, f: impl FnOnce(usize) -> usize) -> Self
    where
        T: Sized,
    {
        self.with_addr(f(self.addr()))
    }
}

#[cfg(test)]
mod test {
    #![allow(unstable_name_collisions)]
    use crate::Strict;

    #[test]
    fn test_overlay() {
        let null_ptr = core::ptr::null_mut::<u8>();
        let ptr = crate::invalid_mut::<u8>(0);
        assert_eq!(ptr, null_ptr);

        let addr = ptr.addr();
        assert_eq!(addr, ptr as usize);

        let new_ptr = ptr.map_addr(|a| a + 1);
        assert_eq!(new_ptr, ptr.wrapping_offset(1));

        let new_ptr = ptr.with_addr(3);
        assert_eq!(new_ptr, 3 as *mut u8);

        let mut x = 7u32;
        let x_ref = &mut x;
        let x_ptr = x_ref as *mut u32;
        let x_addr = x_ptr.expose_addr();
        let x_new_ptr = crate::from_exposed_addr_mut::<u32>(x_addr);

        unsafe {
            *x_new_ptr *= 3;
            *x_ptr *= 5;
            *x_ref *= 13;
            x *= 17;
        }

        assert_eq!(x, 7 * 3 * 5 * 13 * 17);
    }
}

#[cfg(feature = "uptr")]
pub mod int;
#[cfg(feature = "uptr")]
pub use self::int::iptr;
#[cfg(feature = "uptr")]
pub use self::int::uptr;

#[cfg(feature = "opaque_fn")]
pub mod func;
#[cfg(feature = "opaque_fn")]
pub use self::func::OpaqueFnPtr;
