//! # skuld
//!
//! Error and logging utility crate
//! Includes the following:
//!  - `bail`: A macro to return an error from a function
//!  - `location`: A macro to get the location of the call (i.e. a tuple with (file!(), line!(),
//!    column!())
//!  - `SkuldLogger`: A `log` crate facade that writes to the disk.

#[cfg(feature = "facade")]
mod logger;

/// # bail! macro
///
/// A simple macro to return an error from a function. Runs .into() for you!
///
/// ## Example
///
/// ```should_panic
/// use std::io;
/// use skuld::bail;
///
/// #[derive(Debug)]
/// pub struct Error {
///    e: io::Error,
/// }
///
/// impl From<std::io::Error> for Error {
/// 	fn from(e: io::Error) -> Self {
/// 		Self { e }
/// 	}
/// }
///
/// fn my_function() -> Result<(), Error> {
///     bail!(io::Error::new(io::ErrorKind::AddrInUse, String::from("some error")));
///
/// 	Ok(())
/// }
///
/// my_function().unwrap(); // panics with Error { e: io::Error { ... } }
/// ```
#[cfg(feature = "bail")]
#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into());
    };
}

/// # location! macro
///
/// Tuple with `(file!(), line!(), column!())`
#[cfg(feature = "location")]
#[macro_export]
macro_rules! location {
    () => {
        (::core::file!(), ::core::line!(), ::core::column!())
    };
}

#[cfg(feature = "facade")]
pub use logger::prelude as log;
