use ::hashbrown::{ Equivalent, HashSet };
use ::lazy_wrap::LazyWrap;
use ::parking_lot::RwLock;
use ::std::sync::Arc;
use ::std::marker::PhantomData;
use ::std::ops::Deref;
use ::std::sync::atomic::{ AtomicUsize, Ordering::SeqCst };

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

	/// Called when an instance of [`RawString`][`PoolAccess::RawString`]
	/// is being dropped. This gives the pool opportunity to clean up if needed
	fn dropping_instance_of(slice: &[u8]);

	// --- optional methods ---

	/// Instructs the pool to preallocate capacity, for optimisation reasons only.
	/// There is no provided guarantee of such, and should not be expected to
	/// do anything after first access of the pool, usually when the first
	/// [`String`][`super::String`] is created.
	/// Default impl is a noop
	#[inline]
	fn preallocate_capacity(capacity: usize) {}

	/// Provides a [`RawString`][`PoolAccess::RawString`] from a `&str`.
	/// Default impl just calls [`from_slice`][`PoolAccess::from_slice`] with
	/// `str.as_bytes()`
	#[inline]
	fn from_str(s: &str) -> Self::RawString {
		unsafe { Self::from_slice(s.as_bytes()) }
	}
}

/// The default string pool.
pub struct DefaultPool;

// all accesses to this static in this module use SeqCst, which
// could use a weaker ordering i think, however SeqCst
// is definitely correct, and this is only accessed once or twice at the start
// of programs, so the slower ordering isn't that big of a deal
static CAPACITY: AtomicUsize = AtomicUsize::new(0);

static POOL: LazyWrap<RwLock<HashSet<<DefaultPool as PoolProvider>::RawString>>> = LazyWrap::new(|| {
	let hashset = HashSet::with_capacity(CAPACITY.load(SeqCst));
	RwLock::new(hashset)
});

impl PoolProvider for DefaultPool {
	type RawString = Arc<Box<[u8]>>;

	unsafe fn from_slice(slice: &[u8]) -> Self::RawString {
		let slice = &ByteWrap(slice);
		let pool = POOL.read();

		if let Some(raw) = pool.get(slice) {
			let raw = Arc::clone(raw);
			drop(pool);
			raw
		} else {
			drop(pool);

			let mut pool = POOL.write();
			let raw = pool.get_or_insert_with(slice, |ByteWrap(slice)| {
				Arc::new(slice.to_vec().into_boxed_slice())
			});

			let raw = Arc::clone(raw);
			drop(pool);
			raw
		}
	}

	#[inline]
	fn deref_raw_to_slice(raw: &Self::RawString) -> &[u8] {
		raw
	}

	#[inline]
	fn clone_raw(raw: &Self::RawString) -> Self::RawString {
		Arc::clone(raw)
	}

	fn dropping_instance_of(slice: &[u8]) {
		// TODO
	}

	#[inline]
	fn preallocate_capacity(capacity: usize) {
		if !LazyWrap::is_initialised(&POOL) {
			CAPACITY.store(capacity, SeqCst);
		}
	}
}

#[repr(transparent)]
#[derive(Hash)]
struct ByteWrap<'h>(&'h [u8]);

impl<'h> Equivalent<Arc<Box<[u8]>>> for ByteWrap<'h> {
	#[inline]
	fn equivalent(&self, key: &Arc<Box<[u8]>>) -> bool {
		*self.0 == ***key
	}
}
