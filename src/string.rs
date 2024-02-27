use crate::pool::{ GlobalPool, Pool, SlicesWrap };
use ::std::ops::{ Add, AddAssign, Bound, Deref, RangeBounds };
use ::std::string::{ self as std_string, String as StdString };
use ::std::str as std_str;

pub struct String<P: Pool = GlobalPool> {
	raw: P::Raw,
	pool: P
}

#[cfg(test)]
#[path = "./string_tests.rs"]
mod tests;

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

	pub fn to_other_pool<P2: Pool>(&self, pool: P2) -> String<P2> {
		let slice = self.pool.raw_to_slice(&self.raw);
		let raw = unsafe { pool.raw_from_slice(slice) };
		String { raw, pool }
	}

	pub fn into_other_pool<P2: Pool>(self, pool: P2) -> String<P2> {
		let vec = self.pool.raw_into_vec(self.raw);
		let raw = unsafe { pool.raw_from_vec(vec) };
		String { raw, pool }
	}

	pub fn clone_to<P2: Pool>(&self, pool: P2) -> String<P2> {
		self.to_other_pool(pool)
	}
}

// functions that take self
impl<P: Pool> String<P> {
	pub fn into_bytes(self) -> Vec<u8> {
		self.pool.raw_into_vec(self.raw)
	}

	pub fn as_str(&self) -> &str {
		unsafe { std_str::from_utf8_unchecked(self.as_bytes()) }
	}

	// skipping: as_mut_str

	pub fn push_str(&mut self, string: &str) {
		let new_raw = unsafe {
			self.pool.raw_from_slices(SlicesWrap(&[
				self.as_bytes(),
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
		let new_slice = &self.as_bytes()[..new_len];
		let new_raw = unsafe { self.pool.raw_from_slice(new_slice) };

		self.raw = new_raw;
	}

	pub fn pop(&mut self) -> Option<char> {
		let ch = self.chars().next_back()?;
		let new_len = self.len() - ch.len_utf8();

		let new_slice = &self.as_bytes()[..new_len];
		let new_raw = unsafe { self.pool.raw_from_slice(new_slice) };

		self.raw = new_raw;
		Some(ch)
	}

	pub fn remove(&mut self, i: usize) -> char {
		// let slice =
		let ch = self[i..].chars().next()
			.expect("cannot remove a char from the end of a string");
		let next = i + ch.len_utf8();

		let slice = self.as_bytes();
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

	pub fn insert(&mut self, i: usize, ch: char) {
		self.insert_str(i, ch.encode_utf8(&mut [0u8; 4]));
	}

	pub fn insert_str(&mut self, i: usize, string: &str) {
		assert!(self.is_char_boundary(i));
		let slice = self.as_bytes();

		let new_raw = unsafe {
			self.pool.raw_from_slices(SlicesWrap(&[
				&slice[..i],
				string.as_bytes(),
				&slice[i..]
			]))
		};

		self.raw = new_raw;
	}

	// skipping: as_mut_vec

	pub fn split_off(&mut self, at: usize) -> Self {
		self.split_off_in(at, self.pool.clone())
	}

	pub fn split_off_in<P2: Pool>(&mut self, at: usize, pool: P2) -> String<P2> {
		assert!(self.is_char_boundary(at));

		let self_raw = unsafe { self.pool.raw_from_slice(self[..at].as_bytes()) };
		let other_raw = unsafe { pool.raw_from_slice(self[at..].as_bytes()) };

		self.raw = self_raw;
		String { raw: other_raw, pool }
	}

	pub fn clear(&mut self) {
		self.raw = self.pool.raw_empty();
	}

	// skipping (for now): drain
	// skipping (for now): replace_range

	pub fn into_boxed_str(self) -> Box<str> {
		let boxed = self.pool.raw_into_boxed_slice(self.raw);
		unsafe { std_str::from_boxed_utf8_unchecked(boxed) }
	}

	pub fn leak<'h>(self) -> &'h mut str {
		let slice = self.pool.raw_into_vec(self.raw).leak();
		unsafe { std_str::from_utf8_unchecked_mut(slice) }
	}
}

impl<P: Pool> Add<&str> for String<P> {
	type Output = Self;
	fn add(mut self, rhs: &str) -> Self {
		// delegates to AddAssign<&str> for String<P>
		self += rhs;
		self
	}
}

impl<P: Pool> Add<&str> for &String<P> {
	type Output = String<P>;
	fn add(self, rhs: &str) -> String<P> {
		let raw = unsafe {
			self.pool.raw_from_slices(SlicesWrap(&[
				self.as_bytes(),
				rhs.as_bytes()
			]))
		};
		let pool = self.pool.clone();

		String { raw, pool }
	}
}

impl<P: Pool, P2: Pool> Add<String<P2>> for String<P> {
	type Output = Self;
	fn add(self, rhs: String<P2>) -> Self {
		// delegates to Add<&str> for String<P>
		self + &*rhs
	}
}

impl<P: Pool, P2: Pool> Add<String<P2>> for &String<P> {
	type Output = String<P>;
	fn add(self, rhs: String<P2>) -> String<P> {
		// delegates to Add<&str> for &String<P>
		self + &*rhs
	}
}

impl<P: Pool, P2: Pool> Add<&String<P2>> for String<P> {
	type Output = Self;
	fn add(self, rhs: &String<P2>) -> Self {
		// delegates to Add<&str> for String<P>
		self + &**rhs
	}
}

impl<P: Pool, P2: Pool> Add<&String<P2>> for &String<P> {
	type Output = String<P>;
	fn add(self, rhs: &String<P2>) -> String<P> {
		// delegates to Add<&str> for &String<P>
		self + &**rhs
	}
}


impl<P: Pool> AddAssign<&str> for String<P> {
	fn add_assign(&mut self, rhs: &str) {
		self.push_str(rhs);
	}
}

impl<P: Pool, P2: Pool> AddAssign<String<P2>> for String<P> {
	fn add_assign(&mut self, rhs: String<P2>) {
		// delegates to AddAssign<&str> for String<P>
		*self += &*rhs
	}
}

impl<P: Pool, P2: Pool> AddAssign<&String<P2>> for String<P> {
	fn add_assign(&mut self, rhs: &String<P2>) {
		// delegates to AddAssign<&str> for String<P>
		*self += &**rhs
	}
}

impl<P: Pool> Clone for String<P> {
	fn clone(&self) -> Self {
		let raw = self.pool.raw_clone(&self.raw);
		let pool = self.pool.clone();
		Self { raw, pool }
	}
}

impl<P: Pool> Deref for String<P> {
	type Target = str;
	fn deref(&self) -> &str {
		self.as_str()
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
