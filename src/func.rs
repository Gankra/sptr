//! Tools for making it easier to use function pointers.

/// The `void*` equivalent for a function pointer, for when you need to handle "some fn".
///
/// Some platforms (WASM, AVR) have non-uniform representations for "code" and "data" pointers.
/// Rust
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct OpaqueFnPtr(fn() -> ());

impl OpaqueFnPtr {
    /// Create an OpaqueFnPtr from some `fn`.
    ///
    /// Rust doesn't have a good way to express, so this just takes "anything" and it's
    /// up to you to make sure you're actually feeding in a function pointer.
    ///
    /// **If you feed in anything else, it is Undefined Behaviour.**
    #[inline]
    #[must_use]
    pub unsafe fn from_fn<T>(func: T) -> Self {
        assert_eq!(
            core::mem::size_of::<T>(),
            core::mem::size_of::<OpaqueFnPtr>()
        );
        assert_eq!(
            core::mem::align_of::<T>(),
            core::mem::align_of::<OpaqueFnPtr>()
        );

        Self(core::mem::transmute_copy(&func))
    }

    /// Create a `fn` from an OpaqueFnPtr.
    ///
    /// Rust doesn't have a good way to express, so this just takes "anything" and it's
    /// up to you to make sure you're actually feeding in a function pointer type.
    ///
    /// **If you feed in anything else, it is Undefined Behaviour.**
    #[inline]
    #[must_use]
    pub unsafe fn to_fn<T>(self) -> T {
        assert_eq!(
            core::mem::size_of::<T>(),
            core::mem::size_of::<OpaqueFnPtr>()
        );
        assert_eq!(
            core::mem::align_of::<T>(),
            core::mem::align_of::<OpaqueFnPtr>()
        );

        core::mem::transmute_copy(&self.0)
    }

    /// Get the address of the function pointer.
    ///
    /// Note that while you *can* compare this to a data pointer, the result will
    /// almost certainly be meaningless, especially on platforms like WASM and AVR
    /// where function pointers are in a separate address-space from data pointers.
    ///
    /// See [`pointer::addr`][crate::Strict::addr] for details.
    #[inline]
    #[must_use]
    pub fn addr(self) -> usize {
        self.0 as usize
    }
}
