#![feature(stmt_expr_attributes)]

extern crate async_std;
extern crate futures;
extern crate log;

#[cfg(feature = "events")]
mod events;

#[cfg(feature = "queue")]
mod queue;

#[cfg(feature = "events")]
pub use events::*;

#[cfg(feature = "queue")]
pub use queue::*;
