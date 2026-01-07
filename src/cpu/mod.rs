//! 8088 CPU emulation module
//!
//! This module implements the Intel 8088 processor with three-tier execution:
//! - Tier 1: Direct dispatch (cold path)
//! - Tier 2: Decode cache (warm path)
//! - Tier 3: Compiled basic blocks (hot path)

pub mod decode;
pub mod execute;
pub mod harness;
pub mod state;
pub mod tier1;
pub mod timing;

pub use harness::CpuHarness;
pub use state::Cpu;
