use hashbrown::Equivalent;
use std::sync::Arc;

pub(crate) use inner::RawString;

fn get_existing(bytes: &[u8]) -> Result<RawString, inner::MutexLock> {
	let mut lock = inner::get_pool().lock();
	match lock.get(&ByteWrap(bytes)) {
		Some(s) => { Ok(Arc::clone(s)) }
		None => { Err(lock) }
	}
}

pub(crate) unsafe fn from_slice(slice: &[u8]) -> RawString {
	match get_existing(slice) {
		Ok(s) => { s }
		Err(mut lock) => {
			// SAFETY: get_existing checked the value doesn't exist in the set
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
	pub(crate) type MutexLock = MutexGuard<'static, HashSet<RawString>>;

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
