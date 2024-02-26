use crate::Pool;
use crate::pool::GlobalPool;
use ::std::string::{ self as std_string, String as StdString };
use ::std::str as std_str;

pub struct String<P: Pool = GlobalPool> {
	raw: P::Raw,
	pool: P
}

// constructors with default pool
impl String {
	pub fn new() -> Self {
		Self::new_in(GlobalPool)
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self::with_capacity_in(capacity, GlobalPool)
	}

	pub fn from_utf8(vec: Vec<u8>) -> Result<Self, std_string::FromUtf8Error> {
		Self::from_utf8_in(vec, GlobalPool)
	}

	pub fn from_utf8_slice(slice: &[u8]) -> Result<Self, std_str::Utf8Error> {
		Self::from_utf8_slice_in(slice, GlobalPool)
	}

	pub fn from_utf8_lossy(v: &[u8]) -> Self {
		Self::from_utf8_lossy_in(v, GlobalPool)
	}

	pub fn from_utf16(v: &[u16]) -> Result<Self, std_string::FromUtf16Error> {
		Self::from_utf16_in(v, GlobalPool)
	}

	pub fn from_utf16_lossy(v: &[u16]) -> Self {
		Self::from_utf16_lossy_in(v, GlobalPool)
	}

	// skipping nightly apis for now:
	//    from_utf16le
	//    from_utf16le_lossy
	//    from_utf16be
	//    from_utf16be_lossy
	//    into_raw_parts

	// skipping apis for now:
	//    from_raw_parts

	pub unsafe fn from_utf8_unchecked(bytes: Vec<u8>) -> Self {
		Self::from_utf8_unchecked_in(bytes, GlobalPool)
	}

	pub unsafe fn from_utf8_unchecked_slice(slice: &[u8]) -> Self {
		Self::from_utf8_unchecked_slice_in(slice, GlobalPool)
	}
}

// constructors with custom pool
impl<P: Pool> String<P> {
	pub fn new_in(pool: P) -> Self {
		let raw = pool.raw_from_str("");
		Self { raw, pool }
	}

	pub fn with_capacity_in(capacity: usize, pool: P) -> Self {
		let raw = pool.raw_empty_with_capacity(capacity);
		Self { raw, pool }
	}

	pub fn from_utf8_in(vec: Vec<u8>, pool: P) -> Result<Self, std_string::FromUtf8Error> {
		// running it through std String because it gives us FromUtf8Error, for
		// compat with std String's from_utf8 function, don't think there is
		// any other way to get it than through this
		let std_string = StdString::from_utf8(vec)?;
		let vec = std_string.into_bytes();
		let raw = unsafe { pool.raw_from_vec(vec) };
		Ok(Self { raw, pool })
	}

	pub fn from_utf8_slice_in(slice: &[u8], pool: P) -> Result<Self, std_str::Utf8Error> {
		let s = std_str::from_utf8(slice)?;
		let raw = pool.raw_from_str(s);
		Ok(Self { raw, pool })
	}

	pub fn from_utf8_lossy_in(v: &[u8], pool: P) -> Self {
		let s = StdString::from_utf8_lossy(v);
		let raw = pool.raw_from_str(&s);
		Self { raw, pool }
	}

	pub fn from_utf16_in(v: &[u16], pool: P) -> Result<Self, std_string::FromUtf16Error> {
		let s = StdString::from_utf16(v)?;
		let raw = pool.raw_from_str(&s);
		Ok(Self { raw, pool })
	}

	pub fn from_utf16_lossy_in(v: &[u16], pool: P) -> Self {
		let s = StdString::from_utf16_lossy(v);
		let raw = pool.raw_from_str(&s);
		Self { raw, pool }
	}

	pub unsafe fn from_utf8_unchecked_in(bytes: Vec<u8>, pool: P) -> Self {
		let raw = pool.raw_from_vec(bytes);
		Self { raw, pool }
	}

	pub unsafe fn from_utf8_unchecked_slice_in(slice: &[u8], pool: P) -> Self {
		let raw = pool.raw_from_slice(slice);
		Self { raw, pool }
	}
}

impl From<&str> for String {
	fn from(s: &str) -> Self {
		Self::from((s, GlobalPool))
	}
}

impl<P: Pool> From<(&str, P)> for String<P> {
	fn from((s, pool): (&str, P)) -> Self {
		let raw = pool.raw_from_str(s);
		Self { raw, pool }
	}
}
