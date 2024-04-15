//! # skuld
//!
//! Error and logging utility crate
//! Includes the following:
//!  - `bail!`: A macro to return an error from a function
//!  - `location!`: Get the full location information of the call (using file/line/column macros)
//!  - `SkuldLogger`: A `log` crate facade that writes to the disk.

#[cfg(feature = "location")]
use std::fmt;

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

/// `ProvideLocation`
///
/// Similar to `std::panic::Location,` but without lifetimes and we can instantiate
/// it manually.
#[cfg(feature = "location")]
pub struct ProvideLocation(&'static str, u32, u32);

#[cfg(feature = "location")]
impl fmt::Display for ProvideLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.0, self.1, self.2)
    }
}

#[cfg(feature = "location")]
impl fmt::Debug for ProvideLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

/// # location! macro
///
/// returns a `skuld::ProvideLocation`
#[cfg(feature = "location")]
#[macro_export]
macro_rules! location {
    () => {
        ::skuld::ProvideLocation(::core::file!(), ::core::line!(), ::core::column!())
    };
}

#[cfg(feature = "facade")]
pub use logger::prelude as log;
