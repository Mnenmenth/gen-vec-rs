//! Uses exposed/independent index allocator
//!
//! # Examples
//!
//! ```
//! use gen_vec::Index;
//! use gen_vec::exposed::{IndexAllocator, ExposedGenVec};
//!
//! let mut allocator: IndexAllocator = IndexAllocator::new();
//!
//! let index: Index = allocator.allocate();
//!
//! let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
//! vec.set(index, 5);
//! assert!(vec.contains(index));
//!
//! let value: Option<&i32> = vec.get(index);
//! assert_eq!(value, Some(&5));
//! ```

mod gen_vec;
pub use self::gen_vec::*;
mod index_allocator;
pub use self::index_allocator::*;