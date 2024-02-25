pub mod default;
pub mod no;

pub trait PoolProvider {
	/// Raw type of string that this pool manages. Ideally provided instances
	/// of these should point  to the same memory backing store, which is the
	/// whole point of the pool (one global instance per string).
	type RawString;

	/// Provides a [`RawString`][`PoolAccess::RawString`] from a slice
	///
	/// # Safety
	///
	/// Caller must ensure that the slice provided contains valid UTF-8
	unsafe fn from_slice(slice: &[u8]) -> Self::RawString;

	/// Takes a [`RawString`][`PoolAccess::RawString`] and derefs it to `&[u8]`
	/// This is more flexible than requiring `RawString: Deref<[u8]>`
	fn deref_raw_to_slice(raw: &Self::RawString) -> &[u8];

	/// Clones the raw string provided. This should just be
	/// making a copy of the reference.
	fn clone_raw(raw: &Self::RawString) -> Self::RawString;

	// --- optional methods ---

	/// Called when an instance of [`RawString`][`PoolAccess::RawString`]
	/// is being dropped. This gives the pool opportunity to clean up if needed.
	/// Default impl is a noop
	fn dropping_instance_of(slice: &[u8]) {
		let _ = slice;
	}

	/// Instructs the pool to preallocate capacity, for optimisation reasons only.
	/// There is no provided guarantee of such, and should not be expected to
	/// do anything after first access of the pool, usually when the first
	/// [`String`][`super::String`] is created.
	/// Default impl is a noop
	#[inline]
	fn preallocate_capacity(capacity: usize) {
		let _ = capacity;
	}

	/// Provides a [`RawString`][`PoolAccess::RawString`] from a `&str`.
	/// Default impl just calls [`from_slice`][`PoolAccess::from_slice`] with
	/// `str.as_bytes()`
	#[inline]
	fn from_str(s: &str) -> Self::RawString {
		unsafe { Self::from_slice(s.as_bytes()) }
	}
}
