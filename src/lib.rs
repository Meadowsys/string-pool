#![doc = include_str!("../README.md")]

// TODO: remove when more finished
#![allow(dead_code, unused_imports, unused_variables)]
#![allow(clippy::missing_safety_doc)]

#![allow(clippy::new_without_default)]

pub mod pool;
pub mod string;

#[doc(inline)]
pub use crate::string::String;
#[doc(inline)]
pub use crate::pool::{ Pool, GlobalPool };
