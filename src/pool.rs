use ::hashbrown::{ Equivalent, HashSet };
use ::lazy_wrap::LazyWrap;
use ::parking_lot::Mutex;
use ::std::sync::Arc;

pub(crate) type RawString = Arc<Box<[u8]>>;
type Pool = Mutex<HashSet<RawString>>;

static POOL: LazyWrap<Pool> = LazyWrap::new(Pool::default);

#[repr(transparent)]
#[derive(Hash)]
struct ByteWrap<'h>(&'h [u8]);

impl<'h> Equivalent<Arc<Box<[u8]>>> for ByteWrap<'h> {
	#[inline]
	fn equivalent(&self, key: &Arc<Box<[u8]>>) -> bool {
		*self.0 == ***key
	}
}

#[inline]
fn arc_from_bytewrap(ByteWrap(slice): &ByteWrap) -> RawString {
	Arc::new(slice.to_vec().into_boxed_slice())
}

#[inline]
pub(crate) unsafe fn from_slice(slice: &[u8]) -> RawString {
	let mut pool = POOL.lock();

	Arc::clone(pool.get_or_insert_with(
		&ByteWrap(slice),
		arc_from_bytewrap
	))
}

pub(crate) fn from_str(s: &str) -> RawString {
	unsafe { from_slice(s.as_bytes()) }
}
