# string-pool

Global immutable string pool, like Java. Made for fun/educational/experimentation purposes. The goal is for the `String` exported from this crate to eventually be a drop-in replacement for std's `String`, but we're not quite there yet.

Should be okay to use, but at the moment, it is not highly tested. Use at your own risk! Do feel free to file issues for whatever reason too, and I'll get back to them as soon as I can.

Docs for the current published version of the crate can be found [on docs.rs](https://docs.rs/string-pool).

## Known issues

- Cannot implement `as_mut_str`, `as_mut_vec`
- Doesn't make sense to implement capacity related methods (makes no sense in the context of string pool)
  - `with_capacity`, `capacity`, `reserve`, `reserve_exact`, `try_reserve`, `try_reserve_exact`, `shrink_to_fit`, `shrink_to`
- Cannot implement `from_raw_parts`, `into_raw_parts`
