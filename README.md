
# bobcat-return

Helper type and functions for returning structured data from bobcat-sdk contracts.

```rust
bobcat_return(Ok(U::from(123)))
```

Contracts with a common response enum and application-specific error encoding can use
`bobcat_catch_all`:

```rust
use bobcat_return::{Response, bobcat_catch_all};

fn write_dispatch_result(result: Result<Response, Error>) -> usize {
    bobcat_catch_all(result, encode_error)
}
```

A macro is provided for match statements that are common to the entrypoint decoding method:

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    assert!(!unsafe { msg_reentrant() });
    flush_guard(|| bobcat_rd_match! {
        read_cd::<_>(args_len);

        Entry::Number => storage_load(&U::ZERO),
        Entry::SetNumber(w) => storage_store(&U::ZERO, &w),
        Entry::MulNumber(w) => storage_wrapping_mul(&U::ZERO, &w),
        Entry::AddNumber(w) => storage_wrapping_add(&U::ZERO, &w),
        Entry::Increment => storage_wrapping_add(&U::ZERO, &U::ONE),
        Entry::AddFromMsgValue => storage_wrapping_add(&U::ZERO, &msg_value()),
    });
    0
}
```

## Why is this not in bobcat-sdk?

This is pretty ugly, llm generated (with human feedback), and evolving based on needs
we've observed in our long range AI slop contract development.

We don't believe in developing contracts with AI but we've been using it in contexts where
it's safe (and there is no potential for value to be lost).
