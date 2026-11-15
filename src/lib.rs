//! Shared functionality for Gobelin.

pub mod alpaca;

mod transport;

pub use transport::{ClientError, RetryPolicy};
