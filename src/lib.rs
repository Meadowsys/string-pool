// TODO: remove when more finished
#![allow(dead_code, unused_imports, unused_variables)]
#![allow(clippy::missing_safety_doc)]

#![allow(clippy::new_without_default)]

pub mod pool;
mod string;

pub use self::string::String;
pub use self::pool::Pool;
