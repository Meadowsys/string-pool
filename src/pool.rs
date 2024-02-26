use ::std::hash::{ Hash, Hasher };
use ::hashbrown::HashSet;

mod global;

pub use global::Global;

pub trait Pool {
	type Raw;

	/// # Safety
	///
	/// The provided slices, when joined together using
	/// [`SlicesWrap::to_boxed_slice`], must be valid UTF-8
	unsafe fn raw_from_slices(&self, slices: SlicesWrap) -> Self::Raw;

	// --- provided functions ---

	/// note to implementors: The default implementation
	/// of this method is usually enough.
	unsafe fn raw_from_slice(&self, slice: &[u8]) -> Self::Raw {
		self.raw_from_slices(SlicesWrap(&[slice]))
	}

	/// note to implementors: The default implementation
	/// of this method is usually enough.
	fn raw_from_str(&self, s: &str) -> Self::Raw {
		unsafe { self.raw_from_slice(s.as_bytes()) }
	}
}

#[repr(transparent)]
pub struct SlicesWrap<'h>(pub &'h [&'h [u8]]);

impl<'h> SlicesWrap<'h> {
	pub fn to_boxed_slice(&self) -> Box<[u8]> {
		self.into_iter()
			.collect::<Vec<_>>()
			.into_boxed_slice()
	}
}

impl<'h> Hash for SlicesWrap<'h> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.into_iter().for_each(|b| state.write_u8(b));
	}
}

impl<'h> IntoIterator for &SlicesWrap<'h> {
	type Item = <SlicesWrapIter<'h> as Iterator>::Item;
	type IntoIter = SlicesWrapIter<'h>;

	fn into_iter(self) -> Self::IntoIter {
		let mut vec = Vec::with_capacity(self.0.len());
		self.0.iter().rev().for_each(|slice| vec.push(*slice));
		SlicesWrapIter(vec)
	}
}

pub struct SlicesWrapIter<'h>(Vec<&'h [u8]>);

impl<'h> Iterator for SlicesWrapIter<'h> {
	type Item = u8;

	fn next(&mut self) -> Option<u8> {
		// if this is none
		// we will have reached the end of the vec
		let next_slice = self.0.pop()?;

		match next_slice.iter().next() {
			Some(item) => {
				self.0.push(&next_slice[1..]);
				Some(*item)
			}
			None => {
				// we popped so this will start on the slice after current
				self.next()
			}
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.0.iter().map(|s| s.len()).sum();
		(len, Some(len))
	}
}
