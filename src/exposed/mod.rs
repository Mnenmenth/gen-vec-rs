//! Uses exposed/independent index allocator
//!
//! # Example
//!
//! ```
//! let allocator: IndexAllocator = IndexAllocator::new();
//!
//! let index: Index = allocator.allocate();
//!
//! let mut vec: GenerationalVec<i32> = GenerationalVec::new();
//! vec.set(index, 5);
//!
//! let value: &i32 = vec.get(index);
//! assert_eq!(*value, 5);
//! ```

mod gen_vec;
pub use gen_vec::*;
mod index_allocator;
pub use index_allocator::*;