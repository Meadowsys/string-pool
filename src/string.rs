use crate::pool::{ GlobalPool, Pool, SlicesWrap };
use ::std::ops::Deref;
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

	// skipping (nightly, for now): from_utf16le
	// skipping (nightly, for now): from_utf16le_lossy
	// skipping (nightly, for now): from_utf16be
	// skipping (nightly, for now): from_utf16be_lossy
	// skipping (nightly): into_raw_parts
	// skipping: from_raw_parts

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
		let raw = pool.raw_empty();
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
		let raw = unsafe { pool.raw_from_slice(s.as_bytes()) };
		Ok(Self { raw, pool })
	}

	pub fn from_utf8_lossy_in(v: &[u8], pool: P) -> Self {
		let s = StdString::from_utf8_lossy(v);
		let raw = unsafe { pool.raw_from_slice(s.as_bytes()) };
		Self { raw, pool }
	}

	pub fn from_utf16_in(v: &[u16], pool: P) -> Result<Self, std_string::FromUtf16Error> {
		let s = StdString::from_utf16(v)?;
		let raw = unsafe { pool.raw_from_slice(s.as_bytes()) };
		Ok(Self { raw, pool })
	}

	pub fn from_utf16_lossy_in(v: &[u16], pool: P) -> Self {
		let s = StdString::from_utf16_lossy(v);
		let raw = unsafe { pool.raw_from_slice(s.as_bytes()) };
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

// functions that take self
impl<P: Pool> String<P> {
	pub fn into_bytes(self) -> Vec<u8> {
		self.pool.raw_into_vec(self.raw)
	}

	pub fn as_str(&self) -> &str {
		let slice = self.pool.raw_to_slice(&self.raw);
		unsafe { std_str::from_utf8_unchecked(slice) }
	}

	// skipping: as_mut_str

	pub fn push_str(&mut self, string: &str) {
		let new_raw = unsafe {
			self.pool.raw_from_slices(SlicesWrap(&[
				self.pool.raw_to_slice(&self.raw),
				string.as_bytes()
			]))
		};

		self.raw = new_raw;
	}

	// skipping (nightly, for now): extend_from_within
	// skipping: capacity
	// skipping: reserve
	// skipping: reserve_exact
	// skipping: try_reserve
	// skipping: try_reserve_exact
	// skipping: shrink_to_fit
	// skipping: shrink_to

	pub fn push(&mut self, ch: char) {
		self.push_str(ch.encode_utf8(&mut [0u8; 4]));
	}

	pub fn as_bytes(&self) -> &[u8] {
		self.pool.raw_to_slice(&self.raw)
	}

	pub fn truncate(&mut self, new_len: usize) {
		if new_len > self.len() { return }

		assert!(self.is_char_boundary(new_len));
		let new_slice = &self.pool.raw_to_slice(&self.raw)[..new_len];
		let new_raw = unsafe { self.pool.raw_from_slice(new_slice) };

		self.raw = new_raw;
	}

	pub fn pop(&mut self) -> Option<char> {
		let ch = self.chars().next_back()?;
		let new_len = self.len() - ch.len_utf8();

		let new_slice = &self.pool.raw_to_slice(&self.raw)[..new_len];
		let new_raw = unsafe { self.pool.raw_from_slice(new_slice) };

		self.raw = new_raw;
		Some(ch)
	}

	pub fn remove(&mut self, i: usize) -> char {
		// let slice =
		let ch = self[i..].chars().next()
			.expect("cannot remove a char from the end of a string");
		let next = i + ch.len_utf8();

		let slice = self.pool.raw_to_slice(&self.raw);
		let new_raw = unsafe {
			self.pool.raw_from_slices(SlicesWrap(&[
				&slice[..i],
				&slice[next..]
			]))
		};

		self.raw = new_raw;
		ch
	}

	// skipping (nightly, for now): remove_matches

	pub fn retain<F>(&mut self, mut f: F)
	where
		F: FnMut(char) -> bool
	{
		// reason for capacity:
		// worst case is true, false, true, false, etc
		// which is keeping half of 1 byte chars
		// so if chars are longer or longer sequences of no switching, it'll be less
		// +1 is for odd ones, like. true false true, len 3, (3 / 2) + 1 == 2, will fit
		// i mean, worst case is worse performance, it doesn't lead to correctness
		// issues, so its not the most critical :p
		let mut retained = Vec::with_capacity((self.len() / 2) + 1);
		let mut state = None;

		for (i, char) in self.char_indices() {
			match (f(char), state) {
				// start new streak
				(true, None) => {
					state = Some(i);
				}

				// end streak
				(false, Some(i_start)) => {
					retained.push(self[i_start..i].as_bytes());
					state = None;
				}

				// continue true streak
				(true, Some(_)) => { /* noop */ }

				// continue false streak
				(false, None) => { /* noop */ }
			}
		}

		if let Some(i_start) = state {
			retained.push(self[i_start..].as_bytes());
		}

		let new_raw = unsafe { self.pool.raw_from_slices(SlicesWrap(&retained)) };
		self.raw = new_raw;
	}
}

impl From<&str> for String {
	fn from(s: &str) -> Self {
		Self::from((s, GlobalPool))
	}
}

impl<P: Pool> From<(&str, P)> for String<P> {
	fn from((s, pool): (&str, P)) -> Self {
		let raw = unsafe { pool.raw_from_slice(s.as_bytes()) };
		Self { raw, pool }
	}
}

impl<P: Pool> Deref for String<P> {
	type Target = str;
	fn deref(&self) -> &str {
		self.as_str()
	}
}
