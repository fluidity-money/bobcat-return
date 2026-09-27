#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use bobcat_cd::const_keccak_sel;

use bobcat_entry::write_result_slice;

use bobcat_maths::{I, U};

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    None,
    Word(U),
    Address([u8; 20]),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvmRd<'a> {
    Nothing,
    Borrowed(&'a [u8]),
    Word([u8; 32]),
    Error([u8; 36]),

    #[cfg(feature = "alloc")]
    Bytes(alloc::vec::Vec<u8>),

    #[cfg(feature = "alloc")]
    String(alloc::string::String),
}

impl AsRef<[u8]> for EvmRd<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Nothing => &[],
            Self::Borrowed(x) => x,
            Self::Word(x) => x,
            Self::Error(x) => x,

            #[cfg(feature = "alloc")]
            Self::Bytes(x) => x,

            #[cfg(feature = "alloc")]
            Self::String(x) => x.as_bytes(),
        }
    }
}

#[inline]
fn bool_word(x: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = x as u8;
    out
}

#[inline]
fn i_from_i128(x: i128) -> I {
    let mut out = if x < 0 { [0xff; 32] } else { [0u8; 32] };

    out[16..].copy_from_slice(&x.to_be_bytes());

    I::from(out)
}

#[inline]
fn error(selector: [u8; 4], word: [u8; 32]) -> [u8; 36] {
    let mut out = [0u8; 36];

    out[..4].copy_from_slice(&selector);
    out[4..].copy_from_slice(&word);

    out
}

/*
 * Successful values -> EvmRd directly.
 */

impl<'a> From<U> for EvmRd<'a> {
    #[inline]
    fn from(x: U) -> Self {
        Self::Word(x.0)
    }
}

impl<'a> From<I> for EvmRd<'a> {
    #[inline]
    fn from(x: I) -> Self {
        Self::Word(x.0)
    }
}

impl<'a> From<[u8; 20]> for EvmRd<'a> {
    #[inline]
    fn from(x: [u8; 20]) -> Self {
        Self::Word(U::from(x).0)
    }
}

impl<'a> From<bool> for EvmRd<'a> {
    #[inline]
    fn from(x: bool) -> Self {
        Self::Word(bool_word(x))
    }
}

impl<'a> From<&'a [u8]> for EvmRd<'a> {
    #[inline]
    fn from(x: &'a [u8]) -> Self {
        Self::Borrowed(x)
    }
}

impl<'a> From<&'a str> for EvmRd<'a> {
    #[inline]
    fn from(x: &'a str) -> Self {
        Self::Borrowed(x.as_bytes())
    }
}

impl From<()> for EvmRd<'_> {
    #[inline]
    fn from(_: ()) -> Self {
        EvmRd::Nothing
    }
}

