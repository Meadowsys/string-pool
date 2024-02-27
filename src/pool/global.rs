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
#[derive(Debug)]
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

#[cfg(test)]
mod tests {
	use super::*;
	use ::hashbrown::hash_map::DefaultHashBuilder;
	use ::rand::{ Rng, rngs::OsRng };
	use ::std::string::String as StdString;
	use ::std::iter::repeat;
	use ::std::hash::BuildHasher;

	fn rand_string() -> StdString {
		let mut vec = vec![' '; OsRng.gen_range(1..100)];
		OsRng.fill(&mut *vec);
		vec.into_iter().collect()
	}

	#[test]
	fn slices_wrap_iter_hash_and_eq() {
		let hash_builder = hashbrown::hash_map::DefaultHashBuilder::default();

		let hash_item = |item: &dyn Hash| {
			let mut hasher = hash_builder.build_hasher();
			item.hash(&mut hasher);
			hasher.finish()
		};

		for _ in 0..100 {
			// generate vec of random length 0-10, with strings 0-100 chars
			let strs = repeat(0u8)
				.take(OsRng.gen_range(1..10))
				.map(|_| rand_string())
				.collect::<Vec<_>>();

			// create instance of SliceHashWrap (joining strings)
			let pool_strs = strs.iter()
				.map(|s| &**s)
				.collect::<String>();
			let pool_strs = Arc::new(SliceHashWrap(pool_strs.into_bytes().into_boxed_slice()));

			// create instance of SlicesWrap
			let mut _slices = strs.iter()
				.map(|s| s.as_bytes())
				.collect::<Vec<_>>();
			let slices = SlicesWrap(&_slices);

			let hash_pool = hash_item(&pool_strs);
			let hash_slices = hash_item(&slices);

			// test hash eq
			assert_eq!(hash_pool, hash_slices, "hashes should be equal");
			// test actual eq
			assert!(slices.equivalent(&pool_strs), "pool and slices should be equal");

			let last = _slices.last_mut().unwrap();
			let last_str = unsafe { std::str::from_utf8_unchecked(last) };
			*last = &last[..last.len() - last_str.chars().last().unwrap().len_utf8()];

			let slices = SlicesWrap(&_slices);
			let hash_slices = hash_item(&slices);

			assert_ne!(hash_pool, hash_slices, "hashes should not be eq");
			assert!(!slices.equivalent(&pool_strs), "pool and slices should not be equal");
		}
	}
}
