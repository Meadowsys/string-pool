use hashbrown::Equivalent;
use std::sync::Arc;

pub(crate) use inner::RawString;

pub(crate) unsafe fn from_slice(slice: &[u8]) -> RawString {
	let mut lock = inner::get_pool().lock();

	match lock.get(&ByteWrap(slice)) {
		Some(s) => { Arc::clone(s) }
		None => {
			// SAFETY: we just checked, it doesn't exist
			let val = lock.insert_unique_unchecked(Arc::new(slice.to_vec().into_boxed_slice()));

			Arc::clone(val)
		}
	}
}

#[inline(always)]
pub(crate) fn from_str(s: &str) -> RawString {
	// SAFETY: a str is guaranteed to be valid utf8
	unsafe { from_slice(s.as_bytes()) }
}

// stop the world string GC, woohoo lol
// TODO: this could be more efficient, maybe?
pub fn gc() -> usize {
	let mut lock = inner::get_pool().lock();

	let to_be_deleted = lock.iter()
		.filter(|arc| Arc::strong_count(arc) == 1)
		.map(Arc::clone)
		.collect::<Vec<_>>();

	let num = to_be_deleted.len();

	for arc in to_be_deleted {
		debug_assert!(lock.remove(&ByteWrap(&arc)));
	}

	drop(lock);
	num
}

#[derive(Debug, Clone)]
pub enum Stats {
	Empty,
	Populated {
		total: usize,
		non_referenced: usize,
		referenced: usize,
		max_ref_count: usize,
		min_ref_count: usize,
		max_byte_len: usize,
		min_byte_len: usize
	}
}

pub fn stats() -> Stats {
	let pool = inner::get_pool();
	let lock = pool.lock();

	if lock.is_empty() { return Stats::Empty }

	let total = lock.len();
	let mut non_referenced = 0usize;
	let mut referenced = 0usize;

	let mut max_ref_count = 0usize;
	let mut min_ref_count = usize::MAX;

	let mut max_byte_len = 0usize;
	let mut min_byte_len = usize::MAX;

	for s in lock.iter() {
		let strong_count = Arc::strong_count(s);
		let len = s.len();

		if strong_count == 1 { non_referenced += 1 }
		else { referenced += 1 }

		max_ref_count = usize::max(max_ref_count, strong_count - 1);
		min_ref_count = usize::min(min_ref_count, strong_count - 1);

		max_byte_len = usize::max(max_byte_len, len);
		min_byte_len = usize::min(min_byte_len, len);
	}

	drop(lock);

	Stats::Populated {
		total,
		non_referenced,
		referenced,
		max_ref_count,
		min_ref_count,
		max_byte_len,
		min_byte_len
	}
}

#[repr(transparent)]
#[derive(Hash)]
struct ByteWrap<'h>(&'h [u8]);

impl Equivalent<Arc<Box<[u8]>>> for ByteWrap<'_> {
	#[inline]
	fn equivalent(&self, key: &Arc<Box<[u8]>>) -> bool {
		*self.0 == ***key
	}
}

mod inner {
	use hashbrown::HashSet;
	use parking_lot::{ Mutex, MutexGuard, Once };
	use std::mem::MaybeUninit;
	use std::sync::Arc;

	type Pool = Mutex<HashSet<RawString>>;
	pub(crate) type RawString = Arc<Box<[u8]>>;
	pub(super) type MutexLock = MutexGuard<'static, HashSet<RawString>>;

	static mut POOL: MaybeUninit<Pool> = MaybeUninit::uninit();
	static ONCE: Once = Once::new();

	#[inline(always)]
	pub(super) fn get_pool() -> &'static Pool {
		ONCE.call_once(
			#[inline(always)]
			|| unsafe { POOL.write(Mutex::new(HashSet::new())); }
		);

		unsafe { POOL.assume_init_ref() }
	}
}
