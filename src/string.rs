use crate::pool::{ DefaultPool, PoolAccess };
use ::std::str;
use ::std::ops::Deref;

pub struct String<P: PoolAccess = DefaultPool> {
	raw: P::RawString
}

impl<P: PoolAccess> Deref for String<P> {
	type Target = str;
	#[inline]
	fn deref(&self) -> &Self::Target {
		let slice = P::deref_raw_to_slice(&self.raw);
		// SAFETY: Strings are always UTF-8
		unsafe { str::from_utf8_unchecked(slice) }
	}
}
