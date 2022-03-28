# sptr: The Strict Provenance Polyfill

This library contains a stable polyfill for the experimental [strict provenance](https://github.com/rust-lang/rust/issues/95228) APIs in Rust's libcore:

```
// core::ptr
pub fn invalid<T>(addr: usize) -> *const T;
pub fn invalid_mut<T>(addr: usize) -> *mut T;

// core::pointer
pub fn addr(self) -> usize;
pub fn with_addr(self, addr: usize) -> Self;
pub fn map_addr(self, f: impl FnOnce(usize) -> usize) -> Self;
```

