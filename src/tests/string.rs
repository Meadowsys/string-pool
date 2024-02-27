use crate::Pool;
use super::*;
use ::std::fmt::Debug;

#[derive(Clone)]
struct TestPool;

impl Pool for TestPool {
	type Raw = StdString;

	unsafe fn raw_from_slices(&self, slices: SlicesWrap) -> Self::Raw {
		let vec = slices.to_boxed_slice().into_vec();
		unsafe { StdString::from_utf8_unchecked(vec) }
	}

	fn raw_to_slice<'r>(&self, raw: &'r Self::Raw) -> &'r [u8] {
		raw.as_bytes()
	}
}

#[test]
fn new() {
	let empty = "";
	let new = String::new();
	let new_custom_pool = String::new_in(TestPool);

	assert_eq!(empty, new.as_str());
	assert_eq!(empty, new_custom_pool.as_str());
}

#[test]
fn from_utf8_and_slice() {
	fn assert_ok(s: &str, vec: Vec<u8>) {
		let res = String::from_utf8(vec.clone());
		assert!(res.is_ok());
		assert_eq!(&*res.unwrap(), s);

		let res = String::from_utf8_in(vec.clone(), TestPool);
		assert!(res.is_ok());
		assert_eq!(&*res.unwrap(), s);

		let res = String::from_utf8_slice(&vec);
		assert!(res.is_ok());
		assert_eq!(&*res.unwrap(), s);

		let res = String::from_utf8_slice_in(&vec, TestPool);
		assert!(res.is_ok());
		assert_eq!(&*res.unwrap(), s);
	}

	fn assert_err(vec: Vec<u8>) {
		assert!(String::from_utf8(vec.clone()).is_err());
		assert!(String::from_utf8_in(vec.clone(), TestPool).is_err());
		assert!(String::from_utf8_slice(&vec).is_err());
		assert!(String::from_utf8_slice_in(&vec, TestPool).is_err());
	}

	// from macOS's text thingie
	// 🫐
	// bosbessen
	// Unicode: U+1FAD0, UTF-8: 0xF0, 0x9F, 0xAB, 0x90,
	assert_ok("🫐", vec![0xF0u8, 0x9F, 0xAB, 0x90]);

	let complex_string = "programmering er gøy 烏龜很喜歡吃藍莓 ik weet het niet, ik schrijf gewoon willekeurig zinnen lol okay this is a good \n\n test of unicode right? ✨ ÙwÚ ✨";
	assert_ok(complex_string, complex_string.as_bytes().to_vec());

	// stolen from std String's from_utf8 doc
	assert_err(vec![0u8, 159, 146, 150]);

}
