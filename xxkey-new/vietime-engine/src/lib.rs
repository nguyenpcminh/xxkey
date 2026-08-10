//! vietime-engine — Vietnamese input engine (pure Rust, no_std).
//!
//! Ported from the XXKey/OpenKey C++ engine (`Engine.cpp`, `Vietnamese.cpp`,
//! `DataType.h`). The engine is a deterministic state machine: feed it key
//! events, it returns a `HookState` describing how the front-end should edit
//! the text buffer (backspace count + new characters).
//!
//! Design goals:
//! - `no_std`, no heap, no panics in the hot path.
//! - 100% unit-tested against the original C++ behaviour.
//! - The same `TypingWord` buffer model as the original so a byte-for-byte
//!   port is auditable.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod datatype;
pub mod engine;
pub mod keycode;
pub mod vietnamese;

pub use datatype::*;
pub use engine::*;
pub use keycode::*;
pub use vietnamese::*;
