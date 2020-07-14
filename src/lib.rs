//! Vector of reusable, generational indices that owns the contained values
//!
//! [Inspired by Catherine West's closing keynote at RustConf 2018](https://kyren.github.io/2018/09/14/rustconf-talk.html)
//!
//! ## Closed vs. Exposed index allocation implementations
//!
//! `ClosedGenVec` uses a non user-accessible index allocator to manage `Index`s
//!
//! `ExposedGenVec` relies on an external `IndexAllocator`
//!
//! As such, an `IndexAllocator` must be created and used to allocate/deallocate `Index`s manually.
//! This is useful for using the same `Index` across multiple `ExposedGenerationalVec`s
//!
//! **Note:** `IndexAllocator` cannot be used with the self-allocating `ClosedGenerationalVec` since
//! it manages its own `Index`s
//!
//! ## Explanation of Generational Indices
//!
//! `Index` structs are used to access the vector's contents. An `Index` contains an index for the vector
//! and a generation (which is 0 initially).
//!
//! Deallocated/removed `Index`s go into a list of free `Index`s that can be reused
//!
//! Every time an `Index` is reused, the internal generation is incremented. This ensures that a deallocated
//! `Index` handle can't access data that it no longer validly points to

/// An index of a generational vec
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct Index
{
    index: usize,
    generation: usize
}

/// An item within a generational vec
#[derive(Debug)]
struct Item<T>
{
    value: T,
    generation: usize
}

pub mod closed;
pub mod exposed;