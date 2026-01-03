//! Instruction execution handlers
//!
//! This module contains handler functions for all 8088 instructions.
//! Handlers are organized by instruction category.

pub mod arithmetic;
pub mod control_flow;
pub mod data_transfer;
pub mod flags;
pub mod handlers;
pub mod io;
pub mod logic;
pub mod prefix;
pub mod stack;
pub mod string;

// Re-export commonly used handlers
pub use handlers::{invalid_opcode, nop};
