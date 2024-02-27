# string-pool

Global immutable string pool, like Java. Made for fun/educational/experimentation purposes. The goal is for the `String` exported from this crate to eventually be a drop-in replacement for std's `String`, but we're not quite there yet.

Should be okay to use, but at the moment, it is not highly tested. Use at your own risk! Do feel free to file issues for whatever reason too, and I'll get back to them as soon as I can.

Docs for the current published version of the crate can be found [on docs.rs](https://docs.rs/string-pool).

## Known issues

### The nature of a string pool

Cannot implement the following, due to it not making sense in the context of string pool:

- Borrowing as foreign mutable type (as we have no control what happens internally): `as_mut_str`, `as_mut_vec`, `AsMut`, `BorrowMut`
- Capacity related methods: `with_capacity`, `capacity`, `reserve`, `reserve_exact`, `try_reserve`, `try_reserve_exact`, `shrink_to_fit`, `shrink_to`
- Raw pointer related: `from_raw_parts`, `into_raw_parts`

### Trait limitations

- `Debug` impl requires the pool and raw value to also implement `Debug`
- `Default` impl requires the pool to also implement `Default`
