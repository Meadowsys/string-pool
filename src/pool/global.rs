use super::{ Pool, SlicesWrap };
use ::hashbrown::{ Equivalent, HashSet };
use ::lazy_wrap::LazyWrap;
use ::parking_lot::RwLock;
use ::std::hash::{ Hash, Hasher };
use ::std::sync::Arc;

/// The default, global string pool
#[derive(Clone)]
pub struct GlobalPool;

/// The actual backing store for the default global pool
static POOL: LazyWrap<RwLock<HashSet<<GlobalPool as Pool>::Raw>>> = LazyWrap::new(|| {
	let set = HashSet::new();
	RwLock::new(set)
});

impl Pool for GlobalPool {
	type Raw = Arc<SliceHashWrap>;

	unsafe fn raw_from_slices(&self, slices: SlicesWrap) -> Self::Raw {
		let pool = POOL.read();

		if let Some(raw) = pool.get(&slices) {
			let raw = Arc::clone(raw);
			drop(pool);
			raw
		} else {
			drop(pool);

			let mut pool = POOL.write();
			let raw = pool.get_or_insert_with(&slices, |slices| {
				Arc::new(SliceHashWrap(slices.to_boxed_slice()))
			});

			let raw = Arc::clone(raw);
			drop(pool);
			raw
		}
	}

	fn raw_to_slice<'r>(&self, raw: &'r Self::Raw) -> &'r [u8] {
		&raw.0
	}
}

/// Wrapper for `Box<[u8]>` that hashes the slice within by repeatedly
/// calling `Hasher::write_u8`, matching [`Hash`] impl of [`SlicesWrap`]
#[repr(transparent)]
pub struct SliceHashWrap(Box<[u8]>);

impl Hash for SliceHashWrap {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.iter().copied()
			.for_each(|b| state.write_u8(b));
	}
}

impl PartialEq for SliceHashWrap {
	fn eq(&self, other: &Self) -> bool {
		*self.0 == *other.0
	}
}

impl Eq for SliceHashWrap {}

impl<'h> Equivalent<<GlobalPool as Pool>::Raw> for SlicesWrap<'h> {
	fn equivalent(&self, key: &<GlobalPool as Pool>::Raw) -> bool {
		let mut iter1 = key.0.iter().copied();
		let mut iter2 = self.into_iter();

		loop {
			// possible outcomes:
			// Some Some
			//    - if a != b, false
			//    - if a == b, not done yet, continue
			// Some None
			// None Some
			//    - both of these are different lengths
			//    - false
			// None None
			//    - if we haven't returned yet, both iters are same length and all equal
			//    - true
			match (iter1.next(), iter2.next()) {
				(Some(a), Some(b)) if a != b => { return false }
				(Some(_), None) | (None, Some(_)) => { return false }
				(None, None) => { return true }
				(Some(a), Some(b)) => { continue }
			}
		}
	}
}
