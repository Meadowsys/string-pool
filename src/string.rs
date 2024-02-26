use crate::Pool;
use crate::pool::Global;
use ::std::string::String as StdString;

pub struct String<P: Pool = Global> {
	raw: P::Raw,
	pool: P
}

impl String {
	pub fn new() -> Self {
		Self::new_in(Global)
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self::with_capacity_in(capacity, Global)
	}
}

impl<P: Pool> String<P> {
	pub fn new_in(pool: P) -> Self {
		let raw = pool.raw_from_str("");
		Self { raw, pool }
	}

	pub fn with_capacity_in(capacity: usize, pool: P) -> Self {
		let raw = pool.raw_empty_with_capacity(capacity);
		Self { raw, pool }
	}
}
