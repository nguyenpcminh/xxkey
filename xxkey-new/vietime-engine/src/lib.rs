//! vietime-engine — Vietnamese input engine (pure Rust, no_std).
//!
//! Ported from the XXKey/OpenKey C++ engine (`Engine.cpp`, `Vietnamese.cpp`,
//! `Macro.cpp`, `SmartSwitchKey.cpp`, `ConvertTool.cpp`, `DataType.h`). The
//! engine is a deterministic state machine: feed it key events, it returns a
//! `HookState` describing how the front-end should edit the text buffer
//! (backspace count + new characters).

#![cfg_attr(feature = "no_std", no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod convert;
pub mod datatype;
pub mod engine;
pub mod keycode;
pub mod macro_feature;
pub mod smart_switch;
pub mod vietnamese;

#[allow(unsafe_code)]
pub mod ffi;

pub use convert::*;
pub use datatype::*;
pub use engine::*;
pub use keycode::*;
pub use macro_feature::*;
pub use smart_switch::*;
pub use vietnamese::*;
