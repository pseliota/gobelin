//! Shared functionality for Gobelin.

pub mod alpaca;
pub mod config;
pub mod fred;

mod transport;

pub use transport::{ClientError, RetryPolicy};
