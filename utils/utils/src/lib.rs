#![feature(test)]
#![feature(ptr_const_cast)]
#![feature(macro_metavar_expr)]

pub mod acc;
pub mod components;
pub mod container;
pub mod incrementable;
pub mod packed_map;
pub mod query;
pub mod soa;
pub mod compressed_vec2;
pub mod array_difference;

pub use components::Components;
pub use components_derive::Components;
pub use container::*;
pub use dioptre::Fields;
pub use incrementable::*;
pub use packed_map::*;
pub use query::*;
pub use soa::*;
pub use soak::{Columns, RawTable};