impl<'a> From<Response> for EvmRd<'a> {
    #[inline]
    fn from(x: Response) -> Self {
        match x {
            Response::None => Self::Nothing,
            Response::Word(x) => x.into(),
            Response::Address(x) => x.into(),
        }
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<alloc::vec::Vec<u8>> for EvmRd<'a> {
    #[inline]
    fn from(x: alloc::vec::Vec<u8>) -> Self {
        Self::Bytes(x)
    }
}

#[cfg(feature = "alloc")]
impl<'a> From<alloc::string::String> for EvmRd<'a> {
    #[inline]
    fn from(x: alloc::string::String) -> Self {
        Self::String(x)
    }
}

macro_rules! impl_unsigned_rd {
    ($($ty:ty),* $(,)?) => {
        $(
            impl<'a> From<$ty> for EvmRd<'a> {
                #[inline]
                fn from(x: $ty) -> Self {
                    Self::Word(U::from(x).0)
                }
            }
        )*
    };
}

impl_unsigned_rd! {
    u8,
    u16,
    u32,
    u64,
    u128,
}

impl<'a> From<usize> for EvmRd<'a> {
    #[inline]
    fn from(x: usize) -> Self {
        #[cfg(target_pointer_width = "32")]
        {
            Self::Word(U::from(x as u32).0)
        }

        #[cfg(target_pointer_width = "64")]
        {
            Self::Word(U::from(x as u64).0)
        }
    }
}

macro_rules! impl_signed_rd {
    ($($ty:ty),* $(,)?) => {
        $(
            impl<'a> From<$ty> for EvmRd<'a> {
                #[inline]
                fn from(x: $ty) -> Self {
                    Self::Word(i_from_i128(x as i128).0)
                }
            }
        )*
    };
}

impl_signed_rd! {
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
}

/*
 * Error values.
 *
 * Anything implementing this can be the E in Result<T, E>.
 */

pub trait IntoEvmRdError {
    fn into_evm_rd_error(self) -> [u8; 36];
}

impl IntoEvmRdError for U {
    #[inline]
    fn into_evm_rd_error(self) -> [u8; 36] {
        error(const_keccak_sel(b"Error(uint256)"), self.0)
    }
}

impl IntoEvmRdError for I {
    #[inline]
    fn into_evm_rd_error(self) -> [u8; 36] {
        error(const_keccak_sel(b"Error(int256)"), self.0)
    }
}

impl IntoEvmRdError for [u8; 20] {
    #[inline]
    fn into_evm_rd_error(self) -> [u8; 36] {
        error(const_keccak_sel(b"Error(address)"), U::from(self).0)
    }
}

impl IntoEvmRdError for bool {
    #[inline]
    fn into_evm_rd_error(self) -> [u8; 36] {
        error(const_keccak_sel(b"Error(bool)"), bool_word(self))
    }
}

macro_rules! impl_unsigned_error {
    ($(
        $ty:ty => $abi:literal
    ),* $(,)?) => {
        $(
            impl IntoEvmRdError for $ty {
                #[inline]
                fn into_evm_rd_error(self) -> [u8; 36] {
                    error(
                        const_keccak_sel(
                            concat!("Error(", $abi, ")").as_bytes()
                        ),
                        U::from(self).0,
                    )
                }
            }
        )*
    };
}

impl_unsigned_error! {
    u8   => "uint8",
    u16  => "uint16",
    u32  => "uint32",
    u64  => "uint64",
    u128 => "uint128",
}

impl IntoEvmRdError for usize {
    #[inline]
    fn into_evm_rd_error(self) -> [u8; 36] {
        #[cfg(target_pointer_width = "32")]
        {
            error(const_keccak_sel(b"Error(uint32)"), U::from(self as u32).0)
        }

        #[cfg(target_pointer_width = "64")]
        {
            error(const_keccak_sel(b"Error(uint64)"), U::from(self as u64).0)
        }
    }
}

macro_rules! impl_signed_error {
    ($(
        $ty:ty => $abi:literal
    ),* $(,)?) => {
        $(
            impl IntoEvmRdError for $ty {
                #[inline]
                fn into_evm_rd_error(self) -> [u8; 36] {
                    error(
                        const_keccak_sel(
                            concat!("Error(", $abi, ")").as_bytes()
                        ),
                        i_from_i128(self as i128).0,
                    )
                }
            }
        )*
    };
}

impl_signed_error! {
    i8   => "int8",
    i16  => "int16",
    i32  => "int32",
    i64  => "int64",
    i128 => "int128",
}

impl IntoEvmRdError for isize {
    #[inline]
    fn into_evm_rd_error(self) -> [u8; 36] {
        #[cfg(target_pointer_width = "32")]
        {
            error(
                const_keccak_sel(b"Error(int32)"),
                i_from_i128(self as i128).0,
            )
        }

        #[cfg(target_pointer_width = "64")]
        {
            error(
                const_keccak_sel(b"Error(int64)"),
                i_from_i128(self as i128).0,
            )
        }
    }
}

/*
 * Result<T, E> -> EvmRd directly.
 *
 * Ok(T)  -> T.into()
 * Err(E) -> EvmRd::Error(...)
 */

impl<'a, X, E> From<Result<X, E>> for EvmRd<'a>
where
    X: Into<EvmRd<'a>>,
    E: IntoEvmRdError,
{
    #[inline]
    fn from(x: Result<X, E>) -> Self {
        match x {
            Ok(x) => x.into(),
            Err(x) => Self::Error(x.into_evm_rd_error()),
        }
    }
}

/// Write the return data.
///
/// Returns 0 for a successful EVM return and 1 for a revert.
#[inline]
pub fn bobcat_return<'a, X>(x: X) -> usize
where
    X: Into<EvmRd<'a>>,
{
    let x = x.into();

    match x {
        EvmRd::Nothing => 0,

        EvmRd::Error(x) => {
            write_result_slice(&x);
            1
        }

        EvmRd::Borrowed(x) => {
            write_result_slice(x);
            0
        }

        EvmRd::Word(x) => {
            write_result_slice(&x);
            0
        }

        #[cfg(feature = "alloc")]
        EvmRd::Bytes(x) => {
            write_result_slice(&x);
            0
        }

        #[cfg(feature = "alloc")]
        EvmRd::String(x) => {
            write_result_slice(x.as_bytes());
            0
        }
    }
}

/// Write either a successful response or an application-defined encoded revert.
#[inline]
pub fn bobcat_catch_all<E, X, F>(result: Result<Response, E>, encode_error: F) -> usize
where
    X: AsRef<[u8]>,
    F: FnOnce(&E) -> X,
{
    match result {
        Ok(response) => bobcat_return(response),
        Err(error) => {
            let encoded = encode_error(&error);
            write_result_slice(encoded.as_ref());
            1
        }
    }
}

#[macro_export]
macro_rules! bobcat_rd_match {
    (
        $value:expr;
        $(
            $pat:pat $(if $guard:expr)? => $body:expr
        ),* $(,)?
    ) => {
        match $value {
            $(
                $pat $(if $guard)? => {
                    $crate::bobcat_return($body)
                }
            ),*
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_converts_to_evm_return_data() {
        assert_eq!(EvmRd::from(Response::None), EvmRd::Nothing);
        assert_eq!(
            EvmRd::from(Response::Word(U::from(7u8))),
            EvmRd::Word(U::from(7u8).0)
        );

        let address = [0x11; 20];
        assert_eq!(
            EvmRd::from(Response::Address(address)),
            EvmRd::Word(U::from(address).0)
        );
    }

    #[test]
    fn catch_all_returns_success_for_a_response() {
        let result: Result<Response, ()> = Ok(Response::None);

        assert_eq!(0, bobcat_catch_all(result, |_| []));
    }

    #[test]
    fn catch_all_returns_revert_for_an_encoded_error() {
        let result: Result<Response, u8> = Err(7);
        let mut encoded = false;

        let status = bobcat_catch_all(result, |error| {
            encoded = *error == 7;
            [0xde, 0xad, 0xbe, 0xef]
        });

        assert!(encoded);
        assert_eq!(1, status);
    }

    #[test]
    fn test_example() {
        assert_eq!(0, bobcat_return(Ok::<_, U>(U::from(123u32))));
    }

    #[test]
    fn error_u32() {
        let rd: EvmRd<'_> = Err::<U, _>(123u32).into();

        assert_eq!(
            rd,
            EvmRd::Error(error(const_keccak_sel(b"Error(uint32)"), U::from(123u32).0,))
        );
    }
}
