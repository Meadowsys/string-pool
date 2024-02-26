use crate::Pool;
use crate::pool::Global;

pub struct String<P: Pool = Global> {
	raw: P::Raw,
	pool: P
}

impl String {
	pub fn new() -> Self {
		Self::new_in(Global)
	}
}

impl<P: Pool> String<P> {
	pub fn new_in(pool: P) -> Self {
		let raw = pool.raw_from_str("");
		Self { raw, pool }
	}
}
