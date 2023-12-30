use crate::pool::{ self, RawString };
use std::alloc::{ alloc, Layout };
use std::borrow::Cow;
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::{ Range, RangeBounds };
use std::ptr;
use std::slice;
use std::str as std_str;
use std::string::{ self as std_string, String as StdString };
use std::sync::Arc;

pub struct String {
	arc: RawString
}

impl String {
	#[inline]
	pub fn new() -> Self {
		Self { arc: pool::from_str("") }
	}

	pub fn from_utf8(s: &[u8]) -> Result<Self, std_str::Utf8Error> {
		match std_str::from_utf8(s) {
			Ok(_) => {
				let arc = unsafe { pool::from_slice(s) };
				Ok(Self { arc })
			}
			Err(e) => { Err(e) }
		}
	}

	pub fn from_utf8_lossy(s: &[u8]) -> Self {
		let arc = match StdString::from_utf8_lossy(s) {
			Cow::Borrowed(s) => { pool::from_str(s) }
			Cow::Owned(s) => {
				// SAFETY: std String is guaranteed to be valid utf8
				unsafe { pool::from_slice(s.as_bytes()) }
			}
		};

		Self { arc }
	}

	pub fn from_utf16(v: &[u16]) -> Result<String, std_string::FromUtf16Error> {
		match StdString::from_utf16(v) {
			Ok(s) => {
				let arc = pool::from_str(&s);
				Ok(Self { arc })
			}
			Err(e) => { Err(e) }
		}
	}

	pub fn from_utf16_lossy(v: &[u16]) -> String {
		let arc = pool::from_str(&StdString::from_utf16_lossy(v));
		Self { arc }
	}

	#[inline]
	pub unsafe fn from_utf8_unchecked(bytes: &[u8]) -> String {
		let arc = pool::from_slice(bytes);
		Self { arc }
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		// TODO: Deref impl like std String
		// SAFETY: strings in string pool guaranteed to be valid utf8
		unsafe { std_str::from_utf8_unchecked(&self.arc) }
	}

	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		&self.arc
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.arc.len()
	}

	#[inline]
	pub fn is_empty(&self) -> bool {
		self.arc.is_empty()
	}

	#[must_use = "e"]
	pub fn truncate(&self, new_len: usize) -> Self {
		if self.len() > new_len {
			assert!(self.as_str().is_char_boundary(new_len));

			let layout = Layout::array::<u8>(new_len).unwrap();
			let new_ptr = unsafe { alloc(layout) };
			unsafe { new_ptr.copy_from_nonoverlapping(self.arc.as_ptr(), new_len) };

			let s = unsafe { &*ptr::slice_from_raw_parts_mut(new_ptr, new_len) };
			let arc = unsafe { pool::from_slice(s) };
			Self { arc }
		} else {
			self.clone()
		}
	}
}

impl Clone for String {
	fn clone(&self) -> Self {
		Self { arc: Arc::clone(&self.arc) }
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
