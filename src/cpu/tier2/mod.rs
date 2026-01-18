//! Tier 2 execution: Decode cache
//!
//! This module implements the decode cache for frequently executed instructions.
//! When an instruction is first decoded via tier 1, it is cached so that subsequent
//! executions of the same instruction (at the same address) can skip decoding.
//!
//! This is particularly effective for loops where the same instructions are executed
//! many times.

mod cache;

pub use cache::DecodeCache;
