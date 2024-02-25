use super::PoolProvider;

/// Not actually string pool string
pub struct NoPool;

impl PoolProvider for NoPool {
	type RawString = String;

	unsafe fn from_slice(slice: &[u8]) -> Self::RawString {
		unsafe { String::from_utf8_unchecked(slice.to_vec()) }
	}

	fn deref_raw_to_slice(raw: &Self::RawString) -> &[u8] {
		raw.as_bytes()
	}

	fn clone_raw(raw: &Self::RawString) -> Self::RawString {
		raw.clone()
	}
}
