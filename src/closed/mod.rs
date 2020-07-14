//! Uses closed/non user-accessible index allocator
//!
//! # Examples
//!
//! ```
//! use gen_vec::Index;
//! use gen_vec::closed::GenerationalVec;
//!
//! let mut vec: GenerationalVec<i32> = GenerationalVec::new();
//!
//! let index: Index = vec.insert(42);
//! assert!(vec.contains(index));
//!
//! vec.remove(index);
//! assert!(!vec.contains(index));
//! ```

mod gen_vec;
pub use self::gen_vec::*;