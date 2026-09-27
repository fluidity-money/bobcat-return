
# bobcat-return

Helper type and functions for returning structured data from bobcat-sdk contracts.

```rust
bobcat_return(Ok(U::from(123)))
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