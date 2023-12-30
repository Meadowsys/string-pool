use crate::pool::{ self, RawString };
use std::alloc::{ alloc, Layout };
use std::borrow::Cow;
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::{ Deref, DerefMut, Index, IndexMut };
use std::ops::{ Range, RangeFrom, RangeTo, RangeFull, RangeInclusive, RangeToInclusive };
use std::ptr;
use std::slice;
use std::str as std_str;
use std::string::{ self as std_string, String as StdString };
use std::sync::Arc;

pub struct String {
	raw: RawString
}

impl String {
	#[inline]
	pub fn empty() -> Self {
		Self::from_str("")
	}

	#[inline]
	#[allow(clippy::should_implement_trait)]
	pub fn from_str(s: &str) -> Self {
		Self { raw: pool::from_str(s) }
	}

	pub fn from_utf8(s: &[u8]) -> Result<Self, std_str::Utf8Error> {
		match std_str::from_utf8(s) {
			Ok(_) => {
				let raw = unsafe { pool::from_slice(s) };
				Ok(Self { raw })
			}
			Err(e) => { Err(e) }
		}
	}

	pub fn from_utf8_lossy(s: &[u8]) -> Self {
		let raw = match StdString::from_utf8_lossy(s) {
			Cow::Borrowed(s) => { pool::from_str(s) }
			Cow::Owned(s) => { pool::from_str(&s) }
		};

		Self { raw }
	}

	pub fn from_utf16(v: &[u16]) -> Result<String, std_string::FromUtf16Error> {
		match StdString::from_utf16(v) {
			Ok(s) => {
				let raw = pool::from_str(&s);
				Ok(Self { raw })
			}
			Err(e) => { Err(e) }
		}
	}

	pub fn from_utf16_lossy(v: &[u16]) -> String {
		let raw = pool::from_str(&StdString::from_utf16_lossy(v));
		Self { raw }
	}

	#[inline]
	pub unsafe fn from_utf8_unchecked(bytes: &[u8]) -> String {
		let raw = pool::from_slice(bytes);
		Self { raw }
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		self
	}

	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		&self.raw
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.raw.len()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.raw.is_empty()
	}

	#[must_use]
	pub fn truncate(&self, new_len: usize) -> Self {
		if self.len() > new_len {
			assert!(self.as_str().is_char_boundary(new_len));
			let raw = unsafe { pool::from_slice(&self.raw[..new_len]) };
			Self { raw }
		} else {
			self.clone()
		}
	}

	#[must_use]
	pub fn pop(&self) -> Option<(char, String)> {
		let ch = self.chars().next_back()?;
		let new_len = self.len() - ch.len_utf8();

		let layout = Layout::array::<u8>(new_len).unwrap();
		let new_ptr = unsafe { alloc(layout) };
		unsafe { new_ptr.copy_from_nonoverlapping(self.raw.as_ptr(), new_len) };

		let s = unsafe { &*ptr::slice_from_raw_parts_mut(new_ptr, new_len) };
		let raw = unsafe { pool::from_slice(s) };
		Some((ch, Self { raw }))
	}

	#[must_use]
	pub fn remove(&self, i: usize) -> (char, String) {
		let ch = self[i..].chars().next().unwrap();

		let next = i + ch.len_utf8();
		let len = self.len();

		let new_len = len - ch.len_utf8();
		let layout = Layout::array::<u8>(new_len).unwrap();
		let new_ptr = unsafe { alloc(layout) };
		unsafe {
			new_ptr.copy_from_nonoverlapping(self.raw.as_ptr(), i);
			new_ptr.copy_from_nonoverlapping(self.raw.as_ptr().add(next), new_len - i);
		}

		let s = unsafe { &*ptr::slice_from_raw_parts(new_ptr, new_len) };
		let raw = unsafe { pool::from_slice(s) };
		(ch, Self { raw })
	}
}

impl Deref for String {
	type Target = str;
	#[inline]
	fn deref(&self) -> &str {
		// SAFETY: strings in string pool guaranteed to be valid utf8
		unsafe { std_str::from_utf8_unchecked(&self.raw) }
	}
}

impl Index<Range<usize>> for String {
	type Output = str;

	#[inline]
	fn index(&self, index: Range<usize>) -> &str {
		&self[..][index]
	}
}

impl Index<RangeFrom<usize>> for String {
	type Output = str;

	#[inline]
	fn index(&self, index: RangeFrom<usize>) -> &str {
		&self[..][index]
	}
}

impl Index<RangeTo<usize>> for String {
	type Output = str;

	#[inline]
	fn index(&self, index: RangeTo<usize>) -> &str {
		&self[..][index]
	}
}

impl Index<RangeFull> for String {
	type Output = str;

	#[inline]
	fn index(&self, index: RangeFull) -> &str {
		self
	}
}

impl Index<RangeInclusive<usize>> for String {
	type Output = str;

	#[inline]
	fn index(&self, index: RangeInclusive<usize>) -> &str {
		&self[..][index]
	}
}

impl Index<RangeToInclusive<usize>> for String {
	type Output = str;

	#[inline]
	fn index(&self, index: RangeToInclusive<usize>) -> &str {
		&self[..][index]
	}
}


impl From<&str> for String {
	#[inline]
	fn from(value: &str) -> Self {
		Self::from_str(value)
	}
}

impl From<StdString> for String {
	#[inline]
	fn from(value: StdString) -> Self {
		Self::from_str(&value)
	}
}

impl Clone for String {
	#[inline]
	fn clone(&self) -> Self {
		Self { raw: Arc::clone(&self.raw) }
	}
}

impl fmt::Debug for String {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self.as_str(), f)
	}
}

impl fmt::Display for String {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(self.as_str(), f)
	}
}
