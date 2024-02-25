use crate::pool::{ DefaultPool, PoolAccess };
use ::std::{ str as std_str, string as std_string };
use ::std::string::String as StdString;
use ::std::ops::Deref;

#[must_use = "String pool instances are immutable; operations that would mutate std String, actually return a new instance here"]
pub struct String<P: PoolAccess = DefaultPool> {
	raw: P::RawString
}

/// call associated functions on this when using the default pool, because for
/// some reason, doing something like `String::empty()` requires `P` to be
/// specified, which  is weird to me???
pub type StringDefaultPool = String;

impl<P: PoolAccess> String<P> {
	#[inline]
	pub fn empty() -> Self {
		Self::from_str("")
	}

	#[allow(clippy::should_implement_trait)]
	#[inline]
	pub fn from_str(s: &str) -> Self {
		let raw = P::from_str(s);
		Self { raw }
	}

	#[inline]
	pub fn from_utf8(bytes: &[u8]) -> Result<Self, std_str::Utf8Error> {
		Ok(Self::from_str(std_str::from_utf8(bytes)?))
	}

	#[inline]
	pub fn from_utf8_lossy(bytes: &[u8]) -> Self {
		Self::from_str(&StdString::from_utf8_lossy(bytes))
	}

	#[inline]
	pub fn from_utf16(bytes: &[u16]) -> Result<Self, std_string::FromUtf16Error> {
		Ok(Self::from_str(&StdString::from_utf16(bytes)?))
	}

	#[inline]
	pub fn from_utf16_lossy(bytes: &[u16]) -> Self {
		Self::from_str(&StdString::from_utf16_lossy(bytes))
	}

	// TODO unstable stuff: from_utf16le, from_utf16le_lossy, from_utf16be, from_utf16be_lossy
	// skip from_raw_parts?

	#[inline]
	pub unsafe fn from_utf8_unchecked(bytes: &[u8]) -> Self {
		let raw = P::from_slice(bytes);
		Self { raw }
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		let slice = P::deref_raw_to_slice(&self.raw);
		unsafe { std_str::from_utf8_unchecked(slice) }
	}

	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		P::deref_raw_to_slice(&self.raw)
	}

	pub fn to_std(&self) -> StdString {
		StdString::from(&**self)
	}

	#[inline]
	pub fn push_str(&self, s: &str) -> Self {
		// maybe theres a more efficient way to do this?
		let mut std_string = StdString::with_capacity(self.len() + s.len());
		std_string.push_str(self);
		std_string.push_str(s);
		Self::from_str(&std_string)
	}

	// TODO unstable stuff: extend_from_within

	// skipped: capacity, reserve, reserve_exact, try_reserve, try_reserve_exact,
	// shrink_to_fit, shrink_to

	#[inline]
	pub fn push(&self, c: char) -> Self {
		let mut std_string = StdString::with_capacity(self.len() + c.len_utf8());
		std_string.push_str(self);
		std_string.push(c);
		Self::from_str(&std_string)
	}

	pub fn truncate(&self, new_len: usize) -> Self {
		let s = if new_len <= self.len() {
			assert!(self.is_char_boundary(new_len));
			&self[..new_len]
		} else {
			self
		};

		Self::from_str(s)
	}

	pub fn pop(&self) -> Option<(char, Self)> {
		let c = self.chars().next_back()?;
		let new_len = self.len() - c.len_utf8();
		Some((c, Self::from_str(&self[..new_len])))
	}

	pub fn remove(&self, i: usize) -> (char, Self) {
		let mut std_string = self.to_std();
		let c = std_string.remove(i);
		(c, Self::from_str(&std_string))
	}

	// TODO unstable stuff: remove_matches
}

impl<P: PoolAccess> Deref for String<P> {
	type Target = str;
	#[inline]
	fn deref(&self) -> &str {
		self.as_str()
	}
}
