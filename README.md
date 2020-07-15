 # gen-vec [![Build Status](https://travis-ci.org/Mnenmenth/gen-vec-rs.svg?branch=master)](https://travis-ci.org/Mnenmenth/gen-vec-rs)
 
 Vector of reusable, generational indices that owns its values

 [Inspired by Catherine West's closing keynote at RustConf 2018](https://kyren.github.io/2018/09/14/rustconf-talk.html)

 ### Closed vs. Exposed index allocation implementations

 `ClosedGenVec` uses a non user-accessible index allocator to manage indices

 `ExposedGenVec` relies on an external `IndexAllocator`

 As such, an `IndexAllocator` must be created and used to allocate/deallocate indices manually.
 This is useful for using the same `Index` across multiple `ExposedGenerationalVec` instances

 **Note:** `IndexAllocator` cannot be used with `ClosedGenerationalVec` since it has its own 
 internal `IndexAllocator`

 ### Explanation of Generational Indices

 `Index` structs are used to access the vector's contents. An `Index` contains an index for the vector
 and a generation (which is 0 initially).

 Deallocated/removed `Index`s go into a list of free `Index`s that can be reused

 Every time an `Index` is reused, the internal generation is incremented. This ensures that a deallocated
 `Index` handle can't access data that it no longer validly points to
 
 ### Usage
 Add `gen-vec` to your `Cargo.toml`
```toml
[dependencies]
gen-vec = "0.2.0"
```
Using the self-allocating `ClosedGenVec`
```rust
use gen_vec::Index;
use gen_vec::closed::ClosedGenVec;

let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();

let index: Index = vec.insert(42);
assert!(vec.contains(index));

let value: Option<&i32> = vec.get(index);
assert_eq!(value, Some(&42));

vec.remove(index);
assert!(!vec.contains(index));
```

Using `ExposedGenVec` with `IndexAllocator`
```rust
use gen_vec::Index;
use gen_vec::exposed::{IndexAllocator, ExposedGenVec};

let mut allocator: IndexAllocator = IndexAllocator::new();

let index: Index = allocator.allocate();

let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
vec.set(index, 5);
assert!(vec.contains(index));

let value: Option<&i32> = vec.get(index);
assert_eq!(value, Some(&5));
```
