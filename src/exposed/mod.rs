//! Uses exposed/independent index allocator
//!
//! # Examples
//!
//! ```
//! use gen_vec::Index;
//! use gen_vec::exposed::{IndexAllocator, GenerationalVec};
//!
//! let mut allocator: IndexAllocator = IndexAllocator::new();
//!
//! let index: Index = allocator.allocate();
//!
//! let mut vec: GenerationalVec<i32> = GenerationalVec::new();
//! vec.set(index, 5);
//!
//! let value: &i32 = vec.get(index).unwrap();
//! assert_eq!(*value, 5);
//! ```

mod gen_vec;
pub use self::gen_vec::*;
mod index_allocator;
pub use self::index_allocator::*;