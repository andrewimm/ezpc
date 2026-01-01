//! Tier 1 execution (cold path) - Direct dispatch
//!
//! The first time code is executed, it goes through tier 1 which:
//! - Decodes operands on-the-fly
//! - Dispatches to handler functions
//! - Does not cache decoded instructions

pub mod decode;
pub mod dispatch;

pub use dispatch::DISPATCH_TABLE;
